use super::{UserId, Version};
use chrono::{DateTime, Utc};
use derive_more::{From, Into};
use thiserror::Error;
use ulid::Ulid;

#[derive(Clone, Debug, Into, From, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct FileId(Ulid);

#[derive(Clone, Debug)]
pub struct MimeType {
    value: String,
    extension: String,
}

impl MimeType {
    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn extension(&self) -> &str {
        &self.extension
    }
}

#[derive(Error, Debug, Clone)]
pub enum MimeTypeTryFromError {
    #[error("Unsupported MIME type: {0}")]
    Unsupported(String),
}

impl TryFrom<String> for MimeType {
    type Error = MimeTypeTryFromError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "image/png" => Ok(Self {
                value: value.clone(),
                extension: "png".to_string(),
            }),
            "image/jpeg" => Ok(Self {
                value: value.clone(),
                extension: "jpg".to_string(),
            }),
            "image/gif" => Ok(Self {
                value: value.clone(),
                extension: "gif".to_string(),
            }),
            "image/webp" => Ok(Self {
                value: value.clone(),
                extension: "webp".to_string(),
            }),
            _ => Err(MimeTypeTryFromError::Unsupported(value)),
        }
    }
}

// 20 MB
const MAX_FILE_SIZE: i32 = 20 * 1024 * 1024;

#[derive(Error, Debug, Clone)]
pub enum FileSizeTryFromError {
    #[error("File size must be non-negative")]
    NegativeSize,
    #[error("File size exceeds maximum limit")]
    TooLarge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Into)]
pub struct FileSize(i32);

impl TryFrom<i32> for FileSize {
    type Error = FileSizeTryFromError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value < 0 {
            Err(FileSizeTryFromError::NegativeSize)
        } else if value > MAX_FILE_SIZE {
            Err(FileSizeTryFromError::TooLarge)
        } else {
            Ok(Self(value))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Into)]
pub struct FileKey(String);

impl FileKey {
    pub fn new(id: FileId, mime_type: &MimeType) -> Self {
        let key = format!("{}.{}", id.0, mime_type.extension());
        Self(key)
    }

    // for repository
    pub fn from_unchecked(key: String) -> Self {
        Self(key)
    }
}

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
