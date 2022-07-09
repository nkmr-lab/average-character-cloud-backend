use std::future::Future;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use juniper::FieldResult;
use juniper::{
    graphql_interface, EmptySubscription, GraphQLInputObject, GraphQLObject, RootNode, ID,
};
use sqlx::PgPool;
use thiserror::Error;
use ulid::Ulid;

use crate::entities;
use anyhow::{anyhow, ensure, Context};
use std::fmt;
#[derive(Debug)]
pub struct AppCtx {
    pub pool: PgPool,
    pub user_id: Option<String>,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub struct GraphqlUserError {
    #[source]
    pub source: anyhow::Error,
}

impl fmt::Display for GraphqlUserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl From<anyhow::Error> for GraphqlUserError {
    fn from(source: anyhow::Error) -> Self {
        Self { source }
    }
}

impl From<&str> for GraphqlUserError {
    fn from(source: &str) -> Self {
        Self {
            source: anyhow!("{}", source),
        }
    }
}

async fn handler<T, Fut: Future<Output = anyhow::Result<T>>>(
    f: impl FnOnce() -> Fut,
) -> FieldResult<T> {
    match f().await {
        Ok(value) => Ok(value),
        Err(err) => Err(match err.downcast_ref::<GraphqlUserError>() {
            Some(err) => err.source.to_string().into(),
            None => {
                log::error!("{:?}", err);
                "Internal error".into()
            }
        }),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum LimitKind {
    First,
    Last,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Limit {
    kind: LimitKind,
    value: i32,
}

impl Limit {
    fn encode(first: Option<i32>, last: Option<i32>) -> anyhow::Result<Self> {
        let max = 100;

        match (first, last) {
            (Some(first), None) => {
                if first < 0 {
                    Err(GraphqlUserError::from("first must be greater than or equal to 0").into())
                } else if first > max {
                    Err(GraphqlUserError::from(
                        format!("first must be less than or equal to {}", max).as_str(),
                    )
                    .into())
                } else {
                    Ok(Limit {
                        kind: LimitKind::First,
                        value: first,
                    })
                }
            }
            (None, Some(last)) => {
                if last < 0 {
                    Err(GraphqlUserError::from("last must be greater than or equal to 0").into())
                } else if last > max {
                    Err(GraphqlUserError::from(
                        format!("last must be less than or equal to {}", max).as_str(),
                    )
                    .into())
                } else {
                    Ok(Limit {
                        kind: LimitKind::Last,
                        value: last,
                    })
                }
            }
            _ => Err(GraphqlUserError::from("Must provide either first or last, not both").into()),
        }
    }
}

impl juniper::Context for AppCtx {}

#[derive(Debug, Clone)]
enum NodeID {
    FigureRecord(Ulid),
    CharacterConfig(Ulid),
    Character(entities::character::Character),
}

impl NodeID {
    fn to_id(&self) -> ID {
        match self {
            NodeID::FigureRecord(id) => ID::new(base64::encode(format!("FigureRecord:{}", id))),
            NodeID::CharacterConfig(id) => {
                ID::new(base64::encode(format!("CharacterConfig:{}", id)))
            }
            NodeID::Character(character) => ID::new(base64::encode(format!(
                "Character:{}",
                String::from(character.clone())
            ))),
        }
    }

    fn from_id(id: &ID) -> Option<NodeID> {
        let id = id.to_string();
        let buf = base64::decode(id).ok()?;
        let s = String::from_utf8(buf).ok()?;

        if let Some(record_id) = s.strip_prefix("FigureRecord:") {
            let ulid = Ulid::from_str(record_id).ok()?;
            Some(NodeID::FigureRecord(ulid))
        } else if let Some(character_id) = s.strip_prefix("CharacterConfig:") {
            let ulid = Ulid::from_str(character_id).ok()?;
            Some(NodeID::CharacterConfig(ulid))
        } else if let Some(character) = s.strip_prefix("Character:") {
            Some(NodeID::Character(
                entities::character::Character::try_from(character).ok()?,
            ))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
struct FigureRecordModel {
    id: String,
    user_id: String,
    character: String,
    figure: serde_json::Value,
    created_at: DateTime<Utc>,
    stroke_count: i32,
}

impl FigureRecordModel {
    fn into_entity(self) -> anyhow::Result<entities::figure_record::FigureRecord> {
        let id = Ulid::from_str(&self.id).context("ulid decode error")?;

        let character = entities::character::Character::try_from(self.character.as_str())?;

        let figure = entities::figure::Figure::from_json_ast(self.figure)
            .ok_or_else(|| anyhow!("figure must be valid json"))?;

        ensure!(
            self.stroke_count as usize == figure.strokes.len(),
            "stroke_count invalid"
        );

        Ok(entities::figure_record::FigureRecord {
            id,
            user_id: self.user_id,
            character,
            figure,
            created_at: self.created_at,
        })
    }
}

#[derive(Debug, Clone)]
struct CharacterConfigModel {
    id: String,
    user_id: String,
    character: String,
    stroke_count: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: i32,
}

impl CharacterConfigModel {
    fn into_entity(self) -> anyhow::Result<entities::character_config::CharacterConfig> {
        let id = Ulid::from_str(&self.id).context("ulid decode error")?;

        let character = entities::character::Character::try_from(self.character.as_str())?;

        Ok(entities::character_config::CharacterConfig {
            id,
            user_id: self.user_id,
            character,
            stroke_count: self.stroke_count as usize,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

/**
 * replayの仕様に従うこと
 *   https://relay.dev/docs/guides/graphql-server-specification/
 *   https://relay.dev/graphql/connections.htm
*/

#[graphql_interface(for = [FigureRecord, CharacterConfig, Character], context = AppCtx)]
trait Node {
    fn id(&self) -> ID;
}

#[derive(GraphQLObject, Clone, Debug)]
struct PageInfo {
    has_next_page: bool,
    has_previous_page: bool,
    start_cursor: Option<String>,
    end_cursor: Option<String>,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(impl = NodeValue, context = AppCtx)]
struct FigureRecord {
    id: ID,
    figure_record_id: String,
    character: Character,
    figure: String,
    created_at: String,
}

impl FigureRecord {
    fn from_entity(record: &entities::figure_record::FigureRecord) -> FigureRecord {
        FigureRecord {
            id: NodeID::FigureRecord(record.id).to_id(),
            figure_record_id: record.id.to_string(),
            character: Character::from_entity(record.character.clone()),
            figure: record.figure.to_json(),
            created_at: record.created_at.to_rfc3339(),
        }
    }
}

#[graphql_interface]
impl Node for FigureRecord {
    fn id(&self) -> ID {
        self.id.clone()
    }
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct FigureRecordEdge {
    cursor: String,
    node: FigureRecord,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct FigureRecordConnection {
    page_info: PageInfo,
    edges: Vec<FigureRecordEdge>,
}

#[derive(GraphQLInputObject, Clone, Debug)]
struct CreateFigureRecordInput {
    character: String,
    figure: String,
}

#[derive(Clone, Debug)]
struct CharacterConfig {
    id: ID,
    character_config_id: String,
    character: Character,
    stroke_count: i32,
    created_at: String,
    updated_at: String,
}

impl CharacterConfig {
    fn from_entity(character: &entities::character_config::CharacterConfig) -> CharacterConfig {
        CharacterConfig {
            id: NodeID::CharacterConfig(character.id).to_id(),
            character_config_id: character.id.to_string(),
            character: Character::from_entity(character.character.clone()),
            stroke_count: character.stroke_count as i32,
            created_at: character.created_at.to_rfc3339(),
            updated_at: character.updated_at.to_rfc3339(),
        }
    }
}

#[graphql_interface]
impl Node for CharacterConfig {
    fn id(&self) -> ID {
        self.id.clone()
    }
}

#[juniper::graphql_object(Context = AppCtx, impl = NodeValue)]
impl CharacterConfig {
    fn character_config_id(&self) -> String {
        self.character_config_id.clone()
    }

    fn character(&self) -> Character {
        self.character.clone()
    }

    fn stroke_count(&self) -> i32 {
        self.stroke_count
    }

    fn created_at(&self) -> String {
        self.created_at.clone()
    }

    fn updated_at(&self) -> String {
        self.updated_at.clone()
    }

    async fn figure_records(
        &self,
        ctx: &AppCtx,
        ids: Option<Vec<ID>>,
        first: Option<i32>,
        after: Option<String>,
        last: Option<i32>,
        before: Option<String>,
    ) -> FieldResult<FigureRecordConnection> {
        // TODO: N+1
        handler(|| async {
            let Some(user_id) = ctx.user_id.clone() else {
                return Err(GraphqlUserError::from("Authentication required").into());
            };

            let characters = vec![self.character.entity.clone()];

            let ids = ids
                .map(|ids| -> anyhow::Result<Vec<Ulid>> {
                    ids.into_iter()
                        .map(|id| -> anyhow::Result<Ulid> {
                            Ulid::from_str(id.to_string().as_str())
                                .map_err(|_| GraphqlUserError::from("invalid ids").into())
                        })
                        .collect::<anyhow::Result<Vec<_>>>()
                })
                .transpose()?;

            let limit = Limit::encode(first, last)?;

            let after_id = after
                .map(|after| -> anyhow::Result<Ulid> {
                    let Some(NodeID::FigureRecord(id)) = NodeID::from_id(&ID::new(after)) else {
                    return Err(GraphqlUserError::from("after must be a valid cursor").into())
                };

                    Ok(id)
                })
                .transpose()?;

            let before_id = before
                .map(|before| -> anyhow::Result<Ulid> {
                    let Some(NodeID::FigureRecord(id)) = NodeID::from_id(&ID::new(before)) else {
                    return Err(GraphqlUserError::from("before must be a valid cursor").into());
                };

                    Ok(id)
                })
                .transpose()?;

            let characters = characters
                .into_iter()
                .map(|c| String::from(c))
                .collect::<Vec<_>>();

            let ids = ids.map(|ids| ids.into_iter().map(|id| id.to_string()).collect::<Vec<_>>());

            /*
            本来文字とストローク数のペアはselfから参照するべきだが、タプルの配列をINするのは難しいので, JOINしてテーブルから参照する
            mutation後のqueryといったエッジケースを除いて正常に動作するはず
            */
            let result = sqlx::query_as!(
                FigureRecordModel,
                r#"
                SELECT
                    id,
                    user_id,
                    character,
                    figure,
                    created_at,
                    stroke_count
                FROM (
                    SELECT
                        r.id,
                        r.user_id,
                        r.character,
                        r.figure,
                        r.created_at,
                        r.stroke_count,
                        rank() OVER (
                            PARTITION BY r.character
                            ORDER BY
                                CASE WHEN $6 = 0 THEN r.id END DESC,
                                CASE WHEN $6 = 1 THEN r.id END ASC
                        ) AS rank
                    FROM
                        figure_records AS r
                    JOIN
                        character_configs AS c ON r.character = c.character AND r.user_id = c.user_id
                    WHERE
                        r.user_id = $1
                        AND
                        r.character = Any($2)
                        AND
                        ($3::VARCHAR(64)[] IS NULL OR r.id = Any($3))
                        AND
                        ($4::VARCHAR(64) IS NULL OR r.id < $4)
                        AND
                        ($5::VARCHAR(64) IS NULL OR r.id > $5)
                        AND
                        r.stroke_count = c.stroke_count
                ) as r
                WHERE
                    rank <= $7
                ORDER BY
                    id DESC
            "#,
                &user_id,
                characters.as_slice(),
                ids.as_ref().map(|ids| ids.as_slice()),
                after_id.map(|id| id.to_string()),
                before_id.map(|id| id.to_string()),
                (limit.kind == LimitKind::Last) as i32,
                limit.value as i64 + 1,
            )
            .fetch_all(&ctx.pool)
            .await
            .context("fetch figure_records")?;

            let mut records = result
                .into_iter()
                .map(|row| row.into_entity())
                .collect::<anyhow::Result<Vec<_>>>()
                .context("convert records")?;

            let has_extra = records.len() > limit.value as usize;

            records.truncate(limit.value as usize);

            let records = records
                .into_iter()
                .map(|record| FigureRecord::from_entity(&record))
                .collect::<Vec<_>>();

            Ok(FigureRecordConnection {
                page_info: PageInfo {
                    has_next_page: has_extra && limit.kind == LimitKind::First,
                    has_previous_page: has_extra && limit.kind == LimitKind::Last,
                    start_cursor: records.first().map(|record| record.id.to_string()),
                    end_cursor: records.last().map(|record| record.id.to_string()),
                },
                edges: records
                    .into_iter()
                    .map(|record| FigureRecordEdge {
                        cursor: record.id.to_string(),
                        node: record,
                    })
                    .collect(),
            })
        })
        .await
    }
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct CharacterConfigEdge {
    cursor: String,
    node: CharacterConfig,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct CharacterConfigConnection {
    page_info: PageInfo,
    edges: Vec<CharacterConfigEdge>,
}

#[derive(GraphQLInputObject, Clone, Debug)]
struct CreateCharacterConfigInput {
    character: String,
    stroke_count: i32,
}

#[derive(GraphQLInputObject, Clone, Debug)]
struct UpdateCharacterConfigInput {
    id: ID,
    stroke_count: Option<i32>,
}

#[derive(Clone, Debug)]
struct Character {
    entity: entities::character::Character,
}

#[graphql_interface]
impl Node for Character {
    fn id(&self) -> ID {
        NodeID::Character(self.entity.clone()).to_id()
    }
}

impl Character {
    fn from_entity(entity: entities::character::Character) -> Character {
        Character { entity }
    }
}

#[juniper::graphql_object(Context = AppCtx, impl = NodeValue)]
impl Character {
    fn value(&self) -> String {
        String::from(self.entity.clone())
    }

    async fn character_config(&self, ctx: &AppCtx) -> FieldResult<Option<CharacterConfig>> {
        // TODO: N+1
        handler(|| async {
            let Some(user_id) = ctx.user_id.clone() else {
                return Err(GraphqlUserError::from("Authentication required").into());
            };

            let result = sqlx::query_as!(
                CharacterConfigModel,
                r#"
                SELECT
                    id,
                    user_id,
                    character,
                    stroke_count,
                    created_at,
                    updated_at,
                    version
                FROM
                    character_configs
                WHERE
                    user_id = $1
                    AND
                    character = $2
            "#,
                &user_id,
                String::from(self.entity.clone()),
            )
            .fetch_all(&ctx.pool)
            .await
            .context("fetch character_configs")?;

            let character = result
                .into_iter()
                .next()
                .map(|row| row.into_entity())
                .transpose()
                .context("convert CharacterConfig")?;

            let record = character.map(|character| CharacterConfig::from_entity(&character));
            Ok(record)
        })
        .await
    }
}

#[derive(Clone, Debug)]
pub struct QueryRoot;
#[juniper::graphql_object(Context = AppCtx, name = "Query")]
impl QueryRoot {
    fn query() -> QueryRoot {
        QueryRoot
    }

    async fn node(ctx: &AppCtx, id: ID) -> FieldResult<Option<NodeValue>> {
        handler(|| async {
            let Some(user_id) = ctx.user_id.clone() else {
                return Err(GraphqlUserError::from("Authentication required").into());
            };

            let Some(id) = NodeID::from_id(&id) else {
                return Ok(None);
            };

            match id {
                NodeID::FigureRecord(id) => {
                    let record = sqlx::query_as!(
                        FigureRecordModel,
                        r#"
                        SELECT
                            id,
                            user_id,
                            character,
                            figure,
                            created_at,
                            stroke_count
                        FROM
                            figure_records
                        WHERE
                            id = $1
                            AND user_id = $2
                    "#,
                        id.to_string(),
                        user_id,
                    )
                    .fetch_optional(&ctx.pool)
                    .await
                    .context("fetch figure_records")?;
                    let record = record
                        .map(|row| row.into_entity())
                        .transpose()
                        .context("FigureRecordModel::into_entity")?;
                    Ok(record
                        .as_ref()
                        .map(FigureRecord::from_entity)
                        .map(NodeValue::FigureRecord))
                }
                NodeID::CharacterConfig(id) => {
                    let character = sqlx::query_as!(
                        CharacterConfigModel,
                        r#"
                        SELECT
                            id,
                            user_id,
                            character,
                            stroke_count,
                            created_at,
                            updated_at,
                            version
                        FROM
                            character_configs
                        WHERE
                            id = $1
                            AND user_id = $2
                    "#,
                        id.to_string(),
                        user_id,
                    )
                    .fetch_optional(&ctx.pool)
                    .await
                    .context("fetch character_configs")?;
                    let character = character
                        .map(|row| row.into_entity())
                        .transpose()
                        .context("fetch character_configs")?;
                    Ok(character
                        .as_ref()
                        .map(CharacterConfig::from_entity)
                        .map(NodeValue::CharacterConfig))
                }
                NodeID::Character(character) => Ok(Some(NodeValue::Character(
                    Character::from_entity(character),
                ))),
            }
        })
        .await
    }

    async fn characters(values: Vec<String>) -> FieldResult<Vec<Character>> {
        handler(|| async {
            values
                .into_iter()
                .map(|value| {
                    Ok(Character {
                        entity: entities::character::Character::try_from(value.as_str())
                            .map_err(|e| GraphqlUserError::from(anyhow::Error::new(e)))?,
                    })
                })
                .collect::<anyhow::Result<Vec<Character>>>()
        })
        .await
    }

    async fn character_configs(
        ctx: &AppCtx,
        ids: Option<Vec<ID>>,
        first: Option<i32>,
        after: Option<String>,
        last: Option<i32>,
        before: Option<String>,
    ) -> FieldResult<CharacterConfigConnection> {
        handler(|| async {
            let Some(user_id) = ctx.user_id.clone() else {
                return Err(GraphqlUserError::from("Authentication required").into());
            };

            let ids = ids
                .map(|ids| -> anyhow::Result<Vec<Ulid>> {
                    ids.into_iter()
                        .map(|id| -> anyhow::Result<Ulid> {
                            Ulid::from_str(id.to_string().as_str())
                                .map_err(|_| GraphqlUserError::from("invalid ids").into())
                        })
                        .collect::<anyhow::Result<Vec<_>>>()
                })
                .transpose()?;

            let limit = Limit::encode(first, last)?;

            let after_id = after
                .map(|after| -> anyhow::Result<Ulid> {
                    let Some(NodeID::FigureRecord(id)) = NodeID::from_id(&ID::new(after)) else {
                    return Err(GraphqlUserError::from("after must be a valid cursor").into())
                };

                    Ok(id)
                })
                .transpose()?;

            let before_id = before
                .map(|before| -> anyhow::Result<Ulid> {
                    let Some(NodeID::FigureRecord(id)) = NodeID::from_id(&ID::new(before)) else {
                    return Err(GraphqlUserError::from("before must be a valid cursor").into());
                };

                    Ok(id)
                })
                .transpose()?;

            let ids = ids.map(|ids| ids.into_iter().map(|id| id.to_string()).collect::<Vec<_>>());

            let result = sqlx::query_as!(
                CharacterConfigModel,
                r#"
                SELECT
                    id,
                    user_id,
                    character,
                    stroke_count,
                    created_at,
                    updated_at,
                    version
                FROM
                    character_configs
                WHERE
                    user_id = $1
                    AND
                    ($2::VARCHAR(64)[] IS NULL OR id = Any($2))
                    AND
                    ($3::VARCHAR(64) IS NULL OR id < $3)
                    AND
                    ($4::VARCHAR(64) IS NULL OR id > $4)
                ORDER BY
                    CASE WHEN $5 = 0 THEN id END DESC,
                    CASE WHEN $5 = 1 THEN id END ASC
                LIMIT $6
            "#,
                &user_id,
                ids.as_ref().map(|ids| ids.as_slice()),
                after_id.map(|id| id.to_string()),
                before_id.map(|id| id.to_string()),
                (limit.kind == LimitKind::Last) as i32,
                limit.value as i64 + 1,
            )
            .fetch_all(&ctx.pool)
            .await
            .context("fetch character_configs")?;

            let mut characters = result
                .into_iter()
                .map(|row| row.into_entity())
                .collect::<anyhow::Result<Vec<_>>>()
                .context("convert CharacterConfig")?;

            let has_extra = characters.len() > limit.value as usize;

            characters.truncate(limit.value as usize);

            if limit.kind == LimitKind::Last {
                characters.reverse();
            }

            let records = characters
                .into_iter()
                .map(|character| CharacterConfig::from_entity(&character))
                .collect::<Vec<_>>();

            Ok(CharacterConfigConnection {
                page_info: PageInfo {
                    has_next_page: has_extra && limit.kind == LimitKind::First,
                    has_previous_page: has_extra && limit.kind == LimitKind::Last,
                    start_cursor: records.first().map(|record| record.id.to_string()),
                    end_cursor: records.last().map(|record| record.id.to_string()),
                },
                edges: records
                    .into_iter()
                    .map(|character| CharacterConfigEdge {
                        cursor: character.id.to_string(),
                        node: character,
                    })
                    .collect(),
            })
        })
        .await
    }
}
#[derive(Clone, Debug)]
pub struct MutationRoot;

#[juniper::graphql_object(Context = AppCtx, name = "Mutation")]
impl MutationRoot {
    async fn create_figure_record(
        ctx: &AppCtx,
        input: CreateFigureRecordInput,
    ) -> FieldResult<FigureRecord> {
        handler(|| async {
            let Some(user_id) = ctx.user_id.clone() else {
                return Err(GraphqlUserError::from("Authentication required").into());
            } ;

                let character = entities::character::Character::try_from(input.character.as_str())
                    .map_err(|err| GraphqlUserError::from(anyhow::Error::new(err)))?;

                let Some(figure) = entities::figure::Figure::from_json(&input.figure) else {
                return Err(GraphqlUserError::from("figure must be valid json").into());
            };

                let record = entities::figure_record::FigureRecord {
                    id: Ulid::from_datetime(ctx.now),
                    user_id,
                    character: Into::<entities::character::Character>::into(character),
                    figure,
                    created_at: ctx.now,
                };

                sqlx::query!(
                r#"
                    INSERT INTO figure_records (id, user_id, character, figure, created_at, stroke_count)
                    VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                record.id.to_string(),
                record.user_id,
                Into::<String>::into(record.character.clone()),
                record.figure.to_json_ast(),
                record.created_at,
                record.figure.strokes.len() as i32,
            )
            .execute(&ctx.pool)
            .await
            .context("fetch figure_records")?;

            Ok(FigureRecord::from_entity(&record))
        }).await
    }

    async fn create_character_config(
        ctx: &AppCtx,
        input: CreateCharacterConfigInput,
    ) -> FieldResult<CharacterConfig> {
        handler(|| async {
            let Some(user_id) = ctx.user_id.clone() else {
                return Err(GraphqlUserError::from("Authentication required").into());
            } ;

            let character =
                entities::character::Character::try_from(input.character.as_str())
                    .map_err(|err| GraphqlUserError::from(anyhow::Error::new(err)))?;

            let stroke_count = input.stroke_count.try_into().map_err(|_| {
                GraphqlUserError::from("stroke_count must be an non negative integer")
            })?;

            let character = entities::character_config::CharacterConfig {
                id: Ulid::from_datetime(ctx.now),
                user_id,
                character,
                created_at: ctx.now,
                updated_at: ctx.now,
                stroke_count,
            };

            // check exist
            let exists = sqlx::query!(
                r#"
                SELECT
                    id
                FROM
                    character_configs
                WHERE
                    user_id = $1
                    AND
                    character = $2
            "#,
                character.user_id,
                Into::<String>::into(character.character.clone()),
            )
            .fetch_optional(&ctx.pool)
            .await
            .context("check character_config exist")?
            .is_some();

            if exists {
                return Err(GraphqlUserError::from("character already exists").into());
            }

            sqlx::query!(
            r#"
                INSERT INTO character_configs (id, user_id, character, created_at, updated_at, stroke_count, version)
                VALUES ($1, $2, $3, $4, $5, $6, 1)
            "#,
            character.id.to_string(),
            character.user_id,
            Into::<String>::into(character.character.clone()),
            character.created_at,
            character.updated_at,
            character.stroke_count as i32,
        )
        .execute(&ctx.pool)
        .await
        .context("insert character_configs")?;

            Ok(CharacterConfig::from_entity(&character))
        }).await
    }

    async fn update_character_config(
        ctx: &AppCtx,
        input: UpdateCharacterConfigInput,
    ) -> FieldResult<CharacterConfig> {
        handler(|| async {
            let Some(user_id) = ctx.user_id.clone() else {
                return Err(GraphqlUserError::from("Authentication required").into());
            };

            let Some(NodeID::CharacterConfig(id)) = NodeID::from_id(&input.id) else {
                return Err(GraphqlUserError::from("Not found").into());
            };

            let model = sqlx::query_as!(
                CharacterConfigModel,
                r#"
                SELECT
                    id,
                    user_id,
                    character,
                    created_at,
                    updated_at,
                    stroke_count,
                    version
                FROM
                    character_configs
                WHERE
                    id = $1
                    AND
                    user_id = $2
            "#,
                id.to_string(),
                user_id,
            )
            .fetch_optional(&ctx.pool)
            .await
            .context("fetch character_configs")?;

            let Some(model) = model else {
                return Err(GraphqlUserError::from("Not found").into());
            };

            let stroke_count = match input.stroke_count {
                Some(stroke_count) => stroke_count.try_into().map_err(|_| {
                    GraphqlUserError::from("stroke_count must be an non negative integer")
                })?,
                None => model.stroke_count as usize,
            };

            let result = sqlx::query!(
                r#"
                UPDATE character_configs
                SET
                    updated_at = $1,
                    stroke_count = $2,
                    version = $3
                WHERE
                    id = $4
                    AND
                    user_id = $5
                    AND
                    version = $6
            "#,
                ctx.now,
                stroke_count as i32,
                model.version + 1,
                id.to_string(),
                user_id,
                model.version,
            )
            .execute(&ctx.pool)
            .await
            .context("update character_config")?;

            if result.rows_affected() == 0 {
                return Err(anyhow!("conflict").into());
            }

            let entity = model.into_entity().context("convert character model")?;
            let entity = entities::character_config::CharacterConfig {
                updated_at: ctx.now,
                stroke_count,
                ..entity
            };

            Ok(CharacterConfig::from_entity(&entity))
        })
        .await
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<AppCtx>>;

pub fn create_schema() -> Schema {
    Schema::new(
        QueryRoot {},
        MutationRoot {},
        EmptySubscription::<AppCtx>::new(),
    )
}
