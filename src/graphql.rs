use std::str::FromStr;

use chrono::{DateTime, Utc};
use juniper::FieldError;
use juniper::{
    graphql_interface, EmptySubscription, GraphQLInputObject, GraphQLObject, IntoFieldError,
    RootNode, ScalarValue, ID,
};
use sqlx::PgPool;
use ulid::Ulid;

use crate::entities;
use anyhow::{anyhow, ensure, Context};

#[derive(Debug)]
pub struct AppCtx {
    pub pool: PgPool,
    pub user_id: Option<String>,
    pub now: DateTime<Utc>,
}

#[derive(Debug)]
pub enum AppError {
    Other(String),
    Internal(anyhow::Error),
}

impl<S: ScalarValue> IntoFieldError<S> for AppError {
    fn into_field_error(self) -> FieldError<S> {
        match self {
            AppError::Other(msg) => msg.into(),
            AppError::Internal(err) => {
                log::error!("{}", err);
                "Internal error".into()
            }
        }
    }
}

type AppResult<T> = Result<T, AppError>;

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
    fn encode(first: Option<i32>, last: Option<i32>) -> AppResult<Self> {
        let max = 100;

        match (first, last) {
            (Some(first), None) => {
                if first < 0 {
                    Err(AppError::Other(
                        "first must be greater than or equal to 0".to_string(),
                    ))
                } else if first > max {
                    Err(AppError::Other(format!(
                        "first must be less than or equal to {}",
                        max
                    )))
                } else {
                    Ok(Limit {
                        kind: LimitKind::First,
                        value: first,
                    })
                }
            }
            (None, Some(last)) => {
                if last < 0 {
                    Err(AppError::Other(
                        "last must be greater than or equal to 0".to_string(),
                    ))
                } else if last > max {
                    Err(AppError::Other(format!(
                        "last must be less than or equal to {}",
                        max
                    )))
                } else {
                    Ok(Limit {
                        kind: LimitKind::Last,
                        value: last,
                    })
                }
            }
            _ => Err(AppError::Other(
                "Must provide either first or last, not both".to_string(),
            )),
        }
    }
}

impl juniper::Context for AppCtx {}

#[derive(Debug, Clone)]
enum NodeID {
    FigureRecord(Ulid),
    CharacterConfig(Ulid),
}

