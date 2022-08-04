use serde::{Deserialize, Deserializer};
use serde_with::with_prefix;
use std::error::Error;

// 以下の問題が解決されるまでは全てflattenする
// https://github.com/mehcode/config-rs/issues/312

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "UPPERCASE")]
pub enum SessionConfig {
    Redis {
        // enumのバリアントにはwith_prefixは使えないのでとりあえず
        // https://github.com/jonasbb/serde_with/issues/483
        #[serde(rename = "redis_url")]
        url: String,
        #[serde(rename = "redis_crypto_key")]
        crypto_key: CryptoKeyConfig,
    },
    Dummy {
        #[serde(rename = "dummy_user_id")]
        user_id: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "UPPERCASE")]
pub enum AuthConfig {
    Disable,
    Google {
        #[serde(rename = "google_client_id")]
        client_id: String,
        #[serde(rename = "google_enable_front")]
        enable_front: bool,
        #[serde(rename = "google_redirect_url")]
        redirect_url: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub mount_base: PathConfig,
    #[serde(default = "port_default")]
    pub port: u16,
    #[serde(default = "host_default")]
    pub host: String,
    pub database_url: String,
    #[serde(flatten, with = "prefix_auth")]
    pub auth: AuthConfig,
    #[serde(flatten, with = "prefix_session")]
    pub session: SessionConfig,
    pub origin: String,
    pub logout_redirect_url: String,
}

with_prefix!(prefix_auth "auth_");
with_prefix!(prefix_session "session_");

// with_prefix と deserialize_withを一緒に使うことができないのでnewtypeを定義
#[derive(Debug, Clone, Default)]
pub struct PathConfig(pub Vec<String>);
impl<'de> Deserialize<'de> for PathConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)
            .map(|s| PathConfig(s.split_terminator('/').map(|s| s.to_string()).collect()))
    }
}

#[derive(Debug, Clone)]
pub struct CryptoKeyConfig(pub [u8; 64]);
impl<'de> Deserialize<'de> for CryptoKeyConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        String::deserialize(deserializer)
            .and_then(|string| {
                base64::decode(&string).map_err(|err| Error::custom(err.to_string()))
            })
            .and_then(|bytes| {
                bytes
                    .as_slice()
                    .try_into()
                    .map(CryptoKeyConfig)
                    .map_err(|_| Error::custom("not 64 bytes"))
            })
    }
}

fn port_default() -> u16 {
    8080
}

fn host_default() -> String {
    "localhost".to_string()
}

impl AppConfig {
    pub fn from_env() -> Result<AppConfig, Box<dyn Error + Send + Sync>> {
        let config = config::Config::builder()
            // try_parsing, 1という文字列を指定できなくなるのであまり使いたくないが他にいい方法がなさそう
            .add_source(config::Environment::default().try_parsing(true))
            .build()?;

        Ok(config.try_deserialize()?)
    }
}
