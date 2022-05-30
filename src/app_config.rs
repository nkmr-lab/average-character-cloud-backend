use std::env;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub redis_url: String,
    pub crypto_key: [u8; 64],
}

#[derive(Debug, Clone)]
pub enum AuthConfig {
    Dummy,
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
    pub fn from_env() -> Result<AppConfig, Box<dyn Error + Send + Sync>> {
        let origin = env::var("ORIGIN")?;
        let mount_base = env::var("MOUNT_BASE")
            .map(|s| s.split_terminator('/').map(|s| s.to_string()).collect())
            .unwrap_or_else(|_| Vec::new());
        let port = env::var("PORT")
            .unwrap_or("8080".to_owned())
            .parse::<u16>()?;
        let host = env::var("HOST").unwrap_or("localhost".to_owned());
        let database_url = env::var("DATABASE_URL")?;
        let auth = match env::var("AUTH_KIND")?.as_str() {
            "DUMMY" => AuthConfig::Dummy,
            "GOOGLE" => {
                let client_id = env::var("AUTH_GOOGLE_CLIENT_ID")?;
                AuthConfig::Google {
                    client_id,
                    enable_front: env::var("AUTH_GOOGLE_ENABLE_FRONT")
                        .map(|v| v == "TRUE")
                        .unwrap_or(false),
                }
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid auth kind",
            ))?,
        };
        let session = SessionConfig {
            redis_url: env::var("SESSION_REDIS_URL")?,
            crypto_key: base64::decode(env::var("SESSION_CRYPTO_KEY")?)?
                .as_slice()
                .try_into()?,
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
