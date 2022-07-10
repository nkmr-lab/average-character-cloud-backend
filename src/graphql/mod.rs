use std::collections::HashSet;

use std::str::FromStr;

use chrono::{DateTime, Utc};
use derive_more::From;
use juniper::FieldResult;
use juniper::{
    graphql_interface, EmptySubscription, GraphQLInputObject, GraphQLObject, RootNode, ID,
};

use ulid::Ulid;

use crate::entities;
use anyhow::{anyhow, ensure, Context};

pub mod dataloader_with_params;

mod common;
pub use common::*;
mod app_ctx;
pub use app_ctx::*;
mod loaders;
use character_config_query::{
    CharacterConfigByCharacterLoaderParams, CharacterConfigByIdLoaderParams, CharacterConfigModel,
};

mod character_config_query;

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

/**
 * replayの仕様に従うこと
 *   https://relay.dev/docs/guides/graphql-server-specification/
 *   https://relay.dev/graphql/connections.htm
*/

#[graphql_interface(for = [FigureRecord, CharacterConfig, Character], context = AppCtx)]
trait Node {
    #[graphql(name = "id")]
    fn node_id(&self) -> ID;
}

#[derive(Clone, Debug, From)]
struct FigureRecord(entities::figure_record::FigureRecord);

#[juniper::graphql_object(Context = AppCtx, impl = NodeValue)]
impl FigureRecord {
    fn id(&self) -> ID {
        self.node_id()
    }

    fn figure_record_id(&self) -> String {
        self.0.id.to_string()
    }

    fn character(&self) -> Character {
        Character::from(self.0.character.clone())
    }

    fn figure(&self) -> String {
        self.0.figure.to_json()
    }

    fn created_at(&self) -> String {
        self.0.created_at.to_rfc3339()
    }
}

#[graphql_interface]
impl Node for FigureRecord {
    fn node_id(&self) -> ID {
        NodeID::FigureRecord(self.0.id).to_id()
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

#[derive(Clone, Debug, From)]
struct CharacterConfig(entities::character_config::CharacterConfig);

#[graphql_interface]
impl Node for CharacterConfig {
    fn node_id(&self) -> ID {
        NodeID::CharacterConfig(self.0.id).to_id()
    }
}

#[juniper::graphql_object(Context = AppCtx, impl = NodeValue)]
impl CharacterConfig {
    fn id(&self) -> ID {
        self.node_id()
    }

    fn character_config_id(&self) -> String {
        self.0.id.to_string()
    }

    fn character(&self) -> Character {
        Character::from(self.0.character.clone())
    }

    fn stroke_count(&self) -> i32 {
        self.0.stroke_count as i32
    }

    fn created_at(&self) -> String {
        self.0.created_at.to_rfc3339()
    }

    fn updated_at(&self) -> String {
        self.0.updated_at.to_rfc3339()
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

#[derive(Clone, Debug, From)]
struct Character(entities::character::Character);

#[graphql_interface]
impl Node for Character {
    fn node_id(&self) -> ID {
        NodeID::Character(self.0.clone()).to_id()
    }
}

#[juniper::graphql_object(Context = AppCtx, impl = NodeValue)]
impl Character {
    fn id(&self) -> ID {
        self.node_id()
    }

    fn value(&self) -> String {
        String::from(self.0.clone())
    }

    async fn character_config(&self, ctx: &mut AppCtx) -> FieldResult<Option<CharacterConfig>> {
        handler(|| async {
            let Some(user_id) = ctx.user_id.clone() else {
                return Err(GraphqlUserError::from("Authentication required").into());
            };

            let character_config = ctx
                .loaders
                .character_config_by_character_loader
                .load(
                    CharacterConfigByCharacterLoaderParams { user_id },
                    self.0.clone(),
                )
                .await
                .context("load character_config")??;

            Ok(character_config.map(|entity| CharacterConfig::from(entity)))
        })
        .await
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

            let characters = vec![self.0.clone()];

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
                .map(|record| FigureRecord::from(record))
                .collect::<Vec<_>>();

            Ok(FigureRecordConnection {
                page_info: PageInfo {
                    has_next_page: has_extra && limit.kind == LimitKind::First,
                    has_previous_page: has_extra && limit.kind == LimitKind::Last,
                    start_cursor: records.first().map(|record| record.node_id().to_string()),
                    end_cursor: records.last().map(|record| record.node_id().to_string()),
                },
                edges: records
                    .into_iter()
                    .map(|record| FigureRecordEdge {
                        cursor: record.node_id().to_string(),
                        node: record,
                    })
                    .collect(),
            })
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
                    Ok(record.map(FigureRecord::from).map(NodeValue::FigureRecord))
                }
                NodeID::CharacterConfig(id) => {
                    let character_config = ctx
                        .loaders
                        .character_config_by_id_loader
                        .load(CharacterConfigByIdLoaderParams { user_id }, id)
                        .await
                        .context("load character_config")??;

                    Ok(character_config
                        .map(CharacterConfig::from)
                        .map(NodeValue::CharacterConfig))
                }
                NodeID::Character(character) => {
                    Ok(Some(NodeValue::Character(Character::from(character))))
                }
            }
        })
        .await
    }

