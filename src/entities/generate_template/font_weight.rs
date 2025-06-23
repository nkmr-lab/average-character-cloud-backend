use derive_more::Into;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Into, Copy)]
pub struct FontWeight(i32);

const MAX_FONT_WEIGHT: i32 = 100;
const MIN_FONT_WEIGHT: i32 = 1;

#[derive(Error, Debug, Clone)]
pub enum FontWeightTryFromError {
    #[error("Font weight must be less than or equal to {}", MAX_FONT_WEIGHT)]
    TooLarge,
    #[error("Font weight must be greater than or equal to {}", MIN_FONT_WEIGHT)]
    TooSmall,
}
impl TryFrom<i32> for FontWeight {
    type Error = FontWeightTryFromError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value > MAX_FONT_WEIGHT {
            Err(FontWeightTryFromError::TooLarge)
        } else if value < MIN_FONT_WEIGHT {
            Err(FontWeightTryFromError::TooSmall)
        } else {
            Ok(Self(value))
        }
    }
}
