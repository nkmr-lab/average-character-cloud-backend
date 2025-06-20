use chrono::{DateTime, Utc};

use super::{RandomLevel, SharedProportion, UserId, Version};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserConfig {
    pub user_id: UserId,
    pub allow_sharing_character_configs: bool,
    pub allow_sharing_figure_records: bool,
    pub random_level: RandomLevel,
    pub shared_proportion: SharedProportion,
    pub updated_at: Option<DateTime<Utc>>,
    pub version: Version,
}

impl UserConfig {
    pub fn default_config(user_id: UserId) -> UserConfig {
        UserConfig {
            user_id,
            allow_sharing_character_configs: false,
            allow_sharing_figure_records: false,
            updated_at: None,
            version: Version::none(),
            random_level: RandomLevel::default(),
            shared_proportion: SharedProportion::default(),
        }
    }

    pub fn with_allow_sharing_character_configs(
        mut self,
        allow_sharing_character_configs: bool,
    ) -> UserConfig {
        self.allow_sharing_character_configs = allow_sharing_character_configs;
        self
    }

    pub fn with_allow_sharing_figure_records(
        mut self,
        allow_sharing_figure_records: bool,
    ) -> UserConfig {
        self.allow_sharing_figure_records = allow_sharing_figure_records;
        self
    }

    pub fn with_random_level(mut self, random_level: RandomLevel) -> UserConfig {
        self.random_level = random_level;
        self
    }

    pub fn with_shared_proportion(mut self, shared_proportion: SharedProportion) -> UserConfig {
        self.shared_proportion = shared_proportion;
        self
    }
}
