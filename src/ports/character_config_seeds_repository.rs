use chrono::{DateTime, Utc};

use crate::entities;

pub trait CharacterConfigSeedsRepository {
    type Error;

    async fn update_seeds(&mut self, now: DateTime<Utc>) -> Result<(), Self::Error>;

    async fn get_by_characters(
        &mut self,
        characters: &[entities::Character],
    ) -> Result<Vec<entities::CharacterConfigSeed>, Self::Error>;

    async fn query(
        &mut self,
        user_id: entities::UserId,
        after_id: Option<(entities::Character, entities::StrokeCount)>,
        before_id: Option<(entities::Character, entities::StrokeCount)>,
        limit: entities::Limit,
        include_exist_character_config: bool,
    ) -> Result<Vec<entities::CharacterConfigSeed>, Self::Error>;

    async fn get_by_ids(
        &mut self,
        ids: &[(entities::Character, entities::StrokeCount)],
    ) -> Result<Vec<entities::CharacterConfigSeed>, Self::Error>;
}
