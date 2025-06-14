use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::entities;

#[async_trait]
pub trait CharacterConfigSeedsRepository {
    type Error;

    async fn update_seeds(&mut self, now: DateTime<Utc>) -> Result<(), Self::Error>;

    async fn get_by_characters(
        &mut self,
        characters: &[entities::Character],
    ) -> Result<Vec<entities::CharacterConfigSeed>, Self::Error>;

    async fn get(
        &mut self,
        user_id: entities::UserId,
        after_character: Option<entities::Character>,
        before_character: Option<entities::Character>,
        limit: entities::Limit,
        include_exist_character_config: bool,
    ) -> Result<Vec<entities::CharacterConfigSeed>, Self::Error>;
}
