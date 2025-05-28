//! Error types for Plugin API v1.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Error during plugin initialization: {0}")]
    InitError(String),
    #[error("Error during plugin execution: {0}")]
    ExecutionError(String),
    #[error("Plugin configuration error: {0}")]
    ConfigError(String),
    #[error("Resource not available: {0}")]
    ResourceError(String),
    #[error("Communication error: {0}")]
    CommunicationError(String),
    #[error("Plugin timeout: {0}")]
    TimeoutError(String),
    #[error("API version mismatch: plugin requires {required}, core supports {supported}")]
    ApiVersionMismatch { required: String, supported: String },
}

pub type Result<T> = std::result::Result<T, PluginError>;
