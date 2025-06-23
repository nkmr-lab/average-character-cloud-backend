use derive_more::Into;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Into, Copy)]
pub struct Spacing(i32);

const MAX_SPACING: i32 = 64;
const MIN_SPACING: i32 = -MAX_SPACING;

#[derive(Error, Debug, Clone)]
pub enum SpacingTryFromError {
    #[error("Spacing must be less than or equal to {}", MAX_SPACING)]
    TooLarge,
    #[error("Spacing must be greater than or equal to {}", MIN_SPACING)]
    TooSmall,
}

impl TryFrom<i32> for Spacing {
    type Error = SpacingTryFromError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value > MAX_SPACING {
            Err(SpacingTryFromError::TooLarge)
        } else if value < MIN_SPACING {
            Err(SpacingTryFromError::TooSmall)
        } else {
            Ok(Self(value))
        }
    }
}
