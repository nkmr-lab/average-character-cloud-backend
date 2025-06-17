use derive_more::Into;
use thiserror::Error;

/*
あるcharacterのstroke_countの採用率
相対的な値で、{character='a', stroke_count=10, ratio=10}, {character='a', stroke_count=2, ratio=10}があった場合、どちらも50%の採用率となる。
全て0のときは全て1と同じ動作をする。
*/
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Into, Copy)]
pub struct Ratio(i32);

const MAX_RATIO: i32 = 100;

#[derive(Error, Debug, Clone)]
pub enum RatioTryFromError {
    #[error("Stroke count must be less than {}", MAX_RATIO)]
    TooLarge,
    #[error("Stroke count must be non negative")]
    TooSmall,
}

impl TryFrom<i32> for Ratio {
    type Error = RatioTryFromError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value > MAX_RATIO {
            Err(RatioTryFromError::TooLarge)
        } else if value < 0 {
            Err(RatioTryFromError::TooSmall)
        } else {
            Ok(Self(value))
        }
    }
}
