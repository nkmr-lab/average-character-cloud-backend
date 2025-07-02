use std::collections::HashSet;

use chrono::{DateTime, Utc};
use derive_more::From;
use juniper::FieldResult;
use juniper::{
    graphql_interface, EmptySubscription, GraphQLInputObject, GraphQLObject, RootNode, ID,
};
use ulid::Ulid;

use crate::adapters::{
    CharacterConfigsRepositoryImpl, FigureRecordsRepositoryImpl, FilesRepositoryImpl,
    GenerateTemplatesRepositoryImpl, StorageImpl, UserConfigsRepositoryImpl,
};
use crate::{entities, ports};

use crate::graphql::scalars::{FigureScalar, UlidScalar};
use anyhow::Context;

mod common;
pub use common::*;
mod app_ctx;
pub use app_ctx::*;
mod loaders;
use self::scalars::CharacterValueScalar;
use crate::ports::{
    CharacterConfigsRepository, FigureRecordsRepository, FilesRepository,
    GenerateTemplatesRepository, Storage, UserConfigsRepository,
};

mod scalars;
use crate::loaders::{
    CharacterConfigByCharacterLoaderParams, CharacterConfigByIdLoaderParams,
    CharacterConfigLoaderParams, CharacterConfigSeedByCharacterLoaderParams,
    CharacterConfigSeedByIdLoaderParams, CharacterConfigSeedsLoaderParams,
    FigureRecordByIdLoaderParams, FigureRecordsByCharacterConfigIdLoaderParams,
    FileByIdLoaderParams, GenerateTemplateByIdLoaderParams, GenerateTemplatesLoaderParams,
};

/*
 * replayの仕様に従うこと
 *   https://relay.dev/docs/guides/graphql-server-specification/
 *   https://relay.dev/graphql/connections.htm
*/

#[graphql_interface(for = [FigureRecord, CharacterConfig, Character, UserConfig, CharacterConfigSeed, File, GenerateTemplate], context = AppCtx)]
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
struct File(entities::File);

#[juniper::graphql_object(Context = AppCtx, impl = NodeValue)]
impl File {
    fn id(&self) -> ID {
        self.node_id()
    }

    fn file_id(&self) -> UlidScalar {
        UlidScalar(Ulid::from(self.0.id))
    }

    fn mime_type(&self) -> String {
        self.0.mime_type.value().to_string()
    }

    fn size(&self) -> i32 {
        i32::from(self.0.size)
    }

    fn verified(&self) -> bool {
        self.0.verified
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.0.updated_at
    }

    async fn upload_url(&self, ctx: &AppCtx) -> Result<String, ApiError> {
        let mut storage = StorageImpl::new(ctx.config.clone(), ctx.s3_client.clone());
        let url = storage
            .generate_upload_url(&self.0)
            .await
            .map_err(|_| GraphqlUserError::from("Failed to generate upload URL"))?;
        Ok(url)
    }

    async fn download_url(&self, ctx: &AppCtx) -> Result<String, ApiError> {
        let mut storage = StorageImpl::new(ctx.config.clone(), ctx.s3_client.clone());
        let url = storage
            .generate_download_url(&self.0)
            .await
            .map_err(|_| GraphqlUserError::from("Failed to generate download URL"))?;
        Ok(url)
    }
}

#[graphql_interface]
impl Node for File {
    fn node_id(&self) -> ID {
        NodeId::File(self.0.id).to_id()
    }
}

#[derive(GraphQLInputObject, Clone, Debug)]
struct CreateFileInput {
    mime_type: String,
    size: i32,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct CreateFilePayload {
    file: Option<File>,
    errors: Option<Vec<GraphqlErrorType>>,
}

#[derive(GraphQLInputObject, Clone, Debug)]
struct VerifyFileInput {
    id: UlidScalar,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct VerifyFilePayload {
    file: Option<File>,
    errors: Option<Vec<GraphqlErrorType>>,
}

#[derive(Clone, Debug, From)]
struct GenerateTemplate(entities::GenerateTemplate);

#[juniper::graphql_object(Context = AppCtx, impl = NodeValue)]
impl GenerateTemplate {
    fn id(&self) -> ID {
        self.node_id()
    }

