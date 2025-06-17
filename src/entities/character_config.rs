use super::{character, Ratio, StrokeCount, UserId, Version};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct CharacterConfig {
    pub user_id: UserId,
    pub character: character::Character,
    pub stroke_count: StrokeCount,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: Version,
    pub ratio: Ratio,
}
