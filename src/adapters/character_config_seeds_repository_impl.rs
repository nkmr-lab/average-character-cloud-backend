use crate::{entities, ports};
use anyhow::Context;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Acquire, Postgres};

#[derive(Debug, Clone)]
pub struct CharacterConfigSeedsRepositoryImpl<A> {
    db: A,
}

impl<A> CharacterConfigSeedsRepositoryImpl<A> {
    pub fn new(db: A) -> Self {
        Self { db }
    }
}

#[async_trait]
impl<A> ports::CharacterConfigSeedsRepository for CharacterConfigSeedsRepositoryImpl<A>
where
    A: Send,
    for<'c> &'c A: Acquire<'c, Database = Postgres>,
{
    type Error = anyhow::Error;

    async fn update_seeds(&mut self, now: DateTime<Utc>) -> Result<(), Self::Error> {
        let mut trx = self.db.begin().await?;
        let result = sqlx::query!(
            r#"
            SELECT
                character,
                AVG(stroke_count)::INTEGER as avg_stroke_count
            FROM
                character_configs
                INNER JOIN user_configs ON character_configs.user_id = user_configs.user_id
            WHERE
                user_configs.allow_sharing_character_configs
            GROUP BY
                character
            "#,
        )
        .fetch_all(&mut *trx)
        .await?;

        sqlx::query!("DELETE FROM character_config_seeds")
            .execute(&mut *trx)
            .await?;

        for row in result {
            sqlx::query!(
                r#"
                INSERT INTO character_config_seeds (character, stroke_count, updated_at)
                VALUES ($1, $2, $3)
                "#,
                row.character,
                row.avg_stroke_count
                    .ok_or_else(|| anyhow::anyhow!("avg_stroke_count is null"))?,
                now,
            )
            .execute(&mut *trx)
            .await?;
        }

        trx.commit().await?;
        Ok(())
    }
}
