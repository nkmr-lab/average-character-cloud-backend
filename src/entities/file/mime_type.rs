use thiserror::Error;

#[derive(Clone, Debug)]
pub struct MimeType {
    value: String,
    extension: String,
}

impl MimeType {
    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn extension(&self) -> &str {
        &self.extension
    }
}

#[derive(Error, Debug, Clone)]
pub enum MimeTypeTryFromError {
    #[error("Unsupported MIME type: {0}")]
    Unsupported(String),
}

impl TryFrom<String> for MimeType {
    type Error = MimeTypeTryFromError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "image/png" => Ok(Self {
                value: value.clone(),
                extension: "png".to_string(),
            }),
            "image/jpeg" => Ok(Self {
                value: value.clone(),
                extension: "jpg".to_string(),
            }),
            "image/gif" => Ok(Self {
                value: value.clone(),
                extension: "gif".to_string(),
            }),
            "image/webp" => Ok(Self {
                value: value.clone(),
                extension: "webp".to_string(),
            }),
            _ => Err(MimeTypeTryFromError::Unsupported(value)),
        }
    }
}
