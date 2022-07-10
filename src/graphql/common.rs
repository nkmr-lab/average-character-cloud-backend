use std::future::Future;
use std::str::FromStr;

use juniper::FieldResult;
use juniper::ID;

use thiserror::Error;
use ulid::Ulid;

use crate::entities::{self};
use anyhow::anyhow;

use std::fmt;

#[derive(Debug, Error)]
pub struct GraphqlUserError {
    #[source]
    pub source: anyhow::Error,
}

impl fmt::Display for GraphqlUserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl From<anyhow::Error> for GraphqlUserError {
    fn from(source: anyhow::Error) -> Self {
        Self { source }
    }
}

impl From<&str> for GraphqlUserError {
    fn from(source: &str) -> Self {
        Self {
            source: anyhow!("{}", source),
        }
    }
}

pub async fn handler<T, Fut: Future<Output = anyhow::Result<T>>>(
    f: impl FnOnce() -> Fut,
) -> FieldResult<T> {
    match f().await {
        Ok(value) => Ok(value),
        Err(err) => Err(match err.downcast_ref::<GraphqlUserError>() {
            Some(err) => err.source.to_string().into(),
            None => {
                log::error!("{:?}", err);
                "Internal error".into()
            }
        }),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LimitKind {
    First,
    Last,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Limit {
    pub kind: LimitKind,
    pub value: i32,
}

impl Limit {
    pub fn encode(first: Option<i32>, last: Option<i32>) -> anyhow::Result<Self> {
        let max = 100;

        match (first, last) {
            (Some(first), None) => {
                if first < 0 {
                    Err(GraphqlUserError::from("first must be greater than or equal to 0").into())
                } else if first > max {
                    Err(GraphqlUserError::from(
                        format!("first must be less than or equal to {}", max).as_str(),
                    )
                    .into())
                } else {
                    Ok(Limit {
                        kind: LimitKind::First,
                        value: first,
                    })
                }
            }
            (None, Some(last)) => {
                if last < 0 {
                    Err(GraphqlUserError::from("last must be greater than or equal to 0").into())
                } else if last > max {
                    Err(GraphqlUserError::from(
                        format!("last must be less than or equal to {}", max).as_str(),
                    )
                    .into())
                } else {
                    Ok(Limit {
                        kind: LimitKind::Last,
                        value: last,
                    })
                }
            }
            _ => Err(GraphqlUserError::from("Must provide either first or last, not both").into()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeID {
    FigureRecord(Ulid),
    CharacterConfig(Ulid),
    Character(entities::character::Character),
}

impl NodeID {
    pub fn to_id(&self) -> ID {
        match self {
            NodeID::FigureRecord(id) => ID::new(base64::encode(format!("FigureRecord:{}", id))),
            NodeID::CharacterConfig(id) => {
                ID::new(base64::encode(format!("CharacterConfig:{}", id)))
            }
            NodeID::Character(character) => ID::new(base64::encode(format!(
                "Character:{}",
                String::from(character.clone())
            ))),
        }
    }

    pub fn from_id(id: &ID) -> Option<NodeID> {
        let id = id.to_string();
        let buf = base64::decode(id).ok()?;
        let s = String::from_utf8(buf).ok()?;

        if let Some(record_id) = s.strip_prefix("FigureRecord:") {
            let ulid = Ulid::from_str(record_id).ok()?;
            Some(NodeID::FigureRecord(ulid))
        } else if let Some(character_id) = s.strip_prefix("CharacterConfig:") {
            let ulid = Ulid::from_str(character_id).ok()?;
            Some(NodeID::CharacterConfig(ulid))
        } else if let Some(character) = s.strip_prefix("Character:") {
            Some(NodeID::Character(
                entities::character::Character::try_from(character).ok()?,
            ))
        } else {
            None
        }
    }
}