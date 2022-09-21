use std::sync::Arc;

use celery::{prelude::*, Celery};
use chrono::Utc;
use sqlx::PgPool;

use crate::commands::character_config_seed_command;
use once_cell::sync::OnceCell;

static CTX: OnceCell<Ctx> = OnceCell::new();

pub type App = Arc<Celery<celery::broker::AMQPBroker>>;

#[derive(Debug)]
pub struct Ctx {
    pub pool: PgPool,
}

pub async fn init_app(addr: String, ctx: Ctx) -> anyhow::Result<App> {
    CTX.set(ctx).unwrap();

    let app = celery::app!(
        broker = AMQPBroker { addr },
        tasks = [update_seeds],
        task_routes = [
            "*" => "celery",
        ],
    )
    .await?;

    {
        let app = app.clone();
        tokio::spawn(async move {
            app.consume_from(&["celery"]).await.unwrap();
        });
    }

    Ok(app)
}

#[celery::task]
pub async fn update_seeds() -> TaskResult<()> {
    let ctx = CTX.get().unwrap();
    let now = Utc::now();
    character_config_seed_command::update_seeds(&ctx.pool, now)
        .await
        .map_err(|e| TaskError::UnexpectedError(e.to_string()))?;
    Ok(())
}
