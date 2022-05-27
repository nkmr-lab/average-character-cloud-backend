use std::env;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub mount_base: String,
    pub port: u16,
    pub host: String,
    pub database_url: String,
    pub dummy_user_id: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<AppConfig, Box<dyn Error + Send + Sync>> {
        let mount_base = env::var("MOUNT_BASE").unwrap_or_else(|_| "/".to_owned());
        let port = env::var("PORT")
            .unwrap_or("8080".to_owned())
            .parse::<u16>()?;
        let host = env::var("HOST").unwrap_or("localhost".to_owned());
        let database_url = env::var("DATABASE_URL")?;
        let dummy_user_id = env::var("DUMMY_USER_ID").ok();

        Ok(AppConfig {
            mount_base,
            port,
            host,
            database_url,
            dummy_user_id,
        })
    }
}
