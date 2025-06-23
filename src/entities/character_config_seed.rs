use super::{character, Ratio, StrokeCount};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct CharacterConfigSeed {
    pub character: character::Character,
    pub stroke_count: StrokeCount,
    pub updated_at: DateTime<Utc>,
    pub ratio: Ratio,
}
