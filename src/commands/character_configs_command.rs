use crate::entities;
use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use sqlx::{Acquire, Postgres};
use thiserror::Error;

#[derive(Clone, Error, Debug)]
pub enum CreateError {
    #[error("character already exists")]
    AlreadyExists,
}

pub async fn create(
    conn: impl Acquire<'_, Database = Postgres>,
    user_id: entities::UserId,
    now: DateTime<Utc>,
    character: entities::Character,
    stroke_count: entities::StrokeCount,
) -> anyhow::Result<Result<entities::CharacterConfig, CreateError>> {
    let mut trx = conn.begin().await?;
    let character_config = entities::CharacterConfig {
        user_id,
        character,
        created_at: now,
        updated_at: now,
        stroke_count,
        version: entities::Version::new(),
    };

    // check exist
    let exists = sqlx::query!(
        r#"
            SELECT
                1 as check
            FROM
                character_configs
            WHERE
                user_id = $1
                AND
                character = $2
        "#,
        String::from(character_config.user_id.clone()),
        String::from(character_config.character.clone()),
    )
    .fetch_optional(&mut *trx)
    .await
    .context("check character_config exist")?
    .is_some();

    if exists {
        return Ok(Err(CreateError::AlreadyExists));
    }

    sqlx::query!(
            r#"
                INSERT INTO character_configs (user_id, character, created_at, updated_at, stroke_count, version)
                VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            String::from(character_config.user_id.clone()),
            String::from(character_config.character.clone()),
            character_config.created_at,
            character_config.updated_at,
            i32::from(character_config.stroke_count),
            i32::from(character_config.version),
        )
        .execute(&mut *trx)
        .await
        .context("insert character_configs")?;

    trx.commit().await?;
    Ok(Ok(character_config))
}

pub async fn update(
    conn: impl Acquire<'_, Database = Postgres>,
    now: DateTime<Utc>,
    mut character_config: entities::CharacterConfig,
    stroke_count: Option<entities::StrokeCount>,
) -> anyhow::Result<entities::CharacterConfig> {
    let mut trx = conn.begin().await?;
    let prev_version = character_config.version;

    character_config.version = character_config.version.next();
    character_config.updated_at = now;
    if let Some(stroke_count) = stroke_count {
        character_config.stroke_count = stroke_count;
    }

    let result = sqlx::query!(
        r#"
            UPDATE character_configs
                SET
                    updated_at = $1,
                    stroke_count = $2,
                    version = $3
                WHERE
                    user_id = $4
                    AND
                    character = $5
                    AND
                    version = $6
            "#,
        &character_config.updated_at,
        i32::from(character_config.stroke_count),
        i32::from(character_config.version),
        String::from(character_config.user_id.clone()),
        String::from(character_config.character.clone()),
        i32::from(prev_version),
    )
    .execute(&mut *trx)
    .await
    .context("update character_config")?;

    if result.rows_affected() == 0 {
        return Err(anyhow!("conflict"));
    }

    trx.commit().await?;
    Ok(character_config)
}
