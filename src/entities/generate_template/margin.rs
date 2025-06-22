use derive_more::Into;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Into, Copy)]
pub struct Margin(i32);

const MAX_MARGIN: i32 = 256;
const MIN_MARGIN: i32 = -MAX_MARGIN;

#[derive(Error, Debug, Clone)]
pub enum MarginTryFromError {
    #[error("Margin must be less than or equal to {}", MAX_MARGIN)]
    TooLarge,
    #[error("Margin must be greater than or equal to {}", MIN_MARGIN)]
    TooSmall,
}

impl TryFrom<i32> for Margin {
    type Error = MarginTryFromError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value > MAX_MARGIN {
            Err(MarginTryFromError::TooLarge)
        } else if value < MIN_MARGIN {
            Err(MarginTryFromError::TooSmall)
        } else {
            Ok(Self(value))
        }
    }
}
