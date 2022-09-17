use std::collections::HashSet;

use chrono::{DateTime, Utc};
use derive_more::From;
use guard::guard;
use juniper::FieldResult;
use juniper::{
    graphql_interface, EmptySubscription, GraphQLInputObject, GraphQLObject, RootNode, ID,
};
use ulid::Ulid;

use crate::entities;

use crate::graphql::scalars::{FigureScalar, UlidScalar};
use anyhow::{anyhow, Context};

mod common;
pub use common::*;
mod app_ctx;
pub use app_ctx::*;
mod loaders;

use self::scalars::CharacterValueScalar;
use crate::commands::{character_configs_command, figure_records_command};

mod scalars;
use crate::queries::{
    CharacterConfigByCharacterLoaderParams, CharacterConfigByIdLoaderParams,
    CharacterConfigsLoaderParams, FigureRecordByIdLoaderParams,
    FigureRecordsByCharacterLoaderParams,
};
use crate::values::LimitKind;

/*
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
struct FigureRecord(entities::FigureRecord);

#[juniper::graphql_object(Context = AppCtx, impl = NodeValue)]
impl FigureRecord {
    fn id(&self) -> ID {
        self.node_id()
    }

    fn figure_record_id(&self) -> UlidScalar {
        UlidScalar(self.0.id)
    }

    fn character(&self) -> Character {
        Character::from(self.0.character.clone())
    }

    fn figure(&self) -> FigureScalar {
        FigureScalar(self.0.figure.clone())
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
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
    character: CharacterValueScalar,
    figure: FigureScalar,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct CreateFigureRecordPayload {
    figure_record: Option<FigureRecord>,
    errors: Option<Vec<GraphqlErrorType>>,
}

#[derive(Clone, Debug, From)]
struct CharacterConfig(entities::CharacterConfig);

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

    fn character_config_id(&self) -> UlidScalar {
        UlidScalar(self.0.id)
    }

    fn character(&self) -> Character {
        Character::from(self.0.character.clone())
    }

    fn stroke_count(&self) -> i32 {
        self.0.stroke_count as i32
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.0.updated_at
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
    character: CharacterValueScalar,
    stroke_count: i32,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct CreateCharacterConfigPayload {
    character_config: Option<CharacterConfig>,
    errors: Option<Vec<GraphqlErrorType>>,
}

#[derive(GraphQLInputObject, Clone, Debug)]
struct UpdateCharacterConfigInput {
    id: UlidScalar,
    stroke_count: Option<i32>,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct UpdateCharacterConfigPayload {
    character_config: Option<CharacterConfig>,
    errors: Option<Vec<GraphqlErrorType>>,
}

#[derive(Clone, Debug, From)]
struct Character(entities::Character);

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

    fn value(&self) -> CharacterValueScalar {
        CharacterValueScalar(self.0.clone())
    }

    async fn character_config(
        &self,
        ctx: &mut AppCtx,
    ) -> Result<Option<CharacterConfig>, ApiError> {
        guard!(let Some(user_id) = ctx.user_id.clone() else {
            return Err(GraphqlUserError::from("Authentication required").into());
        });

        let character_config = ctx
            .loaders
            .character_config_by_character_loader
            .load(
                CharacterConfigByCharacterLoaderParams { user_id },
                self.0.clone(),
            )
            .await
            .context("load character_config")??;

        Ok(character_config.map(CharacterConfig::from))
    }

    async fn figure_records(
        &self,
        ctx: &AppCtx,
        ids: Option<Vec<UlidScalar>>,
        first: Option<i32>,
        after: Option<String>,
        last: Option<i32>,
        before: Option<String>,
    ) -> Result<FigureRecordConnection, ApiError> {
        // TODO: N+1
        guard!(let Some(user_id) = ctx.user_id.clone() else {
            return Err(GraphqlUserError::from("Authentication required").into());
        });

        let ids = ids.map(|ids| ids.into_iter().map(|id| id.0).collect::<Vec<_>>());

        let limit = encode_limit(first, last)?;

        let after_id = after
            .map(|after| -> anyhow::Result<Ulid> {
                guard!(let Some(NodeID::FigureRecord(id)) = NodeID::from_id(&ID::new(after)) else {
                    return Err(GraphqlUserError::from("after must be a valid cursor").into())
                });

                Ok(id)
            })
            .transpose()?;

        let before_id = before
            .map(|before| -> anyhow::Result<Ulid> {
                guard!(let Some(NodeID::FigureRecord(id)) = NodeID::from_id(&ID::new(before)) else {
                    return Err(GraphqlUserError::from("before must be a valid cursor").into());
                });
                Ok(id)
            })
            .transpose()?;

        let (records, has_extra) = ctx
            .loaders
            .figure_records_by_character_loader
            .load(
                FigureRecordsByCharacterLoaderParams {
                    user_id,
                    ids,
                    after_id,
                    before_id,
                    limit: limit.clone(),
                },
                self.0.clone(),
            )
            .await
            .context("load character_config")??;

        let records = records
            .into_iter()
            .map(FigureRecord::from)
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
    }
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct LoginUser {
    user_id: String,
}

#[derive(Clone, Debug)]
pub struct QueryRoot;
#[juniper::graphql_object(Context = AppCtx, name = "Query")]
impl QueryRoot {
    fn query() -> QueryRoot {
        QueryRoot
    }

    fn login_user(ctx: &AppCtx) -> Option<LoginUser> {
        ctx.user_id.clone().map(|user_id| LoginUser { user_id })
    }

    async fn node(ctx: &AppCtx, id: ID) -> Result<Option<NodeValue>, ApiError> {
        guard!(let Some(user_id) = ctx.user_id.clone() else {
            return Err(GraphqlUserError::from("Authentication required").into());
        });

        guard!(let Some(id) = NodeID::from_id(&id) else {
            return Ok(None);
        });

        match id {
            NodeID::FigureRecord(id) => {
                let figure_record = ctx
                    .loaders
                    .figure_record_by_id_loader
                    .load(FigureRecordByIdLoaderParams { user_id }, id)
                    .await
                    .context("load FigureRecord")??;
                Ok(figure_record
                    .map(FigureRecord::from)
                    .map(NodeValue::FigureRecord))
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
    }

    async fn characters(values: Vec<CharacterValueScalar>) -> FieldResult<Vec<Character>> {
        let mut entities = values
            .into_iter()
            .map(|value| value.0)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        entities.sort();

        Ok(entities
            .into_iter()
            .map(Character::from)
            .collect::<Vec<_>>())
    }

    async fn character_configs(
        ctx: &AppCtx,
        ids: Option<Vec<UlidScalar>>,
        first: Option<i32>,
        after: Option<String>,
        last: Option<i32>,
        before: Option<String>,
    ) -> Result<CharacterConfigConnection, ApiError> {
        guard!(let Some(user_id) = ctx.user_id.clone() else {
            return Err(GraphqlUserError::from("Authentication required").into());
        });

        let ids = ids.map(|ids| ids.into_iter().map(|id| id.0).collect::<Vec<_>>());

        let limit = encode_limit(first, last)?;

        let after_id = after
                .map(|after| -> anyhow::Result<Ulid> {
                    guard!(let Some(NodeID::CharacterConfig(id)) = NodeID::from_id(&ID::new(after)) else {
                    return Err(GraphqlUserError::from("after must be a valid cursor").into())
                });

                    Ok(id)
                })
                .transpose()?;

        let before_id = before
                .map(|before| -> anyhow::Result<Ulid> {
                    guard!(let Some(NodeID::CharacterConfig(id)) = NodeID::from_id(&ID::new(before)) else {
                    return Err(GraphqlUserError::from("before must be a valid cursor").into());
                });

                    Ok(id)
                })
                .transpose()?;

        let (character_configs, has_extra) = ctx
            .loaders
            .character_configs_loader
            .load(
                CharacterConfigsLoaderParams {
                    user_id,
                    ids,
                    after_id,
                    before_id,
                    limit: limit.clone(),
                },
                (),
            )
            .await
            .context("load character_config")??;

        let records = character_configs
            .into_iter()
            .map(CharacterConfig::from)
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
    }
}
#[derive(Clone, Debug)]
pub struct MutationRoot;

#[juniper::graphql_object(Context = AppCtx, name = "Mutation")]
impl MutationRoot {
    async fn create_figure_record(
        ctx: &AppCtx,
        input: CreateFigureRecordInput,
    ) -> Result<CreateFigureRecordPayload, ApiError> {
        guard!(let Some(user_id) = ctx.user_id.clone() else {
                return Err(GraphqlUserError::from("Authentication required").into());
            } );

        let record = figure_records_command::create(
            &ctx.pool,
            user_id,
            ctx.now,
            input.character.0,
            input.figure.0,
        )
        .await?;

        Ok(CreateFigureRecordPayload {
            figure_record: Some(FigureRecord::from(record)),
            errors: None,
        })
    }

    async fn create_character_config(
        ctx: &AppCtx,
        input: CreateCharacterConfigInput,
    ) -> Result<CreateCharacterConfigPayload, ApiError> {
        guard!(let Some(user_id) = ctx.user_id.clone() else {
            return Err(GraphqlUserError::from("Authentication required").into());
        });

        guard!(let Ok(stroke_count) = input.stroke_count.try_into() else {
            return Ok(CreateCharacterConfigPayload {
                character_config: None,
                errors: Some(vec![GraphqlErrorType {
                    message: "stroke_count must be an non negative integer".to_string(),
                }])
            });
        });

        let character_config = match character_configs_command::create(
            &ctx.pool,
            user_id,
            ctx.now,
            input.character.0,
            stroke_count,
        )
        .await?
        {
            Ok(character_config) => character_config,
            Err(e) => {
                return Ok(CreateCharacterConfigPayload {
                    character_config: None,
                    errors: Some(vec![GraphqlErrorType {
                        message: e.to_string(),
                    }]),
                });
            }
        };

        Ok(CreateCharacterConfigPayload {
            character_config: Some(CharacterConfig::from(character_config)),
            errors: None,
        })
    }

    async fn update_character_config(
        ctx: &AppCtx,
        input: UpdateCharacterConfigInput,
    ) -> Result<UpdateCharacterConfigPayload, ApiError> {
        let mut trx = ctx.pool.begin().await?;
        guard!(let Some(user_id) = ctx.user_id.clone() else {
            return Err(GraphqlUserError::from("Authentication required").into());
        });

        let id = input.id.0;

        let character_config = ctx
            .loaders
            .character_config_by_id_loader
            .load(
                CharacterConfigByIdLoaderParams {
                    user_id: user_id.clone(),
                },
                id,
            )
            .await
            .context("load character_config")??;

        guard!(let Some(mut character_config) = character_config else {
            return Err(GraphqlUserError::from("Not found").into());
        });

        let prev_version = character_config.version;

        character_config.version += 1;
        character_config.updated_at = ctx.now;
        if let Some(stroke_count) = input.stroke_count {
            if let Ok(stroke_count) = stroke_count.try_into() {
                character_config.stroke_count = stroke_count;
            } else {
                return Ok(UpdateCharacterConfigPayload {
                    character_config: None,
                    errors: Some(vec![GraphqlErrorType {
                        message: "stroke_count must be an non negative integer".to_string(),
                    }]),
                });
            }
        }

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
            &character_config.updated_at,
            character_config.stroke_count as i32,
            character_config.version,
            id.to_string(),
            &user_id,
            prev_version,
        )
        .execute(&mut trx)
        .await
        .context("update character_config")?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("conflict").into());
        }

        let result = Ok(UpdateCharacterConfigPayload {
            character_config: Some(CharacterConfig::from(character_config)),
            errors: None,
        });
        trx.commit().await?;
        result
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
