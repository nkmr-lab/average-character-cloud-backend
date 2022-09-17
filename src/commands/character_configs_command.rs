use crate::entities;
use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use thiserror::Error;
use ulid::Ulid;

#[derive(Clone, Error, Debug)]
pub enum CreateError {
    #[error("character already exists")]
    AlreadyExists,
}

pub async fn create(
    pool: &Pool<Postgres>,
    user_id: String,
    now: DateTime<Utc>,
    character: entities::Character,
    stroke_count: usize,
) -> anyhow::Result<Result<entities::CharacterConfig, CreateError>> {
    let mut trx = pool.begin().await?;

    let character_config = entities::CharacterConfig {
        id: Ulid::from_datetime(now),
        user_id,
        character,
        created_at: now,
        updated_at: now,
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
        character_config.user_id,
        String::from(character_config.character.clone()),
    )
    .fetch_optional(&mut trx)
    .await
    .context("check character_config exist")?
    .is_some();

    if exists {
        return Ok(Err(CreateError::AlreadyExists));
    }

    sqlx::query!(
            r#"
                INSERT INTO character_configs (id, user_id, character, created_at, updated_at, stroke_count, version)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            character_config.id.to_string(),
            character_config.user_id,
            String::from(character_config.character.clone()),
            character_config.created_at,
            character_config.updated_at,
            character_config.stroke_count as i32,
            character_config.version,
        )
        .execute(&mut trx)
        .await
        .context("insert character_configs")?;

    trx.commit().await?;
    Ok(Ok(character_config))
}

pub async fn update(
    pool: &Pool<Postgres>,
    user_id: String,
    now: DateTime<Utc>,
    mut character_config: entities::CharacterConfig,
    stroke_count: Option<usize>,
) -> anyhow::Result<entities::CharacterConfig> {
    let mut trx = pool.begin().await?;

    let prev_version = character_config.version;

    character_config.version += 1;
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
                    id = $4
                    AND
                    user_id = $5
                    AND
                    version = $6
            "#,
        &character_config.updated_at,
        character_config.stroke_count as i32,
        character_config.version,
        character_config.id.to_string(),
        &user_id,
        prev_version,
    )
    .execute(&mut trx)
    .await
    .context("update character_config")?;

    if result.rows_affected() == 0 {
        return Err(anyhow!("conflict").into());
    }

    trx.commit().await?;
    Ok(character_config)
}