    fn generate_template_id(&self) -> UlidScalar {
        UlidScalar(Ulid::from(self.0.id))
    }

    async fn background_image_file(&self, ctx: &AppCtx) -> Result<File, ApiError> {
        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let file = ctx
            .loaders
            .file_by_id_loader
            .load(
                FileByIdLoaderParams {
                    user_id,
                    verified_only: true,
                },
                self.0.background_image_file_id.clone(),
            )
            .await
            .context("load file")??
            .ok_or_else(|| {
                // TODO: internal error
                GraphqlUserError::from("Background image file not found or not verified")
            })?;

        Ok(File::from(file))
    }

    fn font_color(&self) -> i32 {
        i32::from(self.0.font_color)
    }

    fn writing_mode(&self) -> WritingMode {
        WritingMode::from(self.0.writing_mode)
    }

    fn margin_block_start(&self) -> i32 {
        i32::from(self.0.margin_block_start)
    }

    fn margin_inline_start(&self) -> i32 {
        i32::from(self.0.margin_inline_start)
    }

    fn line_spacing(&self) -> i32 {
        i32::from(self.0.line_spacing)
    }

    fn letter_spacing(&self) -> i32 {
        i32::from(self.0.letter_spacing)
    }

    fn font_size(&self) -> i32 {
        i32::from(self.0.font_size)
    }

