use crate::entities;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait UserConfigsRepository {
    type Conn;
    type Error;

    async fn get(
        &mut self,
        conn: &mut Self::Conn,
        user_id: entities::UserId,
    ) -> Result<entities::UserConfig, Self::Error>;

    async fn save(
        &mut self,
        conn: &mut Self::Conn,
        now: DateTime<Utc>,
        mut user_config: entities::UserConfig,
        allow_sharing_character_configs: Option<bool>,
        allow_sharing_figure_records: Option<bool>,
    ) -> Result<entities::UserConfig, Self::Error>;
}
