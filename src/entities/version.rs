use derive_more::Into;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Into, Copy)]
pub struct Version(i32);

impl Version {
    pub fn new() -> Self {
        Self(1)
    }

    // 永続化していない値
    pub fn none() -> Self {
        Self(0)
    }

    pub fn is_none(&self) -> bool {
        self.0 == 0
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Error, Debug, Clone)]
pub enum VersionTryFromError {
    #[error("Version must be non negative")]
    NegativeInteger,
}

impl TryFrom<i32> for Version {
    type Error = VersionTryFromError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value >= 0 {
            Ok(Self(value))
        } else {
            Err(VersionTryFromError::NegativeInteger)
        }
    }
}
