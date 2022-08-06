use anyhow::{anyhow, Context};
use chrono::{DateTime, FixedOffset, Utc};
use jsonwebtoken::jwk::JwkSet;
use reqwest::header;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Clone)]
struct GooglePublicKey {
    jwks: JwkSet,
    expires: DateTime<FixedOffset>,
}

async fn fetch_google_public_key() -> anyhow::Result<GooglePublicKey> {
    let res = reqwest::get("https://www.googleapis.com/oauth2/v3/certs").await?;
    let expires = DateTime::parse_from_rfc2822(
        res.headers()
            .get(header::EXPIRES)
            .ok_or_else(|| anyhow!("no expires header"))?
            .to_str()?,
    )?;
    let jwks = serde_json::from_str::<JwkSet>(res.text().await?.as_str())?;

    Ok(GooglePublicKey { jwks, expires })
}

#[derive(Debug)]
pub struct GooglePublicKeyProvider {
    data: Option<GooglePublicKey>,
}

impl GooglePublicKeyProvider {
    pub async fn run(mut command_receiver: mpsc::Receiver<GooglePublicKeyProviderCommand>) {
        let mut provider = GooglePublicKeyProvider { data: None };
        while let Some(command) = command_receiver.recv().await {
            match command {
                GooglePublicKeyProviderCommand::Get { resp } => {
                    let _ = resp.send(provider.get().await);
                }
            }
        }
    }

    async fn get(&mut self) -> anyhow::Result<JwkSet> {
        if let Some(data) = self.data.as_ref().filter(|key| key.expires > Utc::now()) {
            Ok(data.jwks.clone())
        } else {
            tracing::info!("fetch google public key");
            let pubkey = fetch_google_public_key()
                .await
                .context("fetch google public key")?;
            self.data = Some(pubkey.clone());
            Ok(pubkey.jwks)
        }
    }
}

#[derive(Debug)]
pub enum GooglePublicKeyProviderCommand {
    Get {
        resp: oneshot::Sender<anyhow::Result<JwkSet>>,
    },
}
