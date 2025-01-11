use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found")]
    NotFound,
    #[error("Invalid machinery provider: {0}")]
    InvalidMachineryProvider(String),
    #[error("Invalid profile: {0}")]
    InvalidProfile(String),
    #[error("Invalid template: {0}")]
    InvalidTemplate(String),
}
