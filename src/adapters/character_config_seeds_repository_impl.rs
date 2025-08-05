use crate::{entities, ports};
use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{Acquire, Postgres};

#[derive(Debug, Clone)]
pub struct CharacterConfigSeedModel {
    pub character: String,
    pub stroke_count: i32,
    pub ratio: i32,
    pub updated_at: DateTime<Utc>,
}

impl CharacterConfigSeedModel {
    pub fn into_entity(self) -> anyhow::Result<entities::CharacterConfigSeed> {
        let character = entities::Character::try_from(self.character.as_str())?;

        Ok(entities::CharacterConfigSeed {
            character,
            stroke_count: entities::StrokeCount::try_from(self.stroke_count)?,
            ratio: entities::Ratio::try_from(self.ratio)?,
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
                ROUND(AVG(stroke_count))::INTEGER AS stroke_count
            FROM
                character_configs
                INNER JOIN user_configs ON character_configs.user_id = user_configs.user_id
            WHERE
                user_configs.allow_sharing_character_configs
                AND
                character_configs.disabled = false
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
                INSERT INTO character_config_seeds (character, stroke_count, ratio, updated_at)
                VALUES ($1, $2, $3, $4)
                "#,
                row.character,
                row.stroke_count,
                100,
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
                    ratio,
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

    async fn query(
        &mut self,
        user_id: entities::UserId,
        after_id: Option<(entities::Character, entities::StrokeCount)>,
        before_id: Option<(entities::Character, entities::StrokeCount)>,
        limit: entities::Limit,
        include_exist_character_config: bool,
    ) -> Result<Vec<entities::CharacterConfigSeed>, Self::Error> {
        let mut conn = self.db.acquire().await?;

        let after_character = after_id
            .clone()
            .map(|(character, _)| String::from(character));
        let after_stroke_count = after_id.map(|(_, stroke_count)| i32::from(stroke_count));
        let before_character = before_id
            .clone()
            .map(|(character, _)| String::from(character));
        let before_stroke_count = before_id.map(|(_, stroke_count)| i32::from(stroke_count));
        let models = sqlx::query_as!(
            CharacterConfigSeedModel,
            r#"
            SELECT
                character,
                stroke_count,
                ratio,
                updated_at
            FROM
                character_config_seeds
            WHERE
                $8 OR NOT EXISTS (
                    SELECT
                        1
                    FROM
                        character_configs
                    WHERE
                        character_configs.user_id = $1
                        AND character_configs.character = character_config_seeds.character
                        AND character_configs.stroke_count = character_config_seeds.stroke_count
                        AND character_configs.disabled = false
                )
                AND
                ($2::VARCHAR(64) IS NULL OR (character, stroke_count) > ($2, $3))
                AND
                ($4::VARCHAR(64) IS NULL OR (character, stroke_count) < ($4, $5))
            ORDER BY
                CASE WHEN $6 = 0 THEN (character, stroke_count) END ASC,
                CASE WHEN $6 = 1 THEN (character, stroke_count) END DESC
            LIMIT $7
        "#,
            String::from(user_id.clone()),
            after_character,
            after_stroke_count,
            before_character,
            before_stroke_count,
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

    async fn get_by_ids(
        &mut self,
        keys: &[(entities::Character, entities::StrokeCount)],
    ) -> Result<Vec<entities::CharacterConfigSeed>, Self::Error> {
        let mut conn = self.db.acquire().await?;

        let character_values = keys
            .iter()
            .map(|(character, _)| String::from(character.clone()))
            .collect::<Vec<_>>();

        let stroke_count_values = keys
            .iter()
            .map(|(_, stroke_count)| i32::from(*stroke_count))
            .collect::<Vec<_>>();

        let models = sqlx::query_as!(
            CharacterConfigSeedModel,
            r#"
            WITH input_pairs AS (
                SELECT a, b
                FROM unnest($1::VARCHAR(8)[], $2::INTEGER[]) AS t(a, b)
            )
            SELECT
                character,
                stroke_count,
                ratio,
                updated_at
            FROM
                character_config_seeds
            JOIN
                input_pairs ON character_config_seeds.character = input_pairs.a
                AND character_config_seeds.stroke_count = input_pairs.b
        "#,
            character_values.as_slice(),
            stroke_count_values.as_slice()
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
}
