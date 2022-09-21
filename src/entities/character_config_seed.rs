use super::character;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct CharacterConfigSeed {
    pub character: character::Character,
    pub stroke_count: usize,
    pub updated_at: DateTime<Utc>,
}
