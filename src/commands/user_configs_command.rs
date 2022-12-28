use crate::entities;
use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use sqlx::{Postgres, Transaction};

pub async fn update(
    trx: &mut Transaction<'_, Postgres>,
    now: DateTime<Utc>,
    mut user_config: entities::UserConfig,
    allow_sharing_character_configs: Option<bool>,
    allow_sharing_figure_records: Option<bool>,
) -> anyhow::Result<entities::UserConfig> {
    let prev_version = user_config.version;
    user_config.version += 1;
    user_config.updated_at = Some(now);

    if let Some(allow_sharing_character_configs) = allow_sharing_character_configs {
        user_config.allow_sharing_character_configs = allow_sharing_character_configs;
    }

    if let Some(allow_sharing_figure_records) = allow_sharing_figure_records {
        user_config.allow_sharing_figure_records = allow_sharing_figure_records;
    }

    if prev_version == 0 {
        sqlx::query!(
            r#" 
                INSERT
                    INTO user_configs (
                        user_id,
                        allow_sharing_character_configs,
                        allow_sharing_figure_records,
                        updated_at,
                        version
                    ) VALUES ($1, $2, $3, $4, $5)
                "#,
            &user_config.user_id,
            user_config.allow_sharing_character_configs,
            user_config.allow_sharing_figure_records,
            user_config.updated_at,
            i32::try_from(user_config.version)?
        )
        .execute(&mut *trx)
        .await
        .context("insert character_config")?;
    } else {
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
        .execute(&mut *trx)
        .await
        .context("update character_config")?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("conflict"));
        }
    }

    Ok(user_config)
}
