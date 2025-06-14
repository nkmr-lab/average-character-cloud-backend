use crate::entities;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait CharacterConfigsRepository {
    type Error;

    async fn create(
        &mut self,
        user_id: entities::UserId,
        now: DateTime<Utc>,
        character: entities::Character,
        stroke_count: entities::StrokeCount,
    ) -> Result<Result<entities::CharacterConfig, CreateError>, Self::Error>;

    async fn update(
        &mut self,
        now: DateTime<Utc>,
        character_config: entities::CharacterConfig,
        stroke_count: Option<entities::StrokeCount>,
    ) -> Result<entities::CharacterConfig, Self::Error>;
}

#[derive(Clone, thiserror::Error, Debug)]
pub enum CreateError {
    #[error("character already exists")]
    AlreadyExists,
}
