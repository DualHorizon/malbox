use thiserror::Error;

#[derive(Error, Debug)]
pub enum DaemonError {
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, DaemonError>;
