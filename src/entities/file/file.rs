use super::{FileId, FileKey, FileSize, MimeType};
use crate::entities::{UserId, Version};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct File {
    pub id: FileId,
    pub user_id: UserId,
    pub key: FileKey,
    pub mime_type: MimeType,
    pub size: FileSize,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: Version,
}
