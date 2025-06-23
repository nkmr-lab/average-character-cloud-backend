use r2d2::Pool;

use sqlx::PgPool;

use crate::faktory::FaktoryConnectionManager;
use crate::jobs;
use faktory::ConsumerBuilder;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
#[error(transparent)]
pub struct JobError(#[from] pub anyhow::Error);

pub trait Job<'de>: Deserialize<'de> + Serialize + 'static + Send {
    const JOB_TYPE: &'static str;

    async fn run(self, ctx: Ctx) -> Result<(), anyhow::Error>;
    fn register(c: &mut ConsumerBuilder<JobError>, ctx: &Ctx) {
        let rt = tokio::runtime::Handle::current();
        let ctx = ctx.clone();
        c.register(Self::JOB_TYPE, move |job| -> Result<(), JobError> {
            rt.block_on(async {
                let params = Self::deserialize(job.args()[0].clone())?;
                params.run(ctx.clone()).await
            })
            .map_err(JobError)
        });
    }

    async fn enqueue(
        self,
        faktory_pool: &Pool<FaktoryConnectionManager>,
    ) -> Result<(), anyhow::Error> {
        let faktory_pool = faktory_pool.clone();
        tokio::task::spawn_blocking(move || {
            faktory_pool.get()?.enqueue(faktory::Job::new(
                Self::JOB_TYPE,
                vec![serde_json::to_value(self).unwrap()],
            ))?;
            let result: Result<(), anyhow::Error> = Ok(());
            result
        })
        .await??;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Ctx {
    pub pool: PgPool,
}

pub fn run_worker(url: &str, ctx: Ctx) -> anyhow::Result<()> {
    let mut c = ConsumerBuilder::default();
    jobs::UpdateSeeds::register(&mut c, &ctx);

    let c = c.connect(Some(url)).unwrap();
    // 終了しないタスクはtokioのspawn_blockingを使ってはいけない
    std::thread::spawn(|| {
        c.run_to_completion(&["default"]);
    });

    Ok(())
}
