use reqwest::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("HTTP status error: {0}")]
    HttpStatus(StatusCode),
    #[error("Download failed: Content length is zero")]
    EmptyContent,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("File type detection error: {0}")]
    Detection(String),
    #[error("Source not found: {0}")]
    SourceNotFound(String),
    #[error("Version not found: {0} for source {1}")]
    VersionNotFound(String, String),
    #[error("Invalid registry data: {0}")]
    InvalidData(String),
    #[error("Registry error: {0}")]
    Registry(String),
}

pub type Result<T> = std::result::Result<T, Error>;

// Convert from serde_json::Error
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::InvalidData(err.to_string())
    }
}
