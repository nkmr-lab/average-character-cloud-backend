use derive_more::Into;
use thiserror::Error;
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Into, Copy)]
pub struct FontSize(i32);

const MAX_FONT_SIZE: i32 = 256;
const MIN_FONT_SIZE: i32 = 1;

#[derive(Error, Debug, Clone)]
pub enum FontSizeTryFromError {
    #[error("Font size must be less than or equal to {}", MAX_FONT_SIZE)]
    TooLarge,
    #[error("Font size must be greater than or equal to {}", MIN_FONT_SIZE)]
    TooSmall,
}

impl TryFrom<i32> for FontSize {
    type Error = FontSizeTryFromError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value > MAX_FONT_SIZE {
            Err(FontSizeTryFromError::TooLarge)
        } else if value < MIN_FONT_SIZE {
            Err(FontSizeTryFromError::TooSmall)
        } else {
            Ok(Self(value))
        }
    }
}
