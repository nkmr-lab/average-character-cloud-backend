use sqlx::PgPool;

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::entities;
use crate::values::{Limit, LimitKind};
use anyhow::Context;
use async_trait::async_trait;

use crate::BatchFnWithParams;
use crate::ShareableError;
#[derive(Debug, Clone)]
pub struct CharacterConfigSeedModel {
    pub character: String,
    pub stroke_count: i32,
    pub updated_at: DateTime<Utc>,
}

impl CharacterConfigSeedModel {
    pub fn into_entity(self) -> anyhow::Result<entities::CharacterConfigSeed> {
        let character = entities::Character::try_from(self.character.as_str())?;

        Ok(entities::CharacterConfigSeed {
            character,
            stroke_count: usize::try_from(self.stroke_count)?,
            updated_at: self.updated_at,
        })
    }
}

#[derive(Clone, Debug)]
pub struct CharacterConfigSeedByCharacterLoader {
    pub pool: PgPool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharacterConfigSeedByCharacterLoaderParams {
    pub user_id: String,
}

#[async_trait]
impl BatchFnWithParams for CharacterConfigSeedByCharacterLoader {
    type K = entities::Character;
    type V = Result<Option<entities::CharacterConfigSeed>, ShareableError>;
    type P = CharacterConfigSeedByCharacterLoaderParams;

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
                CharacterConfigSeedModel,
                r#"
                SELECT
                    character,
                    stroke_count,
                    updated_at
                FROM
                    character_config_seeds
                WHERE
                    NOT EXISTS (
                        SELECT
                            1
                        FROM
                            character_configs
                        WHERE
                            character_configs.user_id = $1
                            AND character_configs.character = character_config_seeds.character
                    )
                    AND
                    character = Any($2)
            "#,
                &params.user_id,
                character_values.as_slice(),
            )
            .fetch_all(&self.pool)
            .await
            .context("fetch character_config_seeds")?;

            let character_config_seeds = models
                .into_iter()
                .map(|model| model.into_entity())
                .collect::<anyhow::Result<Vec<_>>>()
                .context("convert CharacterConfigSeed")?;

            let character_config_seed_map = character_config_seeds
                .into_iter()
                .map(|character_config_seed| {
                    (
                        character_config_seed.character.clone(),
                        character_config_seed,
                    )
                })
                .collect::<HashMap<_, _>>();

            Ok(character_config_seed_map)
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
pub struct CharacterConfigSeedsLoader {
    pub pool: PgPool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CharacterConfigSeedsLoaderParams {
    pub user_id: String,
    pub after_character: Option<entities::Character>,
    pub before_character: Option<entities::Character>,
    pub limit: Limit,
}

#[async_trait]
impl BatchFnWithParams for CharacterConfigSeedsLoader {
    type K = ();
    type V = Result<(Vec<entities::CharacterConfigSeed>, bool), ShareableError>;
    type P = CharacterConfigSeedsLoaderParams;

    async fn load_with_params(
        &mut self,
        params: &Self::P,
        _: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let result: Result<_, ShareableError> = (|| async {
            let models = sqlx::query_as!(
                CharacterConfigSeedModel,
                r#"
            SELECT
                character,
                stroke_count,
                updated_at
            FROM
                character_config_seeds
            WHERE
                NOT EXISTS (
                    SELECT
                        1
                    FROM
                        character_configs
                    WHERE
                        character_configs.user_id = $1
                        AND character_configs.character = character_config_seeds.character
                )
                AND
                ($2::VARCHAR(64) IS NULL OR character < $2)
                AND
                ($3::VARCHAR(64) IS NULL OR character > $3)
            ORDER BY
                CASE WHEN $4 = 0 THEN character END ASC,
                CASE WHEN $4 = 1 THEN character END DESC
            LIMIT $5
        "#,
                &params.user_id,
                params.clone().after_character.map(String::from),
                params.clone().before_character.map(String::from),
                i32::from(params.limit.kind == LimitKind::Last),
                i64::from(params.limit.value) + 1,
            )
            .fetch_all(&self.pool)
            .await
            .context("fetch character_config_seeds")?;

            let mut character_config_seeds = models
                .into_iter()
                .map(|row| row.into_entity())
                .collect::<anyhow::Result<Vec<_>>>()
                .context("convert CharacterConfigSeed")?;

            let has_extra = character_config_seeds.len()
                > usize::try_from(params.limit.value).context("into usize")?;

            character_config_seeds
                .truncate(usize::try_from(params.limit.value).context("into usize")?);

            if params.limit.kind == LimitKind::Last {
                character_config_seeds.reverse();
            }

            Ok((character_config_seeds, has_extra))
        })()
        .await
        .map_err(ShareableError);

        vec![((), result)].into_iter().collect()
    }
}
