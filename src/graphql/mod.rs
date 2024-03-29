use std::collections::HashSet;

use chrono::{DateTime, Utc};
use derive_more::From;
use guard::guard;
use juniper::FieldResult;
use juniper::{
    graphql_interface, EmptySubscription, GraphQLInputObject, GraphQLObject, RootNode, ID,
};
use ulid::Ulid;

use crate::adapters::UserConfigsRepositoryImpl;
use crate::entities;

use crate::graphql::scalars::{FigureScalar, UlidScalar};
use anyhow::Context;

mod common;
pub use common::*;
mod app_ctx;
pub use app_ctx::*;
mod loaders;
use self::scalars::CharacterValueScalar;
use crate::commands::{character_configs_command, figure_records_command};
use crate::ports::UserConfigsRepository;

mod scalars;
use crate::queries;
use crate::queries::{
    CharacterConfigByCharacterLoaderParams, CharacterConfigSeedByCharacterLoaderParams,
    CharacterConfigSeedsLoaderParams, CharacterConfigsLoaderParams, FigureRecordByIdLoaderParams,
    FigureRecordsByCharacterLoaderParams,
};

/*
 * replayの仕様に従うこと
 *   https://relay.dev/docs/guides/graphql-server-specification/
 *   https://relay.dev/graphql/connections.htm
*/

#[graphql_interface(for = [FigureRecord, CharacterConfig, Character, UserConfig, CharacterConfigSeed], context = AppCtx)]
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
        UlidScalar(Ulid::from(self.0.id))
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
        NodeId::FigureRecord(self.0.id).to_id()
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
        NodeId::CharacterConfig(self.0.user_id.clone(), self.0.character.clone()).to_id()
    }
}

#[juniper::graphql_object(Context = AppCtx, impl = NodeValue)]
impl CharacterConfig {
    fn id(&self) -> ID {
        self.node_id()
    }

    fn character(&self) -> Character {
        Character::from(self.0.character.clone())
    }

