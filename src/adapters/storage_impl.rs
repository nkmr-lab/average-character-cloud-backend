use std::time::Duration;

use aws_sdk_s3::presigning::PresigningConfig;

use crate::{app_config::AppConfig, entities, ports::Storage};

#[derive(Debug, Clone)]
pub struct StorageImpl {
    pub config: AppConfig,
    pub client: aws_sdk_s3::Client,
}

impl Storage for StorageImpl {
    type Error = anyhow::Error;

    async fn generate_upload_url(&mut self, file: entities::File) -> Result<String, Self::Error> {
        let expires_in = Duration::from_secs(self.config.storage.presigned_upload_expires_in_secs);

        let req = self
            .client
            .put_object()
            .bucket(&self.config.storage.bucket)
            .key(String::from(file.key).as_str())
            .content_type(file.mime_type.value())
            .content_length(i32::from(file.size) as i64);

        let presigned_req = req
            .presigned(PresigningConfig::expires_in(expires_in)?)
            .await?;

        let url = presigned_req.uri().to_string();
        Ok(url)
    }

    async fn generate_download_url(&mut self, file: entities::File) -> Result<String, Self::Error> {
        let expires_in =
            Duration::from_secs(self.config.storage.presigned_download_expires_in_secs);

        let req = self
            .client
            .get_object()
            .bucket(&self.config.storage.bucket)
            .key(String::from(file.key).as_str());

        let presigned_req = req
            .presigned(PresigningConfig::expires_in(expires_in)?)
            .await?;

        let url = presigned_req.uri().to_string();
        Ok(url)
    }

    async fn verify(&mut self, file: entities::File) -> Result<(), Self::Error> {
        let req = self
            .client
            .head_object()
            .bucket(&self.config.storage.bucket)
            .key(String::from(file.key).as_str());
        let _ = req.send().await?;

        Ok(())
    }
}
