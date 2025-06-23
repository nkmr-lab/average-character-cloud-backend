use derive_more::Into;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Into, Copy)]
pub struct RandomLevel(i32);

const MAX_RANDOM_LEVEL: i32 = 100;

#[derive(Error, Debug, Clone)]
pub enum RandomLevelTryFromError {
    #[error("Random level must be less than {}", MAX_RANDOM_LEVEL)]
    TooLarge,
    #[error("Random level must be non negative")]
    TooSmall,
}

impl TryFrom<i32> for RandomLevel {
    type Error = RandomLevelTryFromError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value > MAX_RANDOM_LEVEL {
            Err(RandomLevelTryFromError::TooLarge)
        } else if value < 0 {
            Err(RandomLevelTryFromError::TooSmall)
        } else {
            Ok(Self(value))
        }
    }
}

impl Default for RandomLevel {
    fn default() -> Self {
        RandomLevel(50)
    }
}
