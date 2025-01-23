use malbox_storage::error::StorageError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found")]
    NotFound,

    #[error("Failed to parse {file}: {error}")]
    Parse { file: String, error: String },

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

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl ConfigError {
    pub fn is_not_found(&self) -> bool {
        matches!(self, ConfigError::NotFound)
    }

    pub fn is_invalid_value(&self) -> bool {
        matches!(self, ConfigError::InvalidValue { .. })
    }

    pub fn is_parse_error(&self) -> bool {
        matches!(self, ConfigError::Parse { .. })
    }

    pub fn is_storage_error(&self) -> bool {
        matches!(self, ConfigError::Storage(_))
    }

    pub fn is_provider_error(&self) -> bool {
        matches!(self, ConfigError::ProviderNotConfigured(_))
    }

    pub fn as_storage_error(&self) -> Option<&StorageError> {
        if let ConfigError::Storage(err) = self {
            Some(err)
        } else {
            None
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::Storage(StorageError::from(err))
    }
}

pub type Result<T> = std::result::Result<T, ConfigError>;
