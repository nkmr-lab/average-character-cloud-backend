use derive_more::Into;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Into, Copy)]
pub struct SharedProportion(i32);

const MAX_SHARED_PROPORTION: i32 = 100;

#[derive(Error, Debug, Clone)]
pub enum SharedProportionTryFromError {
    #[error("Shared proportion must be less than {}", MAX_SHARED_PROPORTION)]
    TooLarge,
    #[error("Shared proportion must be non negative")]
    TooSmall,
}

impl TryFrom<i32> for SharedProportion {
    type Error = SharedProportionTryFromError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value > MAX_SHARED_PROPORTION {
            Err(SharedProportionTryFromError::TooLarge)
        } else if value < 0 {
            Err(SharedProportionTryFromError::TooSmall)
        } else {
            Ok(Self(value))
        }
    }
}

impl Default for SharedProportion {
    fn default() -> Self {
        SharedProportion(50)
    }
}
