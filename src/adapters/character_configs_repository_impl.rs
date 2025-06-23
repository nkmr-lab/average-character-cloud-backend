use std::collections::HashMap;

use crate::{entities, ports};
use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use sqlx::{Acquire, Postgres};

#[derive(Debug, Clone)]
pub struct CharacterConfigModel {
    pub user_id: String,
    pub character: String,
    pub stroke_count: i32,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
    pub ratio: i32,
    pub disabled: bool,
}

impl CharacterConfigModel {
    pub fn into_entity(self) -> anyhow::Result<entities::CharacterConfig> {
        let character = entities::Character::try_from(self.character.as_str())?;

        Ok(entities::CharacterConfig {
            user_id: entities::UserId::from(self.user_id),
            character,
            stroke_count: entities::StrokeCount::try_from(self.stroke_count)?,
            updated_at: Some(self.updated_at),
            version: entities::Version::try_from(self.version)?,
            ratio: entities::Ratio::try_from(self.ratio)?,
            disabled: self.disabled,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CharacterConfigsRepositoryImpl<A> {
    db: A,
}

impl<A> CharacterConfigsRepositoryImpl<A> {
    pub fn new(db: A) -> Self {
        Self { db }
    }
}

impl<A> ports::CharacterConfigsRepository for CharacterConfigsRepositoryImpl<A>
where
    A: Send,
    for<'c> &'c A: Acquire<'c, Database = Postgres>,
{
    type Error = anyhow::Error;

    async fn save(
        &mut self,
        now: DateTime<Utc>,
        mut character_config: entities::CharacterConfig,
    ) -> Result<entities::CharacterConfig, Self::Error> {
        let mut trx = self.db.begin().await?;
        let prev_version = character_config.version;
        character_config.version = character_config.version.next();
        character_config.updated_at = Some(now);

        if prev_version.is_none() {
            sqlx::query!(
                r#"
                    INSERT INTO character_configs (user_id, character, updated_at, stroke_count, ratio, version, disabled)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                String::from(character_config.user_id.clone()),
                String::from(character_config.character.clone()),
                character_config.updated_at,
                i32::from(character_config.stroke_count),
                i32::from(character_config.ratio),
                i32::from(character_config.version),
                character_config.disabled,
            )
            .execute(&mut *trx)
            .await
            .context("insert character_configs")?;
        } else {
            let result = sqlx::query!(
                r#"
                    UPDATE character_configs
                        SET
                            updated_at = $1,
                            ratio = $2,
                            version = $3,
                            disabled = $8
                        WHERE
                            user_id = $4
                            AND
                            character = $5
                            AND
                            stroke_count = $6
                            AND
                            version = $7
                "#,
                &character_config
                    .updated_at
                    .ok_or(anyhow!("updated_at is None"))?,
                i32::from(character_config.ratio),
                i32::from(character_config.version),
                String::from(character_config.user_id.clone()),
                String::from(character_config.character.clone()),
                i32::from(character_config.stroke_count),
                i32::from(prev_version),
                character_config.disabled,
            )
            .execute(&mut *trx)
            .await
            .context("update character_config")?;

            if result.rows_affected() == 0 {
                return Err(anyhow!("conflict"));
            }
        }

        trx.commit().await?;
        Ok(character_config)
    }

    async fn get_by_characters(
        &mut self,
        characters: &[entities::Character],
        user_id: entities::UserId,
    ) -> Result<Vec<entities::CharacterConfig>, Self::Error> {
        let mut conn = self.db.acquire().await?;
        let character_values = characters
            .iter()
            .map(|character| String::from(character.clone()))
            .collect::<Vec<_>>();

        let models = sqlx::query_as!(
            CharacterConfigModel,
            r#"
                SELECT
                    user_id,
                    character,
                    stroke_count,
                    ratio,
                    updated_at,
                    version,
                    disabled
                FROM
                    character_configs
                WHERE
                    user_id = $1
                    AND
                    character = Any($2)
                    AND
                    disabled = false
            "#,
            String::from(user_id.clone()),
            character_values.as_slice(),
        )
        .fetch_all(&mut *conn)
        .await
        .context("fetch character_configs")?;

        let character_configs = models
            .into_iter()
            .map(|model| model.into_entity())
            .collect::<anyhow::Result<Vec<_>>>()
            .context("convert CharacterConfig")?;

        Ok(character_configs)
    }

    async fn query(
        &mut self,
        user_id: entities::UserId,
        after_id: Option<(entities::Character, entities::StrokeCount)>,
        before_id: Option<(entities::Character, entities::StrokeCount)>,
        limit: entities::Limit,
    ) -> Result<Vec<entities::CharacterConfig>, Self::Error> {
        let mut conn = self.db.acquire().await?;

        let after_character = after_id
            .clone()
            .map(|(character, _)| String::from(character));
        let after_stroke_count = after_id.map(|(_, stroke_count)| i32::from(stroke_count));
        let before_character = before_id
            .clone()
            .map(|(character, _)| String::from(character));
        let before_stroke_count = before_id.map(|(_, stroke_count)| i32::from(stroke_count));

        let models = sqlx::query_as!(
            CharacterConfigModel,
            r#"
            SELECT
                user_id,
                character,
                stroke_count,
                ratio,
                updated_at,
                version,
                disabled
            FROM
                character_configs
            WHERE
                user_id = $1
                AND
                ($2::VARCHAR(64) IS NULL OR (character, stroke_count) > ($2, $3))
                AND
                ($4::VARCHAR(64) IS NULL OR (character, stroke_count) < ($4, $5))
                AND
                disabled = false
            ORDER BY
                CASE WHEN $6 = 0 THEN (character, stroke_count) END ASC,
                CASE WHEN $6 = 1 THEN (character, stroke_count) END DESC
            LIMIT $7
        "#,
            String::from(user_id.clone()),
            after_character,
            after_stroke_count,
            before_character,
            before_stroke_count,
            i32::from(limit.kind() == entities::LimitKind::Last),
            i64::from(limit.value()),
        )
        .fetch_all(&mut *conn)
        .await
        .context("fetch character_configs")?;

        let character_configs = models
            .into_iter()
            .map(|row| row.into_entity())
            .collect::<anyhow::Result<Vec<_>>>()
            .context("convert CharacterConfig")?;

        Ok(character_configs)
    }

    async fn get_by_ids(
        &mut self,
        user_id: entities::UserId,
        keys: &[(entities::Character, entities::StrokeCount)],
    ) -> Result<
        HashMap<(entities::Character, entities::StrokeCount), entities::CharacterConfig>,
        Self::Error,
    > {
        let mut conn = self.db.acquire().await?;

        let character_values = keys
            .iter()
            .map(|(character, _)| String::from(character.clone()))
            .collect::<Vec<_>>();

        let stroke_count_values = keys
            .iter()
            .map(|(_, stroke_count)| i32::from(*stroke_count))
            .collect::<Vec<_>>();

        let models = sqlx::query_as!(
            CharacterConfigModel,
            r#"
            WITH input_pairs AS (
                SELECT a, b
                FROM unnest($1::VARCHAR(8)[], $2::INTEGER[]) AS t(a, b)
            )
            SELECT
                user_id,
                character,
                stroke_count,
                ratio,
                updated_at,
                version,
                disabled
            FROM
                character_configs
            JOIN
                input_pairs ON character_configs.character = input_pairs.a
                AND character_configs.stroke_count = input_pairs.b
            WHERE
                user_id = $3
        "#,
            character_values.as_slice(),
            stroke_count_values.as_slice(),
            String::from(user_id.clone()),
        )
        .fetch_all(&mut *conn)
        .await
        .context("fetch character_config")?;

        let character_configs = models
            .into_iter()
            .map(|row| row.into_entity())
            .collect::<anyhow::Result<Vec<_>>>()
            .context("convert CharacterConfig")?;

        let mut character_config_map = character_configs
            .iter()
            .map(|config| {
                (
                    (config.character.clone(), config.stroke_count),
                    config.clone(),
                )
            })
            .collect::<std::collections::HashMap<_, _>>();

        for key in keys {
            if !character_config_map.contains_key(key) {
                character_config_map.insert(
                    key.clone(),
                    entities::CharacterConfig::default_config(
                        user_id.clone(),
                        key.0.clone(),
                        key.1,
                    ),
                );
            }
        }

        Ok(character_config_map)
    }
}
