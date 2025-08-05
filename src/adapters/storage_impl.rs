use std::time::Duration;

use aws_sdk_s3::presigning::PresigningConfig;

use crate::{app_config::AppConfig, entities, ports::Storage};

#[derive(Debug, Clone)]
pub struct StorageImpl {
    pub config: AppConfig,
    pub s3_client: aws_sdk_s3::Client,
    pub s3_presign_client: aws_sdk_s3::Client,
}

impl StorageImpl {
    pub fn new(
        config: AppConfig,
        s3_client: aws_sdk_s3::Client,
        s3_presign_client: aws_sdk_s3::Client,
    ) -> Self {
        Self {
            config,
            s3_client,
            s3_presign_client,
        }
    }
}

impl Storage for StorageImpl {
    type Error = anyhow::Error;

    async fn generate_upload_url(&mut self, file: &entities::File) -> Result<String, Self::Error> {
        let expires_in: Duration =
            Duration::from_secs(self.config.storage.presigned_upload_expires_in_secs);

        let req = self
            .s3_presign_client
            .put_object()
            .bucket(&self.config.storage.bucket)
            .key(String::from(file.key.clone()).as_str())
            .content_type(file.mime_type.value())
            .content_length(i32::from(file.size) as i64);

        let presigned_req = req
            .presigned(PresigningConfig::expires_in(expires_in)?)
            .await?;

        let url = presigned_req.uri().to_string();
        Ok(url)
    }

    async fn generate_download_url(
        &mut self,
        file: &entities::File,
    ) -> Result<String, Self::Error> {
        let expires_in =
            Duration::from_secs(self.config.storage.presigned_download_expires_in_secs);

        let req = self
            .s3_presign_client
            .get_object()
            .bucket(&self.config.storage.bucket)
            .key(String::from(file.key.clone()).as_str());

        let presigned_req = req
            .presigned(PresigningConfig::expires_in(expires_in)?)
            .await?;

        let url = presigned_req.uri().to_string();
        Ok(url)
    }

    async fn verify(&mut self, file: &entities::File) -> Result<(), Self::Error> {
        let req = self
            .s3_client
            .head_object()
            .bucket(&self.config.storage.bucket)
            .key(String::from(file.key.clone()).as_str());
        let _ = req.send().await?;

        Ok(())
    }
}
