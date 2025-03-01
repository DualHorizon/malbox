use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP status error: {0}")]
    HttpStatus(reqwest::StatusCode),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("File detection error: {0}")]
    Detection(String),
    #[error("File exists at path: {0}")]
    FileExists(PathBuf),
    #[error("Empty content received")]
    EmptyContent,
    #[error("Source not found: {0}")]
    SourceNotFound(String),
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("Hash mismatch: {0}")]
    HashMismatch(String),
    #[error("Size mismatch: {0}")]
    SizeMismatch(String),
    #[error("Dialoguer error: {0}")]
    Dialoguer(#[from] dialoguer::Error),
    #[error("Invalid source path: {0}")]
    InvalidSourcePath(String),
    #[error("Source family not found: {0}")]
    SourceFamilyNotFound(String),
    #[error("Source edition not found: {0}")]
    SourceEditionNotFound(String),
    #[error("Source release not found: {0}")]
    SourceReleaseNotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;

// Convert from serde_json::Error
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::InvalidData(err.to_string())
    }
}
