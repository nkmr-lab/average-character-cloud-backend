use super::character;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct CharacterConfig {
    pub user_id: String,
    pub character: character::Character,
    pub stroke_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
}