    fn font_weight(&self) -> i32 {
        i32::from(self.0.font_weight)
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.0.updated_at
    }
}

#[graphql_interface]
impl Node for GenerateTemplate {
    fn node_id(&self) -> ID {
        NodeId::GenerateTemplate(self.0.id).to_id()
    }
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct GenerateTemplateEdge {
    cursor: String,
    node: GenerateTemplate,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct GenerateTemplateConnection {
    page_info: PageInfo,
    edges: Vec<GenerateTemplateEdge>,
}

#[derive(juniper::GraphQLEnum, Clone, Debug)]
enum WritingMode {
    Horizontal,
    Vertical,
}

impl From<entities::WritingMode> for WritingMode {
    fn from(value: entities::WritingMode) -> Self {
        match value {
            entities::WritingMode::Horizontal => WritingMode::Horizontal,
            entities::WritingMode::Vertical => WritingMode::Vertical,
        }
    }
}

impl From<WritingMode> for entities::WritingMode {
    fn from(value: WritingMode) -> Self {
        match value {
            WritingMode::Horizontal => entities::WritingMode::Horizontal,
            WritingMode::Vertical => entities::WritingMode::Vertical,
        }
    }
}

#[derive(GraphQLInputObject, Clone, Debug)]
struct CreateGenerateTemplateInput {
    background_image_file_id: UlidScalar,
    font_color: i32,
    writing_mode: WritingMode,
    margin_block_start: i32,
    margin_inline_start: i32,
    line_spacing: i32,
    letter_spacing: i32,
    font_size: i32,
    font_weight: i32,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct CreateGenerateTemplatePayload {
    generate_template: Option<GenerateTemplate>,
    errors: Option<Vec<GraphqlErrorType>>,
}

#[derive(GraphQLInputObject, Clone, Debug)]
struct UpdateGenerateTemplateInput {
    generate_template_id: UlidScalar,
    background_image_file_id: Option<UlidScalar>,
    font_color: Option<i32>,
    writing_mode: Option<WritingMode>,
    margin_block_start: Option<i32>,
    margin_inline_start: Option<i32>,
    line_spacing: Option<i32>,
    letter_spacing: Option<i32>,
    font_size: Option<i32>,
    font_weight: Option<i32>,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct UpdateGenerateTemplatePayload {
    generate_template: Option<GenerateTemplate>,
    errors: Option<Vec<GraphqlErrorType>>,
}

#[derive(GraphQLInputObject, Clone, Debug)]
struct DeleteGenerateTemplateInput {
    generate_template_id: UlidScalar,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(context = AppCtx)]
struct DeleteGenerateTemplatePayload {
    id: ID,
    errors: Option<Vec<GraphqlErrorType>>,
}

#[derive(Clone, Debug, From)]
struct CharacterConfig(entities::CharacterConfig);

#[graphql_interface]
impl Node for CharacterConfig {
    fn node_id(&self) -> ID {
        NodeId::CharacterConfig(
            self.0.user_id.clone(),
            self.0.character.clone(),
            self.0.stroke_count,
        )
        .to_id()
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

    fn ratio(&self) -> i32 {
        i32::from(self.0.ratio)
    }

    fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }

    fn disabled(&self) -> bool {
        self.0.disabled
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
                let Some(NodeId::FigureRecord(id)) = NodeId::from_id(&ID::new(after)) else {
                    return Err(GraphqlUserError::from("after must be a valid cursor").into());
                };

                Ok(id)
            })
            .transpose()?;

        let before_id = before
            .map(|before| -> anyhow::Result<entities::FigureRecordId> {
                let Some(NodeId::FigureRecord(id)) = NodeId::from_id(&ID::new(before)) else {
                    return Err(GraphqlUserError::from("before must be a valid cursor").into());
                };
                Ok(id)
            })
            .transpose()?;

        let result = ctx
            .loaders
            .figure_records_by_character_config_id_loader
            .load(
                FigureRecordsByCharacterConfigIdLoaderParams {
                    user_id,
                    ids,
                    after_id,
                    before_id,
                    limit: limit.clone(),
                    user_type: user_type.map(|user_type| match user_type {
                        UserType::Myself => ports::UserType::Myself,
                        UserType::Other => ports::UserType::Other,
                    }),
                },
                (self.0.character.clone(), self.0.stroke_count),
            )
            .await
            .context("load character_config")??;

        let records = result
            .values
            .into_iter()
            .map(FigureRecord::from)
            .collect::<Vec<_>>();

        Ok(FigureRecordConnection {
            page_info: PageInfo {
                has_next_page: result.has_next && limit.kind() == entities::LimitKind::First,
                has_previous_page: result.has_next && limit.kind() == entities::LimitKind::Last,
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

#[derive(Clone, Debug, From)]
struct CharacterConfigSeed(entities::CharacterConfigSeed);

#[graphql_interface]
impl Node for CharacterConfigSeed {
    fn node_id(&self) -> ID {
        NodeId::CharacterConfigSeed(self.0.character.clone(), self.0.stroke_count).to_id()
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

    async fn character_config(&self, ctx: &mut AppCtx) -> Result<CharacterConfig, ApiError> {
        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let character_config = ctx
            .loaders
            .character_config_by_id_loader
            .load(
                CharacterConfigByIdLoaderParams { user_id },
                (self.0.character.clone(), self.0.stroke_count),
            )
            .await
            .context("load character_config")??;

        Ok(CharacterConfig::from(character_config))
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

#[derive(GraphQLInputObject, Clone, Debug)]
struct UpdateCharacterConfigInput {
    character: CharacterValueScalar,
    stroke_count: i32,
    ratio: Option<i32>,
    disabled: Option<bool>,
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
    random_level: Option<i32>,
    shared_proportion: Option<i32>,
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

    async fn character_configs(&self, ctx: &mut AppCtx) -> Result<Vec<CharacterConfig>, ApiError> {
        // 通常は1～3程度になるはずだし、どんなに多くてもStrokeCountの最大値である1000を超えることはないのでページネーションはしない
        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let character_configs = ctx
            .loaders
            .character_config_by_character_loader
            .load(
                CharacterConfigByCharacterLoaderParams { user_id },
                self.0.clone(),
            )
            .await
            .context("load character_config")??;

        Ok(character_configs
            .into_iter()
            .map(CharacterConfig::from)
            .collect())
    }

    async fn character_config(
        &self,
        ctx: &mut AppCtx,
        stroke_count: i32,
    ) -> Result<CharacterConfig, ApiError> {
        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let stroke_count = entities::StrokeCount::try_from(stroke_count)
            .map_err(|_| GraphqlUserError::from("stroke_count must be an non negative integer"))?;

        let character_config = ctx
            .loaders
            .character_config_by_id_loader
            .load(
                CharacterConfigByIdLoaderParams { user_id },
                (self.0.clone(), stroke_count),
            )
            .await
            .context("load character_config")??;

        Ok(CharacterConfig::from(character_config))
    }

    async fn character_config_seeds(
        &self,
        ctx: &mut AppCtx,
    ) -> Result<Vec<CharacterConfigSeed>, ApiError> {
        // character_configsと同様にページネーションはしない
        let character_config_seeds = ctx
            .loaders
            .character_config_seed_by_character_loader
            .load(
                CharacterConfigSeedByCharacterLoaderParams {},
                self.0.clone(),
            )
            .await
            .context("load character_config_seed")??;

        Ok(character_config_seeds
            .into_iter()
            .map(CharacterConfigSeed::from)
            .collect())
    }

    async fn character_config_seed(
        &self,
        ctx: &mut AppCtx,
        stroke_count: i32,
    ) -> Result<Option<CharacterConfigSeed>, ApiError> {
        let stroke_count = entities::StrokeCount::try_from(stroke_count)
            .map_err(|_| GraphqlUserError::from("stroke_count must be an non negative integer"))?;

        let character_config_seed = ctx
            .loaders
            .character_config_seed_by_id_loader
            .load(
                CharacterConfigSeedByIdLoaderParams {},
                (self.0.clone(), stroke_count),
            )
            .await
            .context("load character_config_seed")??;

        Ok(character_config_seed.map(CharacterConfigSeed::from))
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

    fn random_level(&self) -> i32 {
        i32::from(self.0.random_level)
    }

    fn shared_proportion(&self) -> i32 {
        i32::from(self.0.shared_proportion)
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
        let mut user_config_repository = UserConfigsRepositoryImpl::new(ctx.pool.clone());

        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        Ok(UserConfig(user_config_repository.get(user_id).await?))
    }

    async fn node(ctx: &AppCtx, id: ID) -> Result<Option<NodeValue>, ApiError> {
        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let Some(id) = NodeId::from_id(&id) else {
            return Ok(None);
        };

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
            NodeId::CharacterConfig(_, character, stroke_count) => {
                let character_config = ctx
                    .loaders
                    .character_config_by_id_loader
                    .load(
                        CharacterConfigByIdLoaderParams { user_id },
                        (character, stroke_count),
                    )
                    .await
                    .context("load character_config")??;

                Ok(Some(NodeValue::CharacterConfig(CharacterConfig::from(
                    character_config,
                ))))
            }
            NodeId::Character(character) => {
                Ok(Some(NodeValue::Character(Character::from(character))))
            }
            NodeId::UserConfig(_) => {
                let mut user_config_repository = UserConfigsRepositoryImpl::new(ctx.pool.clone());

                let user_config = user_config_repository.get(user_id).await?;
                Ok(Some(NodeValue::UserConfig(UserConfig(user_config))))
            }
            NodeId::CharacterConfigSeed(character, stroke_count) => {
                let character_config_seed = ctx
                    .loaders
                    .character_config_seed_by_id_loader
                    .load(
                        CharacterConfigSeedByIdLoaderParams {},
                        (character, stroke_count),
                    )
                    .await
                    .context("load character_config_seed")??;

                Ok(character_config_seed
                    .map(CharacterConfigSeed::from)
                    .map(NodeValue::CharacterConfigSeed))
            }
            NodeId::File(id) => {
                let file = ctx
                    .loaders
                    .file_by_id_loader
                    .load(
                        FileByIdLoaderParams {
                            user_id,
                            verified_only: false,
                        },
                        id,
                    )
                    .await
                    .context("load file")??;
                Ok(file.map(File::from).map(NodeValue::File))
            }
            NodeId::GenerateTemplate(id) => {
                let generate_template = ctx
                    .loaders
                    .generate_template_by_id_loader
                    .load(GenerateTemplateByIdLoaderParams { user_id }, id)
                    .await
                    .context("load generate_template")??;
                Ok(generate_template
                    .map(GenerateTemplate::from)
                    .map(NodeValue::GenerateTemplate))
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

        let after_id = after
            .map(|after| -> anyhow::Result<_> {
                let Some(NodeId::CharacterConfig(_, character, stroke_count)) =
                    NodeId::from_id(&ID::new(after))
                else {
                    return Err(GraphqlUserError::from("after must be a valid cursor").into());
                };

                Ok((character, stroke_count))
            })
            .transpose()?;

        let before_id = before
            .map(|before| -> anyhow::Result<_> {
                let Some(NodeId::CharacterConfig(_, character, stroke_count)) =
                    NodeId::from_id(&ID::new(before))
                else {
                    return Err(GraphqlUserError::from("before must be a valid cursor").into());
                };

                Ok((character, stroke_count))
            })
            .transpose()?;

        let result = ctx
            .loaders
            .character_config_loader
            .load(
                CharacterConfigLoaderParams {
                    user_id,
                    after_id,
                    before_id,
                    limit: limit.clone(),
                },
                (),
            )
            .await
            .context("load character_config")??;

        let records = result
            .values
            .into_iter()
            .map(CharacterConfig::from)
            .collect::<Vec<_>>();

        Ok(CharacterConfigConnection {
            page_info: PageInfo {
                has_next_page: result.has_next && limit.kind() == entities::LimitKind::First,
                has_previous_page: result.has_next && limit.kind() == entities::LimitKind::Last,
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

        let after_id = after
            .map(|after| -> anyhow::Result<_> {
                let Some(NodeId::CharacterConfigSeed(character, stroke_count)) =
                    NodeId::from_id(&ID::new(after))
                else {
                    return Err(GraphqlUserError::from("after must be a valid cursor").into());
                };

                Ok((character, stroke_count))
            })
            .transpose()?;

        let before_id = before
            .map(|before| -> anyhow::Result<_> {
                let Some(NodeId::CharacterConfigSeed(character, stroke_count)) =
                    NodeId::from_id(&ID::new(before))
                else {
                    return Err(GraphqlUserError::from("before must be a valid cursor").into());
                };

                Ok((character, stroke_count))
            })
            .transpose()?;

        let result = ctx
            .loaders
            .character_config_seeds_loader
            .load(
                CharacterConfigSeedsLoaderParams {
                    user_id,
                    after_id,
                    before_id,
                    limit: limit.clone(),
                    include_exist_character_config,
                },
                (),
            )
            .await
            .context("load character_config_seed")??;

        let records = result
            .values
            .into_iter()
            .map(CharacterConfigSeed::from)
            .collect::<Vec<_>>();

        Ok(CharacterConfigSeedConnection {
            page_info: PageInfo {
                has_next_page: result.has_next && limit.kind() == entities::LimitKind::First,
                has_previous_page: result.has_next && limit.kind() == entities::LimitKind::Last,
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

    async fn generate_templates(
        ctx: &AppCtx,
        first: Option<i32>,
        after: Option<String>,
        last: Option<i32>,
        before: Option<String>,
    ) -> Result<GenerateTemplateConnection, ApiError> {
        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let limit = encode_limit(first, last)?;

        let after_id = after
            .map(|after| -> anyhow::Result<_> {
                let Some(NodeId::GenerateTemplate(id)) = NodeId::from_id(&ID::new(after)) else {
                    return Err(GraphqlUserError::from("after must be a valid cursor").into());
                };

                Ok(id)
            })
            .transpose()?;

        let before_id = before
            .map(|before| -> anyhow::Result<_> {
                let Some(NodeId::GenerateTemplate(id)) = NodeId::from_id(&ID::new(before)) else {
                    return Err(GraphqlUserError::from("before must be a valid cursor").into());
                };

                Ok(id)
            })
            .transpose()?;

        let result = ctx
            .loaders
            .generate_templates_loader
            .load(
                GenerateTemplatesLoaderParams {
                    user_id,
                    after_id,
                    before_id,
                    limit: limit.clone(),
                },
                (),
            )
            .await
            .context("load generate_template")??;

        let records = result
            .values
            .into_iter()
            .map(GenerateTemplate::from)
            .collect::<Vec<_>>();

        Ok(GenerateTemplateConnection {
            page_info: PageInfo {
                has_next_page: result.has_next && limit.kind() == entities::LimitKind::First,
                has_previous_page: result.has_next && limit.kind() == entities::LimitKind::Last,
                start_cursor: records.first().map(|record| record.node_id().to_string()),
                end_cursor: records.last().map(|record| record.node_id().to_string()),
            },
            edges: records
                .into_iter()
                .map(|generate_template| GenerateTemplateEdge {
                    cursor: generate_template.node_id().to_string(),
                    node: generate_template,
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
        let mut figure_records_repository = FigureRecordsRepositoryImpl::new(ctx.pool.clone());

        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let record = figure_records_repository
            .create(user_id, ctx.now, input.character.0, input.figure.0)
            .await?;

        Ok(CreateFigureRecordPayload {
            figure_record: Some(FigureRecord::from(record)),
            errors: None,
        })
    }

    async fn update_character_config(
        ctx: &AppCtx,
        input: UpdateCharacterConfigInput,
    ) -> Result<UpdateCharacterConfigPayload, ApiError> {
        let mut character_configs_repository =
            CharacterConfigsRepositoryImpl::new(ctx.pool.clone());

        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let character = input.character.0;

        let stroke_count = entities::StrokeCount::try_from(input.stroke_count)
            .map_err(|_| GraphqlUserError::from("stroke_count must be an non negative integer"))?;

        let mut character_config = character_configs_repository
            .get_by_ids(user_id.clone(), &[(character.clone(), stroke_count)])
            .await
            .context("get character_config")?
            .remove(&(character, stroke_count))
            .ok_or_else(|| anyhow::anyhow!("character_config not found"))?;

        let Ok(ratio) = input.ratio.map(entities::Ratio::try_from).transpose() else {
            return Ok(UpdateCharacterConfigPayload {
                character_config: None,
                errors: Some(vec![GraphqlErrorType {
                    message: "ratio must be an non negative integer".to_string(),
                }]),
            });
        };

        if let Some(ratio) = ratio {
            character_config = character_config.with_ratio(ratio);
        }

        if let Some(disabled) = input.disabled {
            character_config = character_config.with_disabled(disabled);
        }

        let character_config = character_configs_repository
            .save(ctx.now, character_config)
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
        let mut figure_records_repository = FigureRecordsRepositoryImpl::new(ctx.pool.clone());

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

        let figure_record = figure_records_repository
            .update(figure_record, input.disabled)
            .await?;

        Ok(UpdateFigureRecordPayload {
            figure_record: Some(FigureRecord::from(figure_record)),
            errors: None,
        })
    }

    async fn update_user_config(
        ctx: &AppCtx,
        input: UpdateUserConfigInput,
    ) -> Result<UpdateUserConfigPayload, ApiError> {
        let mut user_config_repository = UserConfigsRepositoryImpl::new(ctx.pool.clone());

        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let mut user_config = user_config_repository
            .get(user_id.clone())
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

        if let Some(random_level) = input.random_level {
            let random_level = entities::RandomLevel::try_from(random_level)
                .map_err(|_| GraphqlUserError::from("random_level is invalid"))?;
            user_config = user_config.with_random_level(random_level);
        }

        if let Some(shared_proportion) = input.shared_proportion {
            let shared_proportion = entities::SharedProportion::try_from(shared_proportion)
                .map_err(|_| GraphqlUserError::from("shared_proportion is invalid"))?;
            user_config = user_config.with_shared_proportion(shared_proportion);
        }

        let user_config = user_config_repository.save(ctx.now, user_config).await?;

        Ok(UpdateUserConfigPayload {
            user_config: Some(UserConfig::from(user_config)),
            errors: None,
        })
    }

    async fn create_file(
        ctx: &AppCtx,
        input: CreateFileInput,
    ) -> Result<CreateFilePayload, ApiError> {
        let mut files_repository = FilesRepositoryImpl::new(ctx.pool.clone());

        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let mime_type = entities::MimeType::try_from(input.mime_type.clone())
            .map_err(|_| GraphqlUserError::from("mime_type is invalid"))?;

        let size = entities::FileSize::try_from(input.size)
            .map_err(|_| GraphqlUserError::from("size is invalid"))?;

        let file = files_repository
            .create(user_id, ctx.now, mime_type, size)
            .await?;

        Ok(CreateFilePayload {
            file: Some(File::from(file)),
            errors: None,
        })
    }

    async fn verify_file(
        ctx: &AppCtx,
        input: VerifyFileInput,
    ) -> Result<VerifyFilePayload, ApiError> {
        let mut files_repository = FilesRepositoryImpl::new(ctx.pool.clone());
        let mut storage = StorageImpl::new(ctx.config.clone(), ctx.s3_client.clone());

        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let file = ctx
            .loaders
            .file_by_id_loader
            .load(
                FileByIdLoaderParams {
                    user_id: user_id.clone(),
                    verified_only: false,
                },
                entities::FileId::from(input.id.0),
            )
            .await
            .context("load file")??;
        let file = file.ok_or_else(|| GraphqlUserError::from("File not found"))?;

        // TODO: ここでやることではない
        if file.verified {
            return Ok(VerifyFilePayload {
                file: Some(File::from(file)),
                errors: None,
            });
        }

        storage.verify(&file).await?;

        let file = files_repository
            .verified(ctx.now, file)
            .await
            .context("verify file")?;

        Ok(VerifyFilePayload {
            file: Some(File::from(file)),
            errors: None,
        })
    }

    async fn create_generate_template(
        ctx: &AppCtx,
        input: CreateGenerateTemplateInput,
    ) -> Result<CreateGenerateTemplatePayload, ApiError> {
        let mut generate_templates_repository =
            GenerateTemplatesRepositoryImpl::new(ctx.pool.clone());

        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let id = entities::GenerateTemplateId::from(Ulid::from_datetime(ctx.now));
        let background_image_file = ctx
            .loaders
            .file_by_id_loader
            .load(
                FileByIdLoaderParams {
                    user_id: user_id.clone(),
                    verified_only: true,
                },
                entities::FileId::from(input.background_image_file_id.0),
            )
            .await
            .context("load background image file")??
            .ok_or_else(|| {
                GraphqlUserError::from("background_image_file_id must be a valid file id")
            })?;
        let font_color = entities::Color::try_from(input.font_color)
            .map_err(|_| GraphqlUserError::from("font_color must be a valid hex color"))?;
        let writing_mode = entities::WritingMode::try_from(input.writing_mode)
            .map_err(|_| GraphqlUserError::from("writing_mode must be a valid writing mode"))?;
        let margin_block_start = entities::Margin::try_from(input.margin_block_start)
            .map_err(|_| GraphqlUserError::from("margin_block_start must be a valid margin"))?;
        let margin_inline_start = entities::Margin::try_from(input.margin_inline_start)
            .map_err(|_| GraphqlUserError::from("margin_inline_start must be a valid margin"))?;
        let line_spacing = entities::Spacing::try_from(input.line_spacing)
            .map_err(|_| GraphqlUserError::from("line_spacing must be a valid spacing"))?;
        let letter_spacing = entities::Spacing::try_from(input.letter_spacing)
            .map_err(|_| GraphqlUserError::from("letter_spacing must be a valid spacing"))?;
        let font_size = entities::FontSize::try_from(input.font_size)
            .map_err(|_| GraphqlUserError::from("font_size must be a valid font size"))?;
        let font_weight = entities::FontWeight::try_from(input.font_weight)
            .map_err(|_| GraphqlUserError::from("font_weight must be a valid font weight"))?;

        let generate_template = entities::GenerateTemplate {
            id,
            user_id,
            background_image_file_id: background_image_file.id,
            font_color,
            writing_mode,
            margin_block_start,
            margin_inline_start,
            line_spacing,
            letter_spacing,
            font_size,
            font_weight,
            created_at: ctx.now,
            updated_at: ctx.now,
            disabled: false,
            version: entities::Version::none(),
        };

        let generate_template = generate_templates_repository
            .create(generate_template)
            .await?;

        Ok(CreateGenerateTemplatePayload {
            generate_template: Some(GenerateTemplate::from(generate_template)),
            errors: None,
        })
    }

    async fn delete_generate_template(
        ctx: &AppCtx,
        input: DeleteGenerateTemplateInput,
    ) -> Result<DeleteGenerateTemplatePayload, ApiError> {
        let mut generate_templates_repository =
            GenerateTemplatesRepositoryImpl::new(ctx.pool.clone());

        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;
        let id = entities::GenerateTemplateId::from(input.generate_template_id.0);
        let mut generate_template = ctx
            .loaders
            .generate_template_by_id_loader
            .load(GenerateTemplateByIdLoaderParams { user_id }, id)
            .await
            .context("load generate_template")??
            .ok_or_else(|| GraphqlUserError::from("id must be a valid generate template id"))?;

        generate_template.disabled = true;
        generate_templates_repository
            .update(ctx.now, generate_template)
            .await
            .context("update generate_template")?;

        Ok(DeleteGenerateTemplatePayload {
            id: NodeId::GenerateTemplate(id).to_id(),
            errors: None,
        })
    }

    async fn update_generate_template(
        ctx: &AppCtx,
        input: UpdateGenerateTemplateInput,
    ) -> Result<UpdateGenerateTemplatePayload, ApiError> {
        let mut generate_templates_repository =
            GenerateTemplatesRepositoryImpl::new(ctx.pool.clone());

        let user_id = ctx
            .user_id
            .clone()
            .ok_or_else(|| GraphqlUserError::from("Authentication required"))?;

        let id = entities::GenerateTemplateId::from(input.generate_template_id.0);

        let mut generate_template = ctx
            .loaders
            .generate_template_by_id_loader
            .load(
                GenerateTemplateByIdLoaderParams {
                    user_id: user_id.clone(),
                },
                id,
            )
            .await
            .context("load generate_template")??
            .ok_or_else(|| GraphqlUserError::from("id must be a valid generate template id"))?;

        if let Some(background_image_file_id) = input.background_image_file_id {
            let background_image_file = ctx
                .loaders
                .file_by_id_loader
                .load(
                    FileByIdLoaderParams {
                        user_id: user_id.clone(),
                        verified_only: true,
                    },
                    entities::FileId::from(background_image_file_id.0),
                )
                .await
                .context("load background image file")??
                .ok_or_else(|| {
                    GraphqlUserError::from("background_image_file_id must be a valid file id")
                })?;
            generate_template.background_image_file_id = background_image_file.id;
        }

        if let Some(font_color) = input.font_color {
            generate_template.font_color = entities::Color::try_from(font_color)
                .map_err(|_| GraphqlUserError::from("font_color must be a valid hex color"))?;
        }

        if let Some(writing_mode) = input.writing_mode {
            generate_template.writing_mode = entities::WritingMode::from(writing_mode);
        }

        if let Some(margin_block_start) = input.margin_block_start {
            generate_template.margin_block_start = entities::Margin::try_from(margin_block_start)
                .map_err(|_| {
                GraphqlUserError::from("margin_block_start must be a valid margin")
            })?;
        }

        if let Some(margin_inline_start) = input.margin_inline_start {
            generate_template.margin_inline_start = entities::Margin::try_from(margin_inline_start)
                .map_err(|_| {
                    GraphqlUserError::from("margin_inline_start must be a valid margin")
                })?;
        }

        if let Some(line_spacing) = input.line_spacing {
            generate_template.line_spacing = entities::Spacing::try_from(line_spacing)
                .map_err(|_| GraphqlUserError::from("line_spacing must be a valid spacing"))?;
        }

        if let Some(letter_spacing) = input.letter_spacing {
            generate_template.letter_spacing = entities::Spacing::try_from(letter_spacing)
                .map_err(|_| GraphqlUserError::from("letter_spacing must be a valid spacing"))?;
        }

        if let Some(font_size) = input.font_size {
            generate_template.font_size = entities::FontSize::try_from(font_size)
                .map_err(|_| GraphqlUserError::from("font_size must be a valid font size"))?;
        }

        if let Some(font_weight) = input.font_weight {
            generate_template.font_weight = entities::FontWeight::try_from(font_weight)
                .map_err(|_| GraphqlUserError::from("font_weight must be a valid font weight"))?;
        }

        let generate_template = generate_templates_repository
            .update(ctx.now, generate_template)
            .await
            .context("update generate_template")?;

        Ok(UpdateGenerateTemplatePayload {
            generate_template: Some(GenerateTemplate::from(generate_template)),
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
