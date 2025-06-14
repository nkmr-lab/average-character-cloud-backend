use crate::{entities, ports};
use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{Acquire, Postgres};

#[derive(Debug, Clone)]
pub struct CharacterConfigSeedModel {
    pub character: String,
    pub stroke_count: i32,
    pub updated_at: DateTime<Utc>,
}

impl CharacterConfigSeedModel {
    pub fn into_entity(self) -> anyhow::Result<entities::CharacterConfigSeed> {
        let character = entities::Character::try_from(self.character.as_str())?;

        Ok(entities::CharacterConfigSeed {
            character,
            stroke_count: entities::StrokeCount::try_from(self.stroke_count)?,
            updated_at: self.updated_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CharacterConfigSeedsRepositoryImpl<A> {
    db: A,
}

impl<A> CharacterConfigSeedsRepositoryImpl<A> {
    pub fn new(db: A) -> Self {
        Self { db }
    }
}


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

    async fn get_by_characters(
        &mut self,
        characters: &[entities::Character],
    ) -> Result<Vec<entities::CharacterConfigSeed>, Self::Error> {
        let character_values = characters
            .iter()
            .map(|c| String::from(c.clone()))
            .collect::<Vec<_>>();
        let mut conn = self.db.acquire().await?;
        let models = sqlx::query_as!(
            CharacterConfigSeedModel,
            r#"
                SELECT
                    character,
                    stroke_count,
                    updated_at
                FROM
                    character_config_seeds
                WHERE
                    character = Any($1)
            "#,
            character_values.as_slice(),
        )
        .fetch_all(&mut *conn)
        .await
        .context("fetch character_config_seeds")?;

        let character_config_seeds = models
            .into_iter()
            .map(|model| model.into_entity())
            .collect::<anyhow::Result<Vec<_>>>()
            .context("convert CharacterConfigSeed")?;
        Ok(character_config_seeds)
    }

    async fn get(
        &mut self,
        user_id: entities::UserId,
        after_character: Option<entities::Character>,
        before_character: Option<entities::Character>,
        limit: entities::Limit,
        include_exist_character_config: bool,
    ) -> Result<Vec<entities::CharacterConfigSeed>, Self::Error> {
        let mut conn = self.db.acquire().await?;
        let models = sqlx::query_as!(
            CharacterConfigSeedModel,
            r#"
            SELECT
                character,
                stroke_count,
                updated_at
            FROM
                character_config_seeds
            WHERE
                $6 OR NOT EXISTS (
                    SELECT
                        1
                    FROM
                        character_configs
                    WHERE
                        character_configs.user_id = $1
                        AND character_configs.character = character_config_seeds.character
                )
                AND
                ($2::VARCHAR(64) IS NULL OR character > $2)
                AND
                ($3::VARCHAR(64) IS NULL OR character < $3)
            ORDER BY
                CASE WHEN $4 = 0 THEN character END ASC,
                CASE WHEN $4 = 1 THEN character END DESC
            LIMIT $5
        "#,
            String::from(user_id.clone()),
            after_character.map(String::from),
            before_character.map(String::from),
            i32::from(limit.kind() == entities::LimitKind::Last),
            i64::from(limit.value()),
            include_exist_character_config,
        )
        .fetch_all(&mut *conn)
        .await
        .context("fetch character_config_seeds")?;

        let character_config_seeds = models
            .into_iter()
            .map(|row| row.into_entity())
            .collect::<anyhow::Result<Vec<_>>>()
            .context("convert CharacterConfigSeed")?;

        Ok(character_config_seeds)
    }
}
