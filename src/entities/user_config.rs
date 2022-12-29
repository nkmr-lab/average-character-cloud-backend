use chrono::{DateTime, Utc};

use super::UserId;

#[derive(Clone, Debug)]
pub struct UserConfig {
    pub user_id: UserId,
    pub allow_sharing_character_configs: bool,
    pub allow_sharing_figure_records: bool,
    pub updated_at: Option<DateTime<Utc>>,
    pub version: u32,
}

impl UserConfig {
    pub fn default_config(user_id: UserId) -> UserConfig {
        UserConfig {
            user_id,
            allow_sharing_character_configs: false,
            allow_sharing_figure_records: false,
            updated_at: None,
            version: 0,
        }
    }
}