    async fn characters(values: Vec<String>) -> FieldResult<Vec<Character>> {
        handler(|| async {
            let entities = values
                .into_iter()
                .map(|value| {
                    entities::character::Character::try_from(value.as_str())
                        .map_err(|e| GraphqlUserError::from(anyhow::Error::new(e)).into())
                })
                .collect::<anyhow::Result<Vec<_>>>()?;

            let mut entities = entities
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();

            entities.sort();

            Ok(entities
                .into_iter()
                .map(Character::from)
                .collect::<Vec<_>>())
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
                .map(|character| CharacterConfig::from(character))
                .collect::<Vec<_>>();

            Ok(CharacterConfigConnection {
                page_info: PageInfo {
                    has_next_page: has_extra && limit.kind == LimitKind::First,
                    has_previous_page: has_extra && limit.kind == LimitKind::Last,
                    start_cursor: records.first().map(|record| record.node_id().to_string()),
                    end_cursor: records.last().map(|record| record.node_id().to_string()),
                },
                edges: records
                    .into_iter()
                    .map(|character| CharacterConfigEdge {
                        cursor: character.node_id().to_string(),
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
            let mut trx = ctx.pool.begin().await?;

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
                    character,
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
                String::from(record.character.clone()),
                record.figure.to_json_ast(),
                record.created_at,
                record.figure.strokes.len() as i32,
            )
            .execute(&mut trx)
            .await
            .context("fetch figure_records")?;

            let result = Ok(FigureRecord::from(record));
            trx.commit().await?;
            result
        }).await
    }

    async fn create_character_config(
        ctx: &AppCtx,
        input: CreateCharacterConfigInput,
    ) -> FieldResult<CharacterConfig> {
        handler(|| async {
            let mut trx = ctx.pool.begin().await?;

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
                version: 1,
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
                String::from(character.character.clone()),
            )
            .fetch_optional(&mut trx)
            .await
            .context("check character_config exist")?
            .is_some();

            if exists {
                return Err(GraphqlUserError::from("character already exists").into());
            }

            sqlx::query!(
                r#"
                    INSERT INTO character_configs (id, user_id, character, created_at, updated_at, stroke_count, version)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                character.id.to_string(),
                character.user_id,
                String::from(character.character.clone()),
                character.created_at,
                character.updated_at,
                character.stroke_count as i32,
                character.version,
            )
            .execute(&mut trx)
            .await
            .context("insert character_configs")?;

            let result = Ok(CharacterConfig::from(character));
            trx.commit().await?;
            result
        }).await
    }

    async fn update_character_config(
        ctx: &AppCtx,
        input: UpdateCharacterConfigInput,
    ) -> FieldResult<CharacterConfig> {
        handler(|| async {
            let mut trx = ctx.pool.begin().await?;
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
            .fetch_optional(&mut trx)
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
            .execute(&mut trx)
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

            let result = Ok(CharacterConfig::from(entity));
            trx.commit().await?;
            result
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
