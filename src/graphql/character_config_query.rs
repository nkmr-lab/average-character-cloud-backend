use sqlx::PgPool;

use std::collections::HashMap;

use std::str::FromStr;

use chrono::{DateTime, Utc};

use ulid::Ulid;

use super::dataloader_with_params::BatchFnWithParams;
use crate::entities;
use anyhow::Context;
use async_trait::async_trait;

use super::dataloader_with_params::ShareableError;

#[derive(Debug, Clone)]
pub struct CharacterConfigModel {
    pub id: String,
    pub user_id: String,
    pub character: String,
    pub stroke_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
}

impl CharacterConfigModel {
    pub fn into_entity(self) -> anyhow::Result<entities::character_config::CharacterConfig> {
        let id = Ulid::from_str(&self.id).context("ulid decode error")?;

        let character = entities::character::Character::try_from(self.character.as_str())?;

        Ok(entities::character_config::CharacterConfig {
            id,
            user_id: self.user_id,
            character,
            stroke_count: self.stroke_count as usize,
            created_at: self.created_at,
            updated_at: self.updated_at,
            version: self.version,
        })
    }
}

#[derive(Clone, Debug)]
pub struct CharacterConfigByCharacterLoader {
    pub pool: PgPool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharacterConfigByCharacterLoaderParams {
    pub user_id: String,
}

#[async_trait]
impl BatchFnWithParams for CharacterConfigByCharacterLoader {
    type K = entities::character::Character;
    type V = Result<Option<entities::character_config::CharacterConfig>, ShareableError>;
    type P = CharacterConfigByCharacterLoaderParams;

    async fn load_with_params(
        &mut self,
        params: &Self::P,
        keys: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let character_values = keys
            .iter()
            .map(|character| String::from(character.clone()))
            .collect::<Vec<_>>();

        let result: Result<_, ShareableError> = (|| async {
            let models = sqlx::query_as!(
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
                    character = Any($2)
            "#,
                &params.user_id,
                character_values.as_slice(),
            )
            .fetch_all(&self.pool)
            .await
            .context("fetch character_configs")?;

            let character_configs = models
                .into_iter()
                .map(|model| model.into_entity())
                .collect::<anyhow::Result<Vec<_>>>()
                .context("convert CharacterConfig")?;

            let character_config_map = character_configs
                .into_iter()
                .map(|character_config| (character_config.character.clone(), character_config))
                .collect::<HashMap<_, _>>();

            Ok(character_config_map)
        })()
        .await
        .map_err(ShareableError);

        keys.iter()
            .map(|key| {
                (
                    key.clone(),
                    result
                        .as_ref()
                        .map(|character_config_map| character_config_map.get(key).cloned())
                        .map_err(|e| e.clone()),
                )
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct CharacterConfigByIdLoader {
    pub pool: PgPool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharacterConfigByIdLoaderParams {
    pub user_id: String,
}

#[async_trait]
impl BatchFnWithParams for CharacterConfigByIdLoader {
    type K = Ulid;
    type V = Result<Option<entities::character_config::CharacterConfig>, ShareableError>;
    type P = CharacterConfigByIdLoaderParams;

    async fn load_with_params(
        &mut self,
        params: &Self::P,
        keys: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let ids = keys.iter().map(|id| id.to_string()).collect::<Vec<_>>();

        let result: Result<_, ShareableError> = (|| async {
            let models = sqlx::query_as!(
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
                    id = Any($2)
            "#,
                &params.user_id,
                ids.as_slice(),
            )
            .fetch_all(&self.pool)
            .await
            .context("fetch character_config by id")?;

            let character_configs = models
                .into_iter()
                .map(|model| model.into_entity())
                .collect::<anyhow::Result<Vec<_>>>()
                .context("convert CharacterConfig")?;

            let character_config_map = character_configs
                .into_iter()
                .map(|character_config| (character_config.id.clone(), character_config))
                .collect::<HashMap<_, _>>();

            Ok(character_config_map)
        })()
        .await
        .map_err(ShareableError);

        keys.iter()
            .map(|key| {
                (
                    key.clone(),
                    result
                        .as_ref()
                        .map(|character_config_map| character_config_map.get(key).cloned())
                        .map_err(|e| e.clone()),
                )
            })
            .collect()
    }
}
