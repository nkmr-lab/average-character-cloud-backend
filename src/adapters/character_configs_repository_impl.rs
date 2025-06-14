use crate::{entities, ports};
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Acquire, Postgres};

#[derive(Debug, Clone)]
pub struct CharacterConfigsRepositoryImpl<A> {
    db: A,
}

impl<A> CharacterConfigsRepositoryImpl<A> {
    pub fn new(db: A) -> Self {
        Self { db }
    }
}

#[async_trait]
impl<A> ports::CharacterConfigsRepository for CharacterConfigsRepositoryImpl<A>
where
    A: Send,
    for<'c> &'c A: Acquire<'c, Database = Postgres>,
{
    type Error = anyhow::Error;

    async fn create(
        &mut self,
        user_id: entities::UserId,
        now: DateTime<Utc>,
        character: entities::Character,
        stroke_count: entities::StrokeCount,
    ) -> Result<Result<entities::CharacterConfig, ports::CreateError>, Self::Error> {
        let mut trx = self.db.begin().await?;
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
            return Ok(Err(ports::CreateError::AlreadyExists));
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

    async fn update(
        &mut self,
        now: DateTime<Utc>,
        mut character_config: entities::CharacterConfig,
        stroke_count: Option<entities::StrokeCount>,
    ) -> Result<entities::CharacterConfig, Self::Error> {
        let mut trx = self.db.begin().await?;
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
}
