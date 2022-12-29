use crate::{
    commands::character_config_seed_command,
    job::{Ctx, Job},
};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSeeds {}

#[async_trait]
impl<'de> Job<'de> for UpdateSeeds {
    const JOB_TYPE: &'static str = "UPDATE_SEEDS";

    async fn run(self, ctx: Ctx) -> Result<(), anyhow::Error> {
        let now = Utc::now();
        character_config_seed_command::update_seeds(&ctx.pool, now).await?;
        Ok(())
    }
}