impl NodeID {
    fn to_id(&self) -> ID {
        match self {
            NodeID::FigureRecord(id) => ID::new(base64::encode(format!("FigureRecord:{}", id))),
            NodeID::CharacterConfig(id) => {
                ID::new(base64::encode(format!("CharacterConfig:{}", id)))
            }
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

        let &[character] = self.character.chars().collect::<Vec<_>>().as_slice() else {
            return Err(anyhow!("character must be one character"));
        };

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

        let &[character] = self.character.chars().collect::<Vec<_>>().as_slice() else {
            return Err(anyhow!("character must be one character"));
        };

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

#[graphql_interface(for = [FigureRecord, CharacterConfig])]
trait Node {
    fn id(&self) -> &ID;
}

#[derive(GraphQLObject, Clone, Debug)]
struct PageInfo {
    has_next_page: bool,
    has_previous_page: bool,
    start_cursor: Option<String>,
    end_cursor: Option<String>,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(impl = NodeValue)]
struct FigureRecord {
    id: ID,
    record_id: String,
    character: String,
    figure: String,
    created_at: String,
}

impl FigureRecord {
    fn from_entity(record: &entities::figure_record::FigureRecord) -> FigureRecord {
        FigureRecord {
            id: NodeID::FigureRecord(record.id).to_id(),
            record_id: record.id.to_string(),
            character: record.character.to_string(),
            figure: record.figure.to_json(),
            created_at: record.created_at.to_rfc3339(),
        }
    }
}

#[graphql_interface]
impl Node for FigureRecord {
    fn id(&self) -> &ID {
        &self.id
    }
}

#[derive(GraphQLObject, Clone, Debug)]
struct FigureRecordEdge {
    cursor: String,
    node: FigureRecord,
}

#[derive(GraphQLObject, Clone, Debug)]
struct FigureRecordConnection {
    page_info: PageInfo,
    edges: Vec<FigureRecordEdge>,
}

#[derive(GraphQLInputObject, Clone, Debug)]
struct NewFigureRecord {
    character: String,
    figure: String,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(impl = NodeValue)]
struct CharacterConfig {
    id: ID,
    character_id: String,
    character: String,
    stroke_count: i32,
    created_at: String,
    updated_at: String,
}

impl CharacterConfig {
    fn from_entity(character: &entities::character_config::CharacterConfig) -> CharacterConfig {
        CharacterConfig {
            id: NodeID::CharacterConfig(character.id).to_id(),
            character_id: character.id.to_string(),
            character: character.character.to_string(),
            stroke_count: character.stroke_count as i32,
            created_at: character.created_at.to_rfc3339(),
            updated_at: character.updated_at.to_rfc3339(),
        }
    }
}

#[graphql_interface]
impl Node for CharacterConfig {
    fn id(&self) -> &ID {
        &self.id
    }
}

#[derive(GraphQLObject, Clone, Debug)]
struct CharacterConfigEdge {
    cursor: String,
    node: CharacterConfig,
}

#[derive(GraphQLObject, Clone, Debug)]
struct CharacterConfigConnection {
    page_info: PageInfo,
    edges: Vec<CharacterConfigEdge>,
}

#[derive(GraphQLInputObject, Clone, Debug)]
struct NewCharacterConfig {
    character: String,
    stroke_count: i32,
}

#[derive(GraphQLInputObject, Clone, Debug)]
struct UpdateCharacterConfig {
    stroke_count: Option<i32>,
}

#[derive(Clone, Debug)]
pub struct QueryRoot;

#[juniper::graphql_object(Context = AppCtx, name = "Query")]
impl QueryRoot {
    fn query() -> QueryRoot {
        QueryRoot
    }

    async fn node(ctx: &AppCtx, id: ID) -> AppResult<Option<NodeValue>> {
        let Some(user_id) = ctx.user_id.clone() else {
            return Err(AppError::Other("Authentication required".to_string()));
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
                .map_err(|err| AppError::Internal(err.into()))?;
                let record = record
                    .map(|row| row.into_entity())
                    .transpose()
                    .map_err(AppError::Internal)?;
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
                .map_err(|err| AppError::Internal(err.into()))?;
                let character = character
                    .map(|row| row.into_entity())
                    .transpose()
                    .map_err(AppError::Internal)?;
                Ok(character
                    .as_ref()
                    .map(CharacterConfig::from_entity)
                    .map(NodeValue::CharacterConfig))
            }
        }
    }

    async fn figure_records(
        ctx: &AppCtx,
        characters: Option<Vec<String>>,
        ids: Option<Vec<ID>>,
        first: Option<i32>,
        after: Option<String>,
        last: Option<i32>,
        before: Option<String>,
    ) -> AppResult<FigureRecordConnection> {
        let Some(user_id) = ctx.user_id.clone() else {
            return Err(AppError::Other("Authentication required".to_string()));
        };

        let characters =
            characters
                .map(|characters| -> AppResult<Vec<char>> {
                    characters.into_iter().map(|character|-> AppResult<char>{
                    let &[character] = character.chars().collect::<Vec<_>>().as_slice() else {
                        return Err(AppError::Other(
                            "character must be one character".to_string(),
                        ));
                    };

                    Ok(character)
                }).collect::<AppResult<Vec<_>>>()
                })
                .transpose()?;

        let ids = ids
            .map(|ids| -> AppResult<Vec<Ulid>> {
                ids.into_iter()
                    .map(|id| -> AppResult<Ulid> {
                        Ulid::from_str(id.to_string().as_str())
                            .map_err(|_| AppError::Other("invalid ids".to_string()))
                    })
                    .collect::<AppResult<Vec<_>>>()
            })
            .transpose()?;

        let limit = Limit::encode(first, last)?;

        let after_id = after
            .map(|after| -> AppResult<Ulid> {
                let Some(NodeID::FigureRecord(id)) = NodeID::from_id(&ID::new(after)) else {
                    return Err(AppError::Other("after must be a valid cursor".to_string()))
                };

                Ok(id)
            })
            .transpose()?;

        let before_id = before
            .map(|before| -> AppResult<Ulid> {
                let Some(NodeID::FigureRecord(id)) = NodeID::from_id(&ID::new(before)) else {
                    return Err(AppError::Other("before must be a valid cursor".to_string()));
                };

                Ok(id)
            })
            .transpose()?;

        let characters =
            characters.map(|cs| cs.into_iter().map(|c| c.to_string()).collect::<Vec<_>>());

        let ids = ids.map(|ids| ids.into_iter().map(|id| id.to_string()).collect::<Vec<_>>());

        let result = sqlx::query_as!(
            FigureRecordModel,
            r#"
                SELECT
                    r.id,
                    r.user_id,
                    r.character,
                    r.figure,
                    r.created_at,
                    r.stroke_count
                FROM
                    figure_records AS r
                JOIN
                    character_configs AS c ON r.character = c.character AND r.user_id = c.user_id
                WHERE
                    r.user_id = $1
                    AND
                    ($2::VARCHAR(8)[] IS NULL OR r.character = Any($2))
                    AND
                    ($3::VARCHAR(64)[] IS NULL OR r.id = Any($3))
                    AND
                    ($4::VARCHAR(64) IS NULL OR r.id < $4)
                    AND
                    ($5::VARCHAR(64) IS NULL OR r.id > $5)
                    AND
                    r.stroke_count = c.stroke_count
                ORDER BY
                    CASE WHEN $6 = 0 THEN r.id END DESC,
                    CASE WHEN $6 = 1 THEN r.id END ASC
                LIMIT $7
            "#,
            &user_id,
            characters.as_ref().map(|cs| cs.as_slice()),
            ids.as_ref().map(|ids| ids.as_slice()),
            after_id.map(|id| id.to_string()),
            before_id.map(|id| id.to_string()),
            (limit.kind == LimitKind::Last) as i32,
            limit.value as i64 + 1,
        )
        .fetch_all(&ctx.pool)
        .await
        .map_err(|err| AppError::Internal(err.into()))?;

        let mut records = result
            .into_iter()
            .map(|row| row.into_entity())
            .collect::<anyhow::Result<Vec<_>>>()
            .map_err(AppError::Internal)?;

        let has_extra = records.len() > limit.value as usize;

        records.truncate(limit.value as usize);

        if limit.kind == LimitKind::Last {
            records.reverse();
        }

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
    }

    async fn character_configs(
        ctx: &AppCtx,
        characters: Option<Vec<String>>,
        ids: Option<Vec<ID>>,
        first: Option<i32>,
        after: Option<String>,
        last: Option<i32>,
        before: Option<String>,
    ) -> AppResult<CharacterConfigConnection> {
        let Some(user_id) = ctx.user_id.clone() else {
            return Err(AppError::Other("Authentication required".to_string()));
        };

        let characters =
            characters
                .map(|characters| -> AppResult<Vec<char>> {
                    characters.into_iter().map(|character|-> AppResult<char>{
                    let &[character] = character.chars().collect::<Vec<_>>().as_slice() else {
                        return Err(AppError::Other(
                            "character must be one character".to_string(),
                        ));
                    };

                    Ok(character)
                }).collect::<AppResult<Vec<_>>>()
                })
                .transpose()?;

        let ids = ids
            .map(|ids| -> AppResult<Vec<Ulid>> {
                ids.into_iter()
                    .map(|id| -> AppResult<Ulid> {
                        Ulid::from_str(id.to_string().as_str())
                            .map_err(|_| AppError::Other("invalid ids".to_string()))
                    })
                    .collect::<AppResult<Vec<_>>>()
            })
            .transpose()?;

        let limit = Limit::encode(first, last)?;

        let after_id = after
            .map(|after| -> AppResult<Ulid> {
                let Some(NodeID::FigureRecord(id)) = NodeID::from_id(&ID::new(after)) else {
                    return Err(AppError::Other("after must be a valid cursor".to_string()))
                };

                Ok(id)
            })
            .transpose()?;

        let before_id = before
            .map(|before| -> AppResult<Ulid> {
                let Some(NodeID::FigureRecord(id)) = NodeID::from_id(&ID::new(before)) else {
                    return Err(AppError::Other("before must be a valid cursor".to_string()));
                };

                Ok(id)
            })
            .transpose()?;

        let characters =
            characters.map(|cs| cs.into_iter().map(|c| c.to_string()).collect::<Vec<_>>());

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
                    ($2::VARCHAR(8)[] IS NULL OR character = Any($2))
                    AND
                    ($3::VARCHAR(64)[] IS NULL OR id = Any($3))
                    AND
                    ($4::VARCHAR(64) IS NULL OR id < $4)
                    AND
                    ($5::VARCHAR(64) IS NULL OR id > $5)
                ORDER BY
                    CASE WHEN $6 = 0 THEN id END DESC,
                    CASE WHEN $6 = 1 THEN id END ASC
                LIMIT $7
            "#,
            &user_id,
            characters.as_ref().map(|cs| cs.as_slice()),
            ids.as_ref().map(|ids| ids.as_slice()),
            after_id.map(|id| id.to_string()),
            before_id.map(|id| id.to_string()),
            (limit.kind == LimitKind::Last) as i32,
            limit.value as i64 + 1,
        )
        .fetch_all(&ctx.pool)
        .await
        .map_err(|err| AppError::Internal(err.into()))?;

        let mut characters = result
            .into_iter()
            .map(|row| row.into_entity())
            .collect::<anyhow::Result<Vec<_>>>()
            .map_err(AppError::Internal)?;

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
    }
}
#[derive(Clone, Debug)]
pub struct MutationRoot;

#[juniper::graphql_object(Context = AppCtx, name = "Mutation")]
impl MutationRoot {
    async fn create_figure_record(
        ctx: &AppCtx,
        new_record: NewFigureRecord,
    ) -> AppResult<FigureRecord> {
        let Some(user_id) = ctx.user_id.clone() else {
            return Err(AppError::Other("Authentication required".to_string()));
        } ;

        let &[character] = new_record.character.chars().collect::<Vec<_>>().as_slice() else {
            return Err(AppError::Other(
                "character must be one character".to_string(),
            ));
        };

        let Some(figure) = entities::figure::Figure::from_json(&new_record.figure) else {
            return Err(AppError::Other("figure must be valid json".to_string()));
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
            record.character.to_string(),
            record.figure.to_json_ast(),
            record.created_at,
            record.figure.strokes.len() as i32,
        )
        .execute(&ctx.pool)
        .await
        .map_err(|err| AppError::Internal(err.into()))?;

        Ok(FigureRecord::from_entity(&record))
    }

