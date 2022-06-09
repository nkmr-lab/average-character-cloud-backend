use anyhow::{anyhow, Context};
use std::env;

#[derive(Debug, Clone)]
pub enum SessionConfig {
    Redis { url: String, crypto_key: [u8; 64] },
    Dummy { user_id: String },
}

#[derive(Debug, Clone)]
pub enum AuthConfig {
    Disable,
    Google {
        client_id: String,
        enable_front: bool,
    },
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub mount_base: Vec<String>,
    pub port: u16,
    pub host: String,
    pub database_url: String,
    pub auth: AuthConfig,
    pub session: SessionConfig,
    pub origin: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<AppConfig> {
        let origin = env::var("ORIGIN").context("ORIGIN")?;
        let mount_base = env::var("MOUNT_BASE")
            .map(|s| s.split_terminator('/').map(|s| s.to_string()).collect())
            .unwrap_or_else(|_| Vec::new());
        let port = env::var("PORT")
            .map(|x| x.parse::<u16>())
            .unwrap_or(Ok(8080))
            .context("PORT")?;
        let host = env::var("HOST").unwrap_or_else(|_| "localhost".to_owned());
        let database_url = env::var("DATABASE_URL").context("DATABASE_URL")?;
        let auth = match env::var("AUTH_KIND").context("AUTH_KIND")?.as_str() {
            "DISABLE" => AuthConfig::Disable,
            "GOOGLE" => {
                let client_id =
                    env::var("AUTH_GOOGLE_CLIENT_ID").context("AUTH_GOOGLE_CLIENT_ID")?;
                AuthConfig::Google {
                    client_id,
                    enable_front: env::var("AUTH_GOOGLE_ENABLE_FRONT")
                        .map(|v| v == "TRUE")
                        .unwrap_or(false),
                }
            }
            _ => Err(anyhow!("Invalid auth kind"))?,
        };
        let session = match env::var("SESSION_KIND").context("SESSION_KIND")?.as_str() {
            "REDIS" => {
                let url = env::var("SESSION_REDIS_URL").context("SESSION_REDIS_URL")?;
                let crypto_key = {
                    let res: anyhow::Result<_> = try {
                        base64::decode(env::var("SESSION_REDIS_CRYPTO_KEY")?)?
                            .as_slice()
                            .try_into()?
                    };
                    res
                }
                .context("SESSION_REDIS_CRYPTO_KEY")?;
                SessionConfig::Redis { url, crypto_key }
            }
            "DUMMY" => {
                let user_id = env::var("SESSION_DUMMY_USER_ID").context("SESSION_DUMMY_USER_ID")?;
                SessionConfig::Dummy { user_id }
            }
            _ => Err(anyhow!("Invalid session kind"))?,
        };

        Ok(AppConfig {
            mount_base,
            port,
            host,
            database_url,
            auth,
            session,
            origin,
        })
    }
}
