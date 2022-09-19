use crate::entities;
use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub async fn update(
    pool: &PgPool,
    now: DateTime<Utc>,
    mut user_config: entities::UserConfig,
    allow_sharing_character_configs: Option<bool>,
    allow_sharing_figure_records: Option<bool>,
) -> anyhow::Result<entities::UserConfig> {
    let mut trx = pool.begin().await?;
    let prev_version = user_config.version;
    user_config.version += 1;
    user_config.updated_at = Some(now);

    if let Some(allow_sharing_character_configs) = allow_sharing_character_configs {
        user_config.allow_sharing_character_configs = allow_sharing_character_configs;
    }

    if let Some(allow_sharing_figure_records) = allow_sharing_figure_records {
        user_config.allow_sharing_figure_records = allow_sharing_figure_records;
    }

    let result = sqlx::query!(
        r#"
            UPDATE user_configs
                SET
                    updated_at = $1,
                    allow_sharing_character_configs = $2,
                    allow_sharing_figure_records = $3,
                    version = $4
                WHERE
                    user_id = $5
                    AND
                    version = $6
            "#,
        &user_config
            .updated_at
            .ok_or(anyhow!("updated_at is None"))?,
        user_config.allow_sharing_character_configs,
        user_config.allow_sharing_figure_records,
        i32::try_from(user_config.version)?,
        &user_config.user_id,
        i32::try_from(prev_version)?,
    )
    .execute(&mut trx)
    .await
    .context("update character_config")?;

    if result.rows_affected() == 0 {
        return Err(anyhow!("conflict"));
    }

    trx.commit().await?;

    Ok(user_config)
}