    async fn create_character_config(
        ctx: &AppCtx,
        new_character: NewCharacterConfig,
    ) -> AppResult<CharacterConfig> {
        let Some(user_id) = ctx.user_id.clone() else {
            return Err(AppError::Other("Authentication required".to_string()));
        } ;

        let &[character] = new_character.character.chars().collect::<Vec<_>>().as_slice() else {
            return Err(AppError::Other(
                "character must be one character".to_string(),
            ));
        };

        let stroke_count = new_character.stroke_count.try_into().map_err(|_| {
            AppError::Other("stroke_count must be an non negative integer".to_string())
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
            character.character.to_string(),
        )
        .fetch_optional(&ctx.pool)
        .await
        .map_err(|err| AppError::Internal(err.into()))?
        .is_some();

        if exists {
            return Err(AppError::Other("character already exists".to_string()));
        }

        sqlx::query!(
            r#"
                INSERT INTO character_configs (id, user_id, character, created_at, updated_at, stroke_count, version)
                VALUES ($1, $2, $3, $4, $5, $6, 1)
            "#,
            character.id.to_string(),
            character.user_id,
            character.character.to_string(),
            character.created_at,
            character.updated_at,
            character.stroke_count as i32,
        )
        .execute(&ctx.pool)
        .await
        .map_err(|err| AppError::Internal(err.into()))?;

        Ok(CharacterConfig::from_entity(&character))
    }

    async fn update_character(
        ctx: &AppCtx,
        id: ID,
        update_character: UpdateCharacterConfig,
    ) -> AppResult<CharacterConfig> {
        let Some(user_id) = ctx.user_id.clone() else {
            return Err(AppError::Other("Authentication required".to_string()));
        };

        let Some(NodeID::CharacterConfig(id)) = NodeID::from_id(&id) else {
            return Err(AppError::Other("Not found".to_string()));
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
        .map_err(|err| AppError::Internal(err.into()))?;

        let Some(model) = model else {
            return Err(AppError::Other("Not found".to_string()));
        };

        let stroke_count = match update_character.stroke_count {
            Some(stroke_count) => stroke_count.try_into().map_err(|_| {
                AppError::Other("stroke_count must be an non negative integer".to_string())
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
        .map_err(|err| AppError::Internal(err.into()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::Other("Conflict".to_string()));
        }

        let entity = model.into_entity().map_err(AppError::Internal)?;
        let entity = entities::character_config::CharacterConfig {
            updated_at: ctx.now,
            stroke_count,
            ..entity
        };

        Ok(CharacterConfig::from_entity(&entity))
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
