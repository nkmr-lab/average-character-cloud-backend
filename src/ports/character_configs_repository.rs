use std::collections::HashMap;

use crate::entities;
use chrono::{DateTime, Utc};

pub trait CharacterConfigsRepository {
    type Error;

    async fn save(
        &mut self,
        now: DateTime<Utc>,
        character_config: entities::CharacterConfig,
    ) -> Result<entities::CharacterConfig, Self::Error>;

    // disabled=falseのみ
    async fn get_by_characters(
        &mut self,
        characters: &[entities::Character],
        user_id: entities::UserId,
    ) -> Result<Vec<entities::CharacterConfig>, Self::Error>;

    // disabled=falseのみ
    async fn query(
        &mut self,
        user_id: entities::UserId,
        after_id: Option<(entities::Character, entities::StrokeCount)>,
        before_id: Option<(entities::Character, entities::StrokeCount)>,
        limit: entities::Limit,
    ) -> Result<Vec<entities::CharacterConfig>, Self::Error>;

    // disabled=trueも含む全て
    async fn get_by_ids(
        &mut self,
        user_id: entities::UserId,
        keys: &[(entities::Character, entities::StrokeCount)],
    ) -> Result<
        HashMap<(entities::Character, entities::StrokeCount), entities::CharacterConfig>,
        Self::Error,
    >;
}

#[derive(Clone, thiserror::Error, Debug)]
pub enum CreateError {
    #[error("character already exists")]
    AlreadyExists,
}
