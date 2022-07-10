use chrono::{DateTime, Utc};

use sqlx::PgPool;

pub use super::loaders::Loaders;

pub struct AppCtx {
    pub pool: PgPool,
    pub user_id: Option<String>,
    pub now: DateTime<Utc>,
    pub loaders: Loaders,
}

impl juniper::Context for AppCtx {}
