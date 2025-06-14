use crate::entities;
use chrono::{DateTime, Utc};

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

    async fn get_by_characters(
        &mut self,
        characters: &[entities::Character],
        user_id: entities::UserId,
    ) -> Result<Vec<entities::CharacterConfig>, Self::Error>;

    async fn get(
        &mut self,
        user_id: entities::UserId,
        after_character: Option<entities::Character>,
        before_character: Option<entities::Character>,
        limit: entities::Limit,
    ) -> Result<Vec<entities::CharacterConfig>, Self::Error>;
}

#[derive(Clone, thiserror::Error, Debug)]
pub enum CreateError {
    #[error("character already exists")]
    AlreadyExists,
}
