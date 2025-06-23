use chrono::{DateTime, Utc};

use sqlx::PgPool;

use crate::{app_config::AppConfig, entities};

pub use super::loaders::Loaders;

pub struct AppCtx {
    pub pool: PgPool,
    pub user_id: Option<entities::UserId>,
    pub now: DateTime<Utc>,
    pub loaders: Loaders,
    pub config: AppConfig,
    pub s3_client: aws_sdk_s3::Client,
}

impl juniper::Context for AppCtx {}
