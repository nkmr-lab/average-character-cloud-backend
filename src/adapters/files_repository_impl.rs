use std::str::FromStr;

use crate::{entities, ports};
use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use sqlx::{Acquire, Postgres};
use ulid::Ulid;

#[derive(Debug, Clone)]
pub struct FileModel {
    pub id: String,
    pub user_id: String,
    pub key: String,
    pub mime_type: String,
    pub size: i32,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
}

impl FileModel {
    pub fn into_entity(self) -> anyhow::Result<entities::File> {
        let id = Ulid::from_str(&self.id).context("ulid decode error")?;

        Ok(entities::File {
            id: entities::FileId::from(id),
            user_id: entities::UserId::from(self.user_id),
            key: entities::FileKey::from_unchecked(self.key),
            mime_type: entities::MimeType::try_from(self.mime_type)?,
            size: entities::FileSize::try_from(self.size)?,
            verified: self.verified,
            created_at: self.created_at,
            updated_at: self.updated_at,
            version: entities::Version::try_from(self.version)
                .context("version conversion error")?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FilesRepositoryImpl<A> {
    db: A,
}

impl<A> FilesRepositoryImpl<A> {
    pub fn new(db: A) -> Self {
        Self { db }
    }
}

impl<A> ports::FilesRepository for FilesRepositoryImpl<A>
where
    A: Send,
    for<'c> &'c A: Acquire<'c, Database = Postgres>,
{
    type Error = anyhow::Error;

    async fn create(
        &mut self,
        user_id: entities::UserId,
        now: DateTime<Utc>,
        mime_type: entities::MimeType,
        size: entities::FileSize,
    ) -> Result<entities::File, Self::Error> {
        let mut trx = self.db.begin().await?;
        let id = entities::FileId::from(Ulid::from_datetime(now));
        let key = entities::FileKey::new(id, &mime_type);
        let file = entities::File {
            id,
            user_id,
            key,
            mime_type,
            size,
            verified: false,
            created_at: now,
            updated_at: now,
            version: entities::Version::new(),
        };

        sqlx::query!(
            r#"
                INSERT INTO files (id, user_id, key, mime_type, size, verified, created_at, updated_at, version)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            Ulid::from(file.id).to_string(),
            String::from(file.user_id.clone()),
            String::from(file.key.clone()),
            file.mime_type.value(),
            i32::from(file.size.clone()),
            file.verified,
            file.created_at,
            file.updated_at,
            i32::from(file.version.clone()),
        )
        .execute(&mut *trx)
        .await
        .context("fetch file")?;

        trx.commit().await?;
        Ok(file)
    }

    async fn verified(
        &mut self,
        now: DateTime<Utc>,
        mut file: entities::File,
    ) -> Result<entities::File, Self::Error> {
        let mut trx = self.db.begin().await?;
        let prev_version = file.version;

        file.version = file.version.next();
        file.updated_at = now;
        file.verified = true;

        let result = sqlx::query!(
            r#"
            UPDATE files
                SET
                    verified = $1,
                    updated_at = $2,
                    version = $3
                WHERE
                    user_id = $4
                    AND
                    id = $5
                    AND
                    version = $6
            "#,
            file.verified,
            file.updated_at,
            i32::from(file.version),
            String::from(file.user_id.clone()),
            Ulid::from(file.id.clone()).to_string(),
            i32::from(prev_version),
        )
        .execute(&mut *trx)
        .await
        .context("update file")?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("conflict"));
        }

        trx.commit().await?;
        Ok(file)
    }

    async fn get_by_ids(
        &mut self,
        user_id: entities::UserId,
        ids: &[entities::FileId],
        verified_only: bool,
    ) -> Result<Vec<entities::File>, Self::Error> {
        let mut conn = self.db.acquire().await?;
        let ids = ids
            .iter()
            .map(|&id| Ulid::from(id).to_string())
            .collect::<Vec<_>>();

        let models = sqlx::query_as!(
            FileModel,
            r#"
                SELECT
                    id,
                    user_id,
                    key,
                    mime_type,
                    size,
                    verified,
                    created_at,
                    updated_at,
                    version
                FROM
                    files
                WHERE
                    id = Any($1)
                    AND user_id = $2
                    AND ((NOT $3) OR verified = true)
            "#,
            ids.as_slice(),
            String::from(user_id.clone()),
            verified_only,
        )
        .fetch_all(&mut *conn)
        .await
        .context("fetch file")?;

        let files = models
            .into_iter()
            .map(|model| model.into_entity())
            .collect::<anyhow::Result<Vec<_>>>()
            .context("convert File")?;

        Ok(files)
    }
}
