use std::str::FromStr;

use juniper::GraphQLObject;
use juniper::ID;

use thiserror::Error;
use ulid::Ulid;

use crate::entities::{self};
use anyhow::anyhow;

use crate::values::{Limit, LimitKind};
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

#[derive(Debug)]
pub struct ApiError(pub anyhow::Error);

impl<S: juniper::ScalarValue> juniper::IntoFieldError<S> for ApiError {
    fn into_field_error(self) -> juniper::FieldError<S> {
        match self.0.downcast_ref::<GraphqlUserError>() {
            Some(err) => err.source.to_string().into(),
            None => {
                tracing::error!("{:?}", self.0);
                "Internal error".into()
            }
        }
    }
}

impl<T: Into<anyhow::Error>> From<T> for ApiError {
    fn from(err: T) -> Self {
        Self(err.into())
    }
}

pub fn encode_limit(first: Option<i32>, last: Option<i32>) -> anyhow::Result<Limit> {
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

#[derive(Debug, Clone)]
pub enum NodeID {
    FigureRecord(Ulid),
    CharacterConfig(Ulid),
    Character(entities::Character),
    UserConfig(String),
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
            NodeID::UserConfig(id) => ID::new(base64::encode(format!("UserConfig:{}", id))),
        }
    }

    pub fn from_id(id: &ID) -> Option<NodeID> {
        let id = id.to_string();
        let buf = base64::decode(id).ok()?;
        let s = String::from_utf8(buf).ok()?;
        s.split_once(':').and_then(|(kind, id)| match kind {
            "FigureRecord" => Ulid::from_str(id).ok().map(NodeID::FigureRecord),
            "CharacterConfig" => Ulid::from_str(id).ok().map(NodeID::CharacterConfig),
            "Character" => entities::Character::try_from(id)
                .ok()
                .map(NodeID::Character),
            "UserConfig" => Some(NodeID::UserConfig(id.to_string())),
            _ => None,
        })
    }
}

#[derive(GraphQLObject, Clone, Debug)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(name = "Error")]
pub struct GraphqlErrorType {
    pub message: String,
}
