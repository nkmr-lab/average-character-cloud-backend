use anyhow::{anyhow, Context};
use sqlx::{Acquire, PgConnection};

use crate::{entities, ports};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
struct UserConfigModel {
    user_id: String,
    allow_sharing_character_configs: bool,
    allow_sharing_figure_records: bool,
    updated_at: DateTime<Utc>,
    version: i32,
}

#[derive(Debug)]
pub struct UserConfigsRepositoryImpl;

impl UserConfigsRepositoryImpl {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ports::UserConfigsRepository for UserConfigsRepositoryImpl {
    type Conn = PgConnection;
    type Error = anyhow::Error;

    async fn get(
        &mut self,
        conn: &mut Self::Conn,
        user_id: entities::UserId,
    ) -> Result<entities::UserConfig, Self::Error> {
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
        .fetch_optional(&mut *conn)
        .await?;

        let user_config = record
            .map(|record| -> anyhow::Result<_> {
                Ok(entities::UserConfig {
                    user_id: entities::UserId::from(record.user_id),
                    allow_sharing_character_configs: record.allow_sharing_character_configs,
                    allow_sharing_figure_records: record.allow_sharing_figure_records,
                    updated_at: Some(record.updated_at),
                    version: entities::Version::try_from(record.version)?,
                })
            })
            .unwrap_or_else(|| Ok(entities::UserConfig::default_config(user_id)))?;

        Ok(user_config)
    }

    async fn save(
        &mut self,
        conn: &mut Self::Conn,
        now: DateTime<Utc>,
        mut user_config: entities::UserConfig,
    ) -> Result<entities::UserConfig, Self::Error> {
        let mut trx = conn.begin().await?;
        let prev_version = user_config.version;
        user_config.version = user_config.version.next();
        user_config.updated_at = Some(now);

        if prev_version.is_none() {
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
                &String::from(user_config.user_id.clone()),
                user_config.allow_sharing_character_configs,
                user_config.allow_sharing_figure_records,
                user_config.updated_at,
                i32::from(user_config.version)
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
                i32::from(user_config.version),
                &String::from(user_config.user_id.clone()),
                i32::from(prev_version),
            )
            .execute(&mut *trx)
            .await
            .context("update character_config")?;

            if result.rows_affected() == 0 {
                return Err(anyhow!("conflict"));
            }
        }

        trx.commit().await?;

        Ok(user_config)
    }
}
