use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum WritingModeTryFromError {
    #[error("Invalid writing mode value: {0}")]
    InvalidValue(i32),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Copy)]
pub enum WritingMode {
    Horizontal,
    Vertical,
}

impl TryFrom<i32> for WritingMode {
    type Error = WritingModeTryFromError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Horizontal),
            1 => Ok(Self::Vertical),
            _ => Err(WritingModeTryFromError::InvalidValue(value)),
        }
    }
}

impl Into<i32> for WritingMode {
    fn into(self) -> i32 {
        match self {
            Self::Horizontal => 0,
            Self::Vertical => 1,
        }
    }
}
