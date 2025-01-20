use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found")]
    NotFound,
    #[error("Failed to parse {file}: {error}")]
    Parse { file: String, error: String },
    #[error("Failed to validate {field}: {message}")]
    ValidationError { field: String, message: String },
    #[error("Invalid value for {field}: {message}")]
    InvalidValue { field: String, message: String },
    #[error("Required environment variable {0} not set")]
    EnvVarNotSet(String),
    #[error("Provider {0} not configured")]
    ProviderNotConfigured(String),
    #[error("Profile {0} not found")]
    ProfileNotFound(String),
    #[error("Template {0} not found")]
    TemplateNotFound(String),
    #[error("Path error: {message} for {path}")]
    PathError { message: String, path: PathBuf },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),
    #[error("Machinery error: {0}")]
    Machinery(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl ConfigError {
    pub fn is_not_found(&self) -> bool {
        matches!(self, ConfigError::NotFound)
    }

    pub fn is_validation_error(&self) -> bool {
        matches!(self, ConfigError::ValidationError { .. })
    }

    pub fn is_io_error(&self) -> bool {
        matches!(self, ConfigError::Io(_))
    }
}

pub type Result<T> = std::result::Result<T, ConfigError>;
