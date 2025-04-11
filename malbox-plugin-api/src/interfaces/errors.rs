use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Error during plugin initialization: {0}")]
    InitError(String),
}

pub type Result<T> = std::result::Result<T, PluginError>;
