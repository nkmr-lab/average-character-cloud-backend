use anyhow::{anyhow, Context};
use sqlx::{Acquire, Postgres};

use crate::{entities, ports};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
struct UserConfigModel {
    user_id: String,
    allow_sharing_character_configs: bool,
    allow_sharing_figure_records: bool,
    updated_at: DateTime<Utc>,
    version: i32,
}

#[derive(Debug, Clone)]
pub struct UserConfigsRepositoryImpl<A> {
    db: A,
}

impl<A> UserConfigsRepositoryImpl<A> {
    pub fn new(db: A) -> Self {
        Self { db }
    }
}


impl<A> ports::UserConfigsRepository for UserConfigsRepositoryImpl<A>
where
    A: Send,
    for<'c> &'c A: Acquire<'c, Database = Postgres>,
{
    type Error = anyhow::Error;

    async fn get(
        &mut self,
        user_id: entities::UserId,
    ) -> Result<entities::UserConfig, Self::Error> {
        let mut conn = self.db.acquire().await?;
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
        now: DateTime<Utc>,
        mut user_config: entities::UserConfig,
    ) -> Result<entities::UserConfig, Self::Error> {
        let mut trx = self.db.begin().await?;
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

#[cfg(test)]
mod tests {
    use chrono::Timelike;

    use super::*;
    use crate::ports::UserConfigsRepository;

    #[sqlx::test]
    async fn test_user_configs_repository(pool: sqlx::PgPool) {
        let mut repo = UserConfigsRepositoryImpl::new(pool);
        let user_id = entities::UserId::from("test_user".to_string());

        let mut config = repo.get(user_id.clone()).await.unwrap();
        assert_eq!(config.user_id, user_id);
        assert!(!config.allow_sharing_character_configs);
        assert!(!config.allow_sharing_figure_records);
        assert!(config.updated_at.is_none());
        assert_eq!(config.version, entities::Version::none());

        // TODO: DBと時刻の精度が違う
        let now = Utc::now().with_nanosecond(0).unwrap();

        config.allow_sharing_character_configs = true;
        config.allow_sharing_figure_records = true;
        let saved_config = repo.save(now, config.clone()).await.unwrap();
        assert_eq!(saved_config.user_id, user_id);
        assert!(saved_config.allow_sharing_character_configs);
        assert!(saved_config.allow_sharing_figure_records);
        assert_eq!(saved_config.updated_at, Some(now));
        assert_eq!(saved_config.version, config.version.next());

        let fetched_config = repo.get(user_id).await.unwrap();
        assert_eq!(fetched_config, saved_config);
    }
}
