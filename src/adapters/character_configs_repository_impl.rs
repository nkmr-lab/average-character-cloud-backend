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

    async fn get(
        &mut self,
        user_id: entities::UserId,
        after_character: Option<entities::Character>,
        before_character: Option<entities::Character>,
        limit: entities::Limit,
    ) -> Result<Vec<entities::CharacterConfig>, Self::Error> {
        let mut conn = self.db.acquire().await?;

        let models = sqlx::query_as!(
            CharacterConfigModel,
            r#"
            SELECT
                user_id,
                character,
                stroke_count,
                created_at,
                updated_at,
                version
            FROM
                character_configs
            WHERE
                user_id = $1
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
}
