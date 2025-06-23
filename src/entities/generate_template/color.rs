use derive_more::Into;
use thiserror::Error;
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Into, Copy)]
pub struct Color(i32);

const MAX_COLOR_VALUE: i32 = 0xFFFFFF;
const MIN_COLOR_VALUE: i32 = 0x000000;

#[derive(Error, Debug, Clone)]
pub enum ColorTryFromError {
    #[error("Color value must be less than or equal to {}", MAX_COLOR_VALUE)]
    TooLarge,
    #[error("Color value must be greater than or equal to {}", MIN_COLOR_VALUE)]
    TooSmall,
}

impl TryFrom<i32> for Color {
    type Error = ColorTryFromError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value > MAX_COLOR_VALUE {
            Err(ColorTryFromError::TooLarge)
        } else if value < MIN_COLOR_VALUE {
            Err(ColorTryFromError::TooSmall)
        } else {
            Ok(Self(value))
        }
    }
}
