use chrono::{DateTime, Utc};

use sqlx::PgPool;

use crate::entities;

pub use super::loaders::Loaders;

pub struct AppCtx {
    pub pool: PgPool,
    pub user_id: Option<entities::UserId>,
    pub now: DateTime<Utc>,
    pub loaders: Loaders,
}

impl juniper::Context for AppCtx {}
