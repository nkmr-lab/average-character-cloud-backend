use crate::{entities, ports};
use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use sqlx::{Acquire, Postgres};

#[derive(Debug, Clone)]
pub struct CharacterConfigModel {
    pub user_id: String,
    pub character: String,
    pub stroke_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
    pub ratio: i32,
}

impl CharacterConfigModel {
    pub fn into_entity(self) -> anyhow::Result<entities::CharacterConfig> {
        let character = entities::Character::try_from(self.character.as_str())?;

        Ok(entities::CharacterConfig {
            user_id: entities::UserId::from(self.user_id),
            character,
            stroke_count: entities::StrokeCount::try_from(self.stroke_count)?,
            created_at: self.created_at,
            updated_at: self.updated_at,
            version: entities::Version::try_from(self.version)?,
            ratio: entities::Ratio::try_from(self.ratio)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CharacterConfigsRepositoryImpl<A> {
    db: A,
}

impl<A> CharacterConfigsRepositoryImpl<A> {
    pub fn new(db: A) -> Self {
        Self { db }
    }
}

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
        ratio: entities::Ratio,
    ) -> Result<Result<entities::CharacterConfig, ports::CreateError>, Self::Error> {
        let mut trx = self.db.begin().await?;
        let character_config = entities::CharacterConfig {
            user_id,
            character,
            created_at: now,
            updated_at: now,
            stroke_count,
            version: entities::Version::new(),
            ratio,
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
                AND
                ratio = $3
        "#,
            String::from(character_config.user_id.clone()),
            String::from(character_config.character.clone()),
            i32::from(character_config.ratio)
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
                INSERT INTO character_configs (user_id, character, created_at, updated_at, stroke_count, ratio, version)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            String::from(character_config.user_id.clone()),
            String::from(character_config.character.clone()),
            character_config.created_at,
            character_config.updated_at,
            i32::from(character_config.stroke_count),
            i32::from(character_config.ratio),
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
        ratio: Option<entities::Ratio>,
    ) -> Result<entities::CharacterConfig, Self::Error> {
        let mut trx = self.db.begin().await?;
        let prev_version = character_config.version;

        character_config.version = character_config.version.next();
        character_config.updated_at = now;
        if let Some(ratio) = ratio {
            character_config.ratio = ratio;
        }

        let result = sqlx::query!(
            r#"
            UPDATE character_configs
                SET
                    updated_at = $1,
                    ratio = $2,
                    version = $3
                WHERE
                    user_id = $4
                    AND
                    character = $5
                    AND
                    stroke_count = $6
                    AND
                    version = $7
            "#,
            &character_config.updated_at,
            i32::from(character_config.ratio),
            i32::from(character_config.version),
            String::from(character_config.user_id.clone()),
            String::from(character_config.character.clone()),
            i32::from(character_config.stroke_count),
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

    async fn get_by_characters(
        &mut self,
        characters: &[entities::Character],
        user_id: entities::UserId,
    ) -> Result<Vec<entities::CharacterConfig>, Self::Error> {
        let mut conn = self.db.acquire().await?;
        let character_values = characters
            .iter()
            .map(|character| String::from(character.clone()))
            .collect::<Vec<_>>();

        let models = sqlx::query_as!(
            CharacterConfigModel,
            r#"
                SELECT
                    user_id,
                    character,
                    stroke_count,
                    ratio,
                    created_at,
                    updated_at,
                    version
                FROM
                    character_configs
                WHERE
                    user_id = $1
                    AND
                    character = Any($2)
            "#,
            String::from(user_id.clone()),
            character_values.as_slice(),
        )
        .fetch_all(&mut *conn)
        .await
        .context("fetch character_configs")?;

        let character_configs = models
            .into_iter()
            .map(|model| model.into_entity())
            .collect::<anyhow::Result<Vec<_>>>()
            .context("convert CharacterConfig")?;

        Ok(character_configs)
    }

    async fn query(
        &mut self,
        user_id: entities::UserId,
        after_id: Option<(entities::Character, entities::StrokeCount)>,
        before_id: Option<(entities::Character, entities::StrokeCount)>,
        limit: entities::Limit,
    ) -> Result<Vec<entities::CharacterConfig>, Self::Error> {
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
            CharacterConfigModel,
            r#"
            SELECT
                user_id,
                character,
                stroke_count,
                ratio,
                created_at,
                updated_at,
                version
            FROM
                character_configs
            WHERE
                user_id = $1
                AND
                ($2::VARCHAR(64) IS NULL OR (character, stroke_count) > ($2, $3))
                AND
                ($4::VARCHAR(64) IS NULL OR (character, stroke_count) < ($4, $5))
            ORDER BY
                CASE WHEN $6 = 0 THEN (character, ratio) END ASC,
                CASE WHEN $6 = 1 THEN (character, ratio) END DESC
            LIMIT $7
        "#,
            String::from(user_id.clone()),
            after_character,
            after_stroke_count,
            before_character,
            before_stroke_count,
            i32::from(limit.kind() == entities::LimitKind::Last),
            i64::from(limit.value()),
        )
        .fetch_all(&mut *conn)
        .await
        .context("fetch character_configs")?;

        let character_configs = models
            .into_iter()
            .map(|row| row.into_entity())
            .collect::<anyhow::Result<Vec<_>>>()
            .context("convert CharacterConfig")?;

        Ok(character_configs)
    }

    async fn get_by_ids(
        &mut self,
        user_id: entities::UserId,
        keys: &[(entities::Character, entities::StrokeCount)],
    ) -> Result<Vec<entities::CharacterConfig>, Self::Error> {
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
            CharacterConfigModel,
            r#"
            WITH input_pairs AS (
                SELECT a, b
                FROM unnest($1::VARCHAR(8)[], $2::INTEGER[]) AS t(a, b)
            )
            SELECT
                user_id,
                character,
                stroke_count,
                ratio,
                created_at,
                updated_at,
                version
            FROM
                character_configs
            JOIN
                input_pairs ON character_configs.character = input_pairs.a
                AND character_configs.stroke_count = input_pairs.b
            WHERE
                user_id = $3
        "#,
            character_values.as_slice(),
            stroke_count_values.as_slice(),
            String::from(user_id.clone()),
        )
        .fetch_all(&mut *conn)
        .await
        .context("fetch character_config")?;

        let character_configs = models
            .into_iter()
            .map(|row| row.into_entity())
            .collect::<anyhow::Result<Vec<_>>>()
            .context("convert CharacterConfig")?;

        Ok(character_configs)
    }
}
