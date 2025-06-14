use crate::{
    adapters::CharacterConfigSeedsRepositoryImpl,
    job::{Ctx, Job},
    ports::CharacterConfigSeedsRepository,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSeeds {}

impl<'de> Job<'de> for UpdateSeeds {
    const JOB_TYPE: &'static str = "UPDATE_SEEDS";

    async fn run(self, ctx: Ctx) -> Result<(), anyhow::Error> {
        let now = Utc::now();
        let mut character_config_seeds_repository =
            CharacterConfigSeedsRepositoryImpl::new(ctx.pool.clone());

        character_config_seeds_repository.update_seeds(now).await?;
        Ok(())
    }
}
