use crate::entities;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait UserConfigsRepository {
    type Error;

    async fn get(&mut self, user_id: entities::UserId)
        -> Result<entities::UserConfig, Self::Error>;

    async fn save(
        &mut self,
        now: DateTime<Utc>,
        user_config: entities::UserConfig,
    ) -> Result<entities::UserConfig, Self::Error>;
}
