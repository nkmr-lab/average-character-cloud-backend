use crate::entities;
use chrono::{DateTime, Utc};

pub trait FilesRepository {
    type Error;

    async fn create(
        &mut self,
        user_id: entities::UserId,
        now: DateTime<Utc>,
        mime_type: entities::MimeType,
        size: entities::FileSize,
    ) -> Result<entities::File, Self::Error>;

    async fn verified(
        &mut self,
        now: DateTime<Utc>,
        file: entities::File,
    ) -> Result<entities::File, Self::Error>;

    async fn get_by_ids(
        &mut self,
        user_id: entities::UserId,
        ids: &[entities::FileId],
        verified_only: bool,
    ) -> Result<Vec<entities::File>, Self::Error>;
}
