use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum LimitKind {
    First,
    Last,
}

const MAX_LIMIT: i32 = 100;
#[derive(Error, Debug, Clone)]

pub enum CreateLimitError {
    #[error("Limit must be less than or equal to {}", MAX_LIMIT)]
    LimitTooLarge,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Limit {
    kind: LimitKind,
    value: i32,
}

impl Limit {
    pub fn new(kind: LimitKind, value: i32) -> Result<Self, CreateLimitError> {
        if value > MAX_LIMIT {
            Err(CreateLimitError::LimitTooLarge)
        } else {
            Ok(Self { kind, value })
        }
    }

    pub fn kind(&self) -> LimitKind {
        self.kind
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}
