use thiserror::Error;

#[derive(Debug, Error)]
pub enum InternalError {
    #[error("Communication error: {0}")]
    CommunicationError(String),
}

pub type Result<T> = std::result::Result<T, InternalError>;
