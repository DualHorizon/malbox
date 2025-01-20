use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Configuration error: {0}")]
    Config(#[from] malbox_config::ConfigError),
    #[error("Builder error: {0}")]
    Builder(String),
    #[error("Infrastructure error: {0}")]
    Infrastructure(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Command failed: {0}")]
    CommandFailed(String),
}

pub type Result<T> = std::result::Result<T, CliError>;
