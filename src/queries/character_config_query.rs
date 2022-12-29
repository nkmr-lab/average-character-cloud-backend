use sqlx::PgPool;

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::entities;
use anyhow::Context;
use async_trait::async_trait;

use crate::BatchFnWithParams;
use crate::ShareableError;
#[derive(Debug, Clone)]
pub struct CharacterConfigModel {
    pub user_id: String,
    pub character: String,
    pub stroke_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
}

impl CharacterConfigModel {
    pub fn into_entity(self) -> anyhow::Result<entities::CharacterConfig> {
        let character = entities::Character::try_from(self.character.as_str())?;

        Ok(entities::CharacterConfig {
            user_id: entities::UserId::from(self.user_id),
            character,
            stroke_count: usize::try_from(self.stroke_count)?,
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
    pub user_id: entities::UserId,
}

#[async_trait]
impl BatchFnWithParams for CharacterConfigByCharacterLoader {
    type K = entities::Character;
    type V = Result<Option<entities::CharacterConfig>, ShareableError>;
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
                String::from(params.user_id.clone()),
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
pub struct CharacterConfigsLoader {
    pub pool: PgPool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharacterConfigsLoaderParams {
    pub user_id: entities::UserId,
    pub after_character: Option<entities::Character>,
    pub before_character: Option<entities::Character>,
    pub limit: entities::Limit,
}

#[async_trait]
impl BatchFnWithParams for CharacterConfigsLoader {
    type K = ();
    type V = Result<(Vec<entities::CharacterConfig>, bool), ShareableError>;
    type P = CharacterConfigsLoaderParams;

    async fn load_with_params(
        &mut self,
        params: &Self::P,
        _: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let result: Result<_, ShareableError> = (|| async {
            let models = sqlx::query_as!(
                CharacterConfigModel,
                r#"
            SELECT
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
                ($2::VARCHAR(64) IS NULL OR character > $2)
                AND
                ($3::VARCHAR(64) IS NULL OR character < $3)
            ORDER BY
                CASE WHEN $4 = 0 THEN character END ASC,
                CASE WHEN $4 = 1 THEN character END DESC
            LIMIT $5
        "#,
                String::from(params.user_id.clone()),
                params.clone().after_character.map(String::from),
                params.clone().before_character.map(String::from),
                i32::from(params.limit.kind() == entities::LimitKind::Last),
                i64::from(params.limit.value()) + 1,
            )
            .fetch_all(&self.pool)
            .await
            .context("fetch character_configs")?;

            let mut character_configs = models
                .into_iter()
                .map(|row| row.into_entity())
                .collect::<anyhow::Result<Vec<_>>>()
                .context("convert CharacterConfig")?;

            let has_extra = character_configs.len()
                > usize::try_from(params.limit.value()).context("into usize")?;

            character_configs
                .truncate(usize::try_from(params.limit.value()).context("into usize")?);

            if params.limit.kind() == entities::LimitKind::Last {
                character_configs.reverse();
            }

            Ok((character_configs, has_extra))
        })()
        .await
        .map_err(ShareableError);

        vec![((), result)].into_iter().collect()
    }
}
