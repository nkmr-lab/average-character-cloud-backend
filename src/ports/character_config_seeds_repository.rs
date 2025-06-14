use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait CharacterConfigSeedsRepository {
    type Error;

    async fn update_seeds(&mut self, now: DateTime<Utc>) -> Result<(), Self::Error>;
}
