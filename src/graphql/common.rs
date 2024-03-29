use std::str::FromStr;

use juniper::GraphQLObject;
use juniper::ID;

use thiserror::Error;
use ulid::Ulid;

use crate::entities;
use anyhow::anyhow;

#[derive(Debug, Error)]
#[error("{source}")]
pub struct GraphqlUserError {
    #[source]
    pub source: anyhow::Error,
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

pub fn encode_limit(
    first: Option<i32>,
    last: Option<i32>,
) -> Result<entities::Limit, GraphqlUserError> {
    match (first, last) {
        (Some(first), None) => {
            if first < 0 {
                Err(GraphqlUserError::from(
                    "first must be greater than or equal to 0",
                ))
            } else {
                entities::Limit::new(entities::LimitKind::First, first)
                    .map_err(anyhow::Error::from)
                    .map_err(GraphqlUserError::from)
            }
        }
        (None, Some(last)) => {
            if last < 0 {
                Err(GraphqlUserError::from(
                    "last must be greater than or equal to 0",
                ))
            } else {
                entities::Limit::new(entities::LimitKind::Last, last)
                    .map_err(anyhow::Error::from)
                    .map_err(GraphqlUserError::from)
            }
        }
        _ => Err(GraphqlUserError::from(
            "Must provide either first or last, not both",
        )),
    }
}

#[derive(Debug, Clone)]
pub enum NodeId {
    FigureRecord(entities::FigureRecordId),
    CharacterConfig(entities::UserId, entities::Character),
    Character(entities::Character),
    UserConfig(entities::UserId),
    CharacterConfigSeed(entities::Character),
}

impl NodeId {
    pub fn to_id(&self) -> ID {
        match self {
            NodeId::FigureRecord(id) => {
                ID::new(base64::encode(format!("FigureRecord:{}", Ulid::from(*id))))
            }
            NodeId::CharacterConfig(user_id, character) => ID::new(base64::encode(format!(
                "CharacterConfig:{}-{}",
                base64::encode(String::from(user_id.clone())),
                base64::encode(String::from(character.clone()))
            ))),
            NodeId::Character(character) => ID::new(base64::encode(format!(
                "Character:{}",
                String::from(character.clone())
            ))),
            NodeId::UserConfig(id) => ID::new(base64::encode(format!(
                "UserConfig:{}",
                String::from(id.clone())
            ))),
            NodeId::CharacterConfigSeed(character) => ID::new(base64::encode(format!(
                "CharacterConfigSeed:{}",
                String::from(character.clone())
            ))),
        }
    }

    pub fn from_id(id: &ID) -> Option<NodeId> {
        let id = id.to_string();
        let buf = base64::decode(id).ok()?;
        let s = String::from_utf8(buf).ok()?;
        s.split_once(':').and_then(|(kind, id)| match kind {
            "FigureRecord" => Ulid::from_str(id)
                .ok()
                .map(|id| NodeId::FigureRecord(entities::FigureRecordId::from(id))),
            "CharacterConfig" => {
                let (user_id, character) = id.split_once('-')?;
                let user_id = base64::decode(user_id).ok()?;
                let user_id = entities::UserId::from(String::from_utf8(user_id).ok()?);
                let character = base64::decode(character).ok()?;
                let character = String::from_utf8(character).ok()?;
                let character = entities::Character::try_from(character.as_str()).ok()?;
                Some(NodeId::CharacterConfig(user_id, character))
            }
            "Character" => entities::Character::try_from(id)
                .ok()
                .map(NodeId::Character),
            "UserConfig" => Some(NodeId::UserConfig(entities::UserId::from(id.to_string()))),
            "CharacterConfigSeed" => entities::Character::try_from(id)
                .ok()
                .map(NodeId::CharacterConfigSeed),
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
