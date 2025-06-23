use super::{character, Ratio, StrokeCount, UserId, Version};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct CharacterConfig {
    pub user_id: UserId,
    pub character: character::Character,
    pub stroke_count: StrokeCount,
    pub updated_at: Option<DateTime<Utc>>,
    pub version: Version,
    pub ratio: Ratio,
    pub disabled: bool,
}

impl CharacterConfig {
    pub fn default_config(
        user_id: UserId,
        character: character::Character,
        stroke_count: StrokeCount,
    ) -> CharacterConfig {
        CharacterConfig {
            user_id,
            character,
            stroke_count,
            updated_at: None,
            version: Version::none(),
            ratio: Ratio::default(),
            disabled: true,
        }
    }

    pub fn with_ratio(mut self, ratio: Ratio) -> Self {
        self.ratio = ratio;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
