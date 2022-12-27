use std::sync::Arc;

use chrono::Utc;
use sqlx::PgPool;

use crate::commands::character_config_seed_command;
use crate::jobs;
use faktory::ConsumerBuilder;
use serde::Deserialize;
use thiserror::Error;
use tokio::runtime::Handle;

#[derive(Error, Debug)]
#[error(transparent)]
pub struct JobError(#[from] pub anyhow::Error);

#[derive(Debug)]
pub struct Ctx {
    pub pool: PgPool,
    pub rt: Handle,
}

pub fn run_worker(url: &str, ctx: Ctx) -> anyhow::Result<()> {
    let ctx = Arc::new(ctx);

    let mut c = ConsumerBuilder::default();
    {
        let ctx = ctx.clone();
        c.register(
            jobs::update_seeds::JOB_TYPE,
            move |job| -> Result<(), JobError> {
                ctx.rt
                    .block_on(async {
                        let _arg = jobs::update_seeds::Arg::deserialize(&job.args()[0])?;
                        let now = Utc::now();
                        character_config_seed_command::update_seeds(&ctx.pool, now).await?;

                        Ok(())
                    })
                    .map_err(JobError)
            },
        );
    }

    let c = c.connect(Some(url)).unwrap();
    // 終了しないタスクはtokioのspawn_blockingを使ってはいけない
    std::thread::spawn(|| {
        c.run_to_completion(&["default"]);
    });

    Ok(())
}
