use derive_more::Into;
use thiserror::Error;

// 50 MB
const MAX_FILE_SIZE: i32 = 50 * 1024 * 1024;

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
