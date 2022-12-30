use derive_more::Into;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Into, Copy)]
pub struct StrokeCount(i32);

const MAX_STROKE_COUNT: i32 = 1000;

#[derive(Error, Debug, Clone)]
pub enum StrokeCountTryFromError {
    #[error("Stroke count must be less than {}", MAX_STROKE_COUNT)]
    TooLarge,
    #[error("Stroke count must be non negative")]
    TooSmall,
}

impl TryFrom<i32> for StrokeCount {
    type Error = StrokeCountTryFromError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value > MAX_STROKE_COUNT {
            Err(StrokeCountTryFromError::TooLarge)
        } else if value < 0 {
            Err(StrokeCountTryFromError::TooSmall)
        } else {
            Ok(Self(value))
        }
    }
}
