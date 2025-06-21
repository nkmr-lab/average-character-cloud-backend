use crate::entities;
use chrono::{DateTime, Utc};

pub trait Storage {
    type Error;

    async fn generate_upload_url(&mut self, file: entities::File) -> Result<String, Self::Error>;

    async fn generate_download_url(&mut self, file: entities::File) -> Result<String, Self::Error>;

    async fn verify(&mut self, file: entities::File) -> Result<(), Self::Error>;
}
