use std::convert::TryInto;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::entities;

#[derive(Debug, Clone)]
pub struct UserConfigModel {
    user_id: String,
    allow_sharing_character_configs: bool,
    allow_sharing_figure_records: bool,
    updated_at: DateTime<Utc>,
    version: i32,
}

pub async fn load_user_config(
    pool: &PgPool,
    user_id: entities::UserId,
) -> anyhow::Result<entities::UserConfig> {
    let record = sqlx::query_as!(
        UserConfigModel,
        r#"
        SELECT
            user_id,
            allow_sharing_character_configs,
            allow_sharing_figure_records,
            updated_at,
            version
        FROM
            user_configs
        WHERE
            user_id = $1
        "#,
        String::from(user_id.clone())
    )
    .fetch_optional(pool)
    .await?;

    let user_config = record
        .map(|record| -> anyhow::Result<_> {
            Ok(entities::UserConfig {
                user_id: entities::UserId::from(record.user_id),
                allow_sharing_character_configs: record.allow_sharing_character_configs,
                allow_sharing_figure_records: record.allow_sharing_figure_records,
                updated_at: Some(record.updated_at),
                version: record.version.try_into()?,
            })
        })
        .unwrap_or_else(|| Ok(entities::UserConfig::default_config(user_id)))?;

    Ok(user_config)
}
