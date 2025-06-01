use crate::errors::InternalError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginManagerError {
    #[error("Plugin registry error: {0}")]
    PluginRegistryError(#[from] PluginRegistryError),
    #[error("Host IPC error: {0}")]
    IpcError(#[from] InternalError),
    #[error("Plugin instance error: {0}")]
    PluginInstanceError(#[from] PluginInstanceError),
}

#[derive(Error, Debug)]
pub enum PluginRegistryError {
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Plugin discovery error: {0}")]
    DiscoveryError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[derive(Error, Debug)]
pub enum PluginInstanceError {
    #[error("Execution error: {0}")]
    ExecutionError(String),
}

pub type Result<T> = std::result::Result<T, PluginManagerError>;