    fn stroke_count(&self) -> i32 {
        i32::from(self.0.stroke_count)
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

#[derive(Clone, Debug, From)]
struct CharacterConfigSeed(entities::CharacterConfigSeed);

#[graphql_interface]
impl Node for CharacterConfigSeed {
    fn node_id(&self) -> ID {
        NodeId::CharacterConfigSeed(self.0.character.clone()).to_id()
    }
}

#[juniper::graphql_object(Context = AppCtx, impl = NodeValue)]
impl CharacterConfigSeed {
    fn id(&self) -> ID {
        self.node_id()
    }

    fn character(&self) -> Character {
        Character::from(self.0.character.clone())
    }

    fn stroke_count(&self) -> i32 {
        i32::from(self.0.stroke_count)
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.0.updated_at
    }
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct CharacterConfigSeedEdge {
    cursor: String,
    node: CharacterConfigSeed,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct CharacterConfigSeedConnection {
    page_info: PageInfo,
    edges: Vec<CharacterConfigSeedEdge>,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct CreateCharacterConfigPayload {
    character_config: Option<CharacterConfig>,
    errors: Option<Vec<GraphqlErrorType>>,
}

#[derive(GraphQLInputObject, Clone, Debug)]
struct UpdateCharacterConfigInput {
    character: CharacterValueScalar,
    stroke_count: Option<i32>,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct UpdateCharacterConfigPayload {
    character_config: Option<CharacterConfig>,
    errors: Option<Vec<GraphqlErrorType>>,
}

#[derive(GraphQLInputObject, Clone, Debug)]
struct UpdateFigureRecordInput {
    id: UlidScalar,
    disabled: Option<bool>,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct UpdateFigureRecordPayload {
    figure_record: Option<FigureRecord>,
    errors: Option<Vec<GraphqlErrorType>>,
}

#[derive(GraphQLInputObject, Clone, Debug)]
struct UpdateUserConfigInput {
    allow_sharing_character_configs: Option<bool>,
    allow_sharing_figure_records: Option<bool>,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct UpdateUserConfigPayload {
    user_config: Option<UserConfig>,
    errors: Option<Vec<GraphqlErrorType>>,
}

#[derive(Clone, Debug, From)]
struct Character(entities::Character);

#[graphql_interface]
impl Node for Character {
    fn node_id(&self) -> ID {
        NodeId::Character(self.0.clone()).to_id()
    }
}

#[derive(juniper::GraphQLEnum)]
enum UserType {
    Myself,
    Other,
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
        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

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

    async fn character_config_seed(
        &self,
        ctx: &mut AppCtx,
    ) -> Result<Option<CharacterConfigSeed>, ApiError> {
        let _user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let character_config_seed = ctx
            .loaders
            .character_config_seed_by_character_loader
            .load(
                CharacterConfigSeedByCharacterLoaderParams {},
                self.0.clone(),
            )
            .await
            .context("load character_config_seed")??;

        Ok(character_config_seed.map(CharacterConfigSeed::from))
    }

    async fn figure_records(
        &self,
        ctx: &AppCtx,
        ids: Option<Vec<UlidScalar>>,
        first: Option<i32>,
        after: Option<String>,
        last: Option<i32>,
        before: Option<String>,
        user_type: Option<UserType>,
    ) -> Result<FigureRecordConnection, ApiError> {
        // TODO: N+1
        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let ids = ids.map(|ids| {
            ids.into_iter()
                .map(|id| entities::FigureRecordId::from(id.0))
                .collect::<Vec<_>>()
        });

        let limit = encode_limit(first, last)?;

        let after_id = after
            .map(|after| -> anyhow::Result<entities::FigureRecordId> {
                guard!(let Some(NodeId::FigureRecord(id)) = NodeId::from_id(&ID::new(after)) else {
                    return Err(GraphqlUserError::from("after must be a valid cursor").into())
                });

                Ok(id)
            })
            .transpose()?;

        let before_id = before
            .map(|before| -> anyhow::Result<entities::FigureRecordId> {
                guard!(let Some(NodeId::FigureRecord(id)) = NodeId::from_id(&ID::new(before)) else {
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
                    user_type: user_type.map(|user_type| match user_type {
                        UserType::Myself => queries::UserType::Myself,
                        UserType::Other => queries::UserType::Other,
                    }),
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
                has_next_page: has_extra && limit.kind() == entities::LimitKind::First,
                has_previous_page: has_extra && limit.kind() == entities::LimitKind::Last,
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

#[derive(Clone, Debug, From)]
struct UserConfig(entities::UserConfig);

#[graphql_interface]
impl Node for UserConfig {
    fn node_id(&self) -> ID {
        NodeId::UserConfig(self.0.user_id.clone()).to_id()
    }
}

#[juniper::graphql_object(Context = AppCtx, impl = NodeValue)]
impl UserConfig {
    fn id(&self) -> ID {
        self.node_id()
    }

    fn allow_sharing_character_configs(&self) -> bool {
        self.0.allow_sharing_character_configs
    }

    fn allow_sharing_figure_records(&self) -> bool {
        self.0.allow_sharing_figure_records
    }

    fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}

#[derive(Clone, Debug)]
struct LoginUser {
    user_id: entities::UserId,
}

#[juniper::graphql_object(Context = AppCtx)]
impl LoginUser {
    fn user_id(&self) -> String {
        String::from(self.user_id.clone())
    }
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

    async fn user_config(&self, ctx: &AppCtx) -> Result<UserConfig, ApiError> {
        let mut user_config_repository = UserConfigsRepositoryImpl::new();
        let mut conn = ctx.pool.acquire().await?;

        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        Ok(UserConfig(
            user_config_repository.get(&mut conn, user_id).await?,
        ))
    }

    async fn node(ctx: &AppCtx, id: ID) -> Result<Option<NodeValue>, ApiError> {
        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        guard!(let Some(id) = NodeId::from_id(&id) else {
            return Ok(None);
        });

        match id {
            NodeId::FigureRecord(id) => {
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
            NodeId::CharacterConfig(_, character) => {
                let character_config = ctx
                    .loaders
                    .character_config_by_character_loader
                    .load(
                        CharacterConfigByCharacterLoaderParams { user_id },
                        character,
                    )
                    .await
                    .context("load character_config")??;

                Ok(character_config
                    .map(CharacterConfig::from)
                    .map(NodeValue::CharacterConfig))
            }
            NodeId::Character(character) => {
                Ok(Some(NodeValue::Character(Character::from(character))))
            }
            NodeId::UserConfig(_) => {
                let mut user_config_repository = UserConfigsRepositoryImpl::new();
                let mut conn = ctx.pool.acquire().await?;

                let user_config = user_config_repository.get(&mut conn, user_id).await?;
                Ok(Some(NodeValue::UserConfig(UserConfig(user_config))))
            }
            NodeId::CharacterConfigSeed(character) => {
                let character_config_seed = ctx
                    .loaders
                    .character_config_seed_by_character_loader
                    .load(CharacterConfigSeedByCharacterLoaderParams {}, character)
                    .await
                    .context("load character_config_seed")??;

                Ok(character_config_seed
                    .map(CharacterConfigSeed::from)
                    .map(NodeValue::CharacterConfigSeed))
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
        first: Option<i32>,
        after: Option<String>,
        last: Option<i32>,
        before: Option<String>,
    ) -> Result<CharacterConfigConnection, ApiError> {
        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let limit = encode_limit(first, last)?;

        let after_character = after
                .map(|after| -> anyhow::Result<_> {
                    guard!(let Some(NodeId::CharacterConfig(_, character)) = NodeId::from_id(&ID::new(after)) else {
                    return Err(GraphqlUserError::from("after must be a valid cursor").into())
                });

                    Ok(character)
                })
                .transpose()?;

        let before_character = before
                .map(|before| -> anyhow::Result<_> {
                    guard!(let Some(NodeId::CharacterConfig(_, character)) = NodeId::from_id(&ID::new(before)) else {
                    return Err(GraphqlUserError::from("before must be a valid cursor").into());
                });

                    Ok(character)
                })
                .transpose()?;

        let (character_configs, has_extra) = ctx
            .loaders
            .character_configs_loader
            .load(
                CharacterConfigsLoaderParams {
                    user_id,
                    after_character,
                    before_character,
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
                has_next_page: has_extra && limit.kind() == entities::LimitKind::First,
                has_previous_page: has_extra && limit.kind() == entities::LimitKind::Last,
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

    async fn character_config_seeds(
        ctx: &AppCtx,
        first: Option<i32>,
        after: Option<String>,
        last: Option<i32>,
        before: Option<String>,
        #[graphql(default = false)] include_exist_character_config: bool,
    ) -> Result<CharacterConfigSeedConnection, ApiError> {
        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let limit = encode_limit(first, last)?;

        let after_character = after
                .map(|after| -> anyhow::Result<_> {
                    guard!(let Some(NodeId::CharacterConfigSeed(character)) = NodeId::from_id(&ID::new(after)) else {
                    return Err(GraphqlUserError::from("after must be a valid cursor").into())
                });

                    Ok(character)
                })
                .transpose()?;

        let before_character = before
                .map(|before| -> anyhow::Result<_> {
                    guard!(let Some(NodeId::CharacterConfigSeed(character)) = NodeId::from_id(&ID::new(before)) else {
                    return Err(GraphqlUserError::from("before must be a valid cursor").into());
                });

                    Ok(character)
                })
                .transpose()?;

        let (character_config_seeds, has_extra) = ctx
            .loaders
            .character_config_seeds_loader
            .load(
                CharacterConfigSeedsLoaderParams {
                    user_id,
                    after_character,
                    before_character,
                    limit: limit.clone(),
                    include_exist_character_config,
                },
                (),
            )
            .await
            .context("load character_config_seed")??;

        let records = character_config_seeds
            .into_iter()
            .map(CharacterConfigSeed::from)
            .collect::<Vec<_>>();

        Ok(CharacterConfigSeedConnection {
            page_info: PageInfo {
                has_next_page: has_extra && limit.kind() == entities::LimitKind::First,
                has_previous_page: has_extra && limit.kind() == entities::LimitKind::Last,
                start_cursor: records.first().map(|record| record.node_id().to_string()),
                end_cursor: records.last().map(|record| record.node_id().to_string()),
            },
            edges: records
                .into_iter()
                .map(|character| CharacterConfigSeedEdge {
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
        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

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
        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        guard!(let Ok(stroke_count) = entities::StrokeCount::try_from(input.stroke_count) else {
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
        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let character = input.character.0;

        let character_config = ctx
            .loaders
            .character_config_by_character_loader
            .load(
                CharacterConfigByCharacterLoaderParams {
                    user_id: user_id.clone(),
                },
                character,
            )
            .await
            .context("load character_config")??
            .ok_or_else(|| GraphqlUserError::from("Not found"))?;

        guard!(let Ok(stroke_count) = input
        .stroke_count
        .map(entities::StrokeCount::try_from)
        .transpose() else {
           return Ok(UpdateCharacterConfigPayload {
                character_config: None,
                errors: Some(vec![GraphqlErrorType {
                    message: "stroke_count must be an non negative integer".to_string(),
                }]),
            });
        });

        let character_config =
            character_configs_command::update(&ctx.pool, ctx.now, character_config, stroke_count)
                .await?;

        Ok(UpdateCharacterConfigPayload {
            character_config: Some(CharacterConfig::from(character_config)),
            errors: None,
        })
    }

    async fn update_figure_record(
        ctx: &AppCtx,
        input: UpdateFigureRecordInput,
    ) -> Result<UpdateFigureRecordPayload, ApiError> {
        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let id = entities::FigureRecordId::from(input.id.0);

        let figure_record = ctx
            .loaders
            .figure_record_by_id_loader
            .load(
                FigureRecordByIdLoaderParams {
                    user_id: user_id.clone(),
                },
                id,
            )
            .await
            .context("load figure_record")??
            .ok_or_else(|| GraphqlUserError::from("Not found"))?;

        let figure_record =
            figure_records_command::update(&ctx.pool, figure_record, input.disabled).await?;

        Ok(UpdateFigureRecordPayload {
            figure_record: Some(FigureRecord::from(figure_record)),
            errors: None,
        })
    }

    async fn update_user_config(
        ctx: &AppCtx,
        input: UpdateUserConfigInput,
    ) -> Result<UpdateUserConfigPayload, ApiError> {
        let mut user_config_repository = UserConfigsRepositoryImpl::new();
        let mut conn = ctx.pool.acquire().await?;

        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let mut user_config = user_config_repository
            .get(&mut conn, user_id.clone())
            .await
            .context("load user_config")?;

        if let Some(allow_sharing_character_configs) = input.allow_sharing_character_configs {
            user_config =
                user_config.with_allow_sharing_character_configs(allow_sharing_character_configs);
        }

        if let Some(allow_sharing_figure_records) = input.allow_sharing_figure_records {
            user_config =
                user_config.with_allow_sharing_figure_records(allow_sharing_figure_records);
        }

        let user_config = user_config_repository
            .save(&mut conn, ctx.now, user_config)
            .await?;

        Ok(UpdateUserConfigPayload {
            user_config: Some(UserConfig::from(user_config)),
            errors: None,
        })
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
