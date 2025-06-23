use super::{FileId, MimeType};
use derive_more::Into;
use ulid::Ulid;

#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Into)]
pub struct FileKey(String);

impl FileKey {
    pub fn new(id: FileId, mime_type: &MimeType) -> Self {
        let key = format!("{}.{}", Ulid::from(id), mime_type.extension());
        Self(key)
    }

    // for repository
    pub fn from_unchecked(key: String) -> Self {
        Self(key)
    }
}
