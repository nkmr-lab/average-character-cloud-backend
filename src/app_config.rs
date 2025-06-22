use serde::{Deserialize, Deserializer};
use std::error::Error;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionConfig {
    Redis {
        redis_url: String,
        #[serde(deserialize_with = "deserialize_crypto_key")]
        crypto_key: [u8; 64],
    },
    Dummy {
        user_id: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthConfig {
    Disable {},
    Google {
        client_id: String,
        #[serde(default)]
        enable_front: bool,
        redirect_url: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default, deserialize_with = "deserialize_path")]
    pub mount_base: Vec<String>,
    #[serde(default = "port_default")]
    pub port: u16,
    #[serde(default = "host_default")]
    pub host: String,
    pub database_url: String,
    pub auth: AuthConfig,
    pub session: SessionConfig,
    pub origin: String,
    pub logout_redirect_url: String,
    pub faktory_url: String,
    pub enqueue_cron_task: bool,
    #[serde(default)]
    pub enable_task_front: bool,
    #[serde(default = "workers_default")]
    pub workers: usize,
    pub storage: StorageConfig,
}

// serde_envがprefixに未対応なので
#[derive(Debug, Clone, Deserialize)]
struct PrefixedAppConfig {
    avcc: AppConfig,
}

fn deserialize_path<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
        .map(|s| s.split_terminator('/').map(|s| s.to_string()).collect())
}

fn deserialize_crypto_key<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|string| base64::decode(string).map_err(|err| Error::custom(err.to_string())))
        .and_then(|bytes| {
            bytes
                .as_slice()
                .try_into()
                .map_err(|_| Error::custom("not 64 bytes"))
        })
}

fn port_default() -> u16 {
    8080
}

fn host_default() -> String {
    "localhost".to_string()
}

fn workers_default() -> usize {
    num_cpus::get()
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub bucket: String,
    #[serde(default = "storage_presigned_upload_expires_in_secs_default")]
    pub presigned_upload_expires_in_secs: u64,
    #[serde(default = "storage_presigned_download_expires_in_secs_default")]
    pub presigned_download_expires_in_secs: u64,
    #[serde(default)]
    pub path_style: bool,
}

fn storage_presigned_upload_expires_in_secs_default() -> u64 {
    60 * 60 // 1 hour
}

fn storage_presigned_download_expires_in_secs_default() -> u64 {
    60 * 60 * 24 // 1 day
}

impl AppConfig {
    pub fn from_env() -> Result<AppConfig, Box<dyn Error + Send + Sync>> {
        Ok(serde_env::from_env::<PrefixedAppConfig>()?.avcc)
    }
}
