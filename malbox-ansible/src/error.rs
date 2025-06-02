use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnsibleError {
    #[error("Playbook execution failed: {playbook}")]
    PlaybookFailed {
        playbook: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("Playbook not found: {path}")]
    PlaybookNotFound { path: PathBuf },
    #[error("Invalid playbook configuration: {message}")]
    InvalidPlaybook { message: String },
    #[error("Inventory generation failed: {inventory}")]
    InventoryFailed { inventory: String },
    #[error("Invalid inventory configuration: {message}")]
    InvalidInventory { message: String },
    #[error("Ansible executable not found or not accessible")]
    AnsibleNotFound,
    #[error("Variable validation failed: {variable} = {value}")]
    InvalidVariable { variable: String, value: String },
    #[error("Host connection failed: {host}")]
    HostConnectionFailed { host: String },
    #[error("I/O error: {operation}")]
    Io {
        operation: String,
        #[source]
        source: std::io::Error,
    },
    #[error("YAML serialization error")]
    Serialization {
        #[source]
        source: serde_norway::Error,
    },
    #[error("Command execution error")]
    Execution {
        #[from]
        source: malbox_io_utils::error::IoError,
    },
}

pub type Result<T> = std::result::Result<T, AnsibleError>;
