use malbox_io_utils::error::IoError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TerraformError {
    /// Terraform command execution failed.
    #[error("Terraform command failed: {command}")]
    CommandFailed {
        command: String,
        #[source]
        source: IoError,
    },
    /// Terraform command returned non-zero exit code.
    #[error("Terraform command '{command}' failed with exit code {exit_code}: {stderr}")]
    CommandExitCode {
        command: String,
        exit_code: i32,
        stderr: String,
    },
    /// Terraform binary not found or not executable.
    #[error("Terraform binary not found or not executable: {message}")]
    BinaryNotFound { message: String },
    /// Workspace operation failed.
    #[error("Workspace operation failed: {operation}")]
    WorkspaceError {
        operation: String,
        #[source]
        source: Box<TerraformError>,
    },
    /// State operation failed.
    #[error("State operation failed: {operation}")]
    StateError {
        operation: String,
        #[source]
        source: Box<TerraformError>,
    },
    /// Configuration validation failed.
    #[error("Configuration validation failed: {message}")]
    ConfigValidation { message: String },
    /// Environment directory not found.
    #[error("Environment directory not found: {path}")]
    EnvironmentNotFound { path: PathBuf },
    /// Variable parsing failed.
    #[error("Failed to parse variables from {file}: {message}")]
    VariableParsing { file: String, message: String },
    /// HCL parsing error.
    #[error("HCL parsing failed: {message}")]
    HclParsing {
        message: String,
        #[source]
        source: malbox_hcl_utils::error::HclError,
    },
    /// File I/O error.
    #[error("File I/O error: {operation}")]
    Io {
        operation: String,
        #[source]
        source: std::io::Error,
    },
    /// Missing required variable.
    #[error("Missing required variable: {variable}")]
    MissingVariable { variable: String },
    /// Invalid variable value.
    #[error("Invalid value for variable '{variable}': {message}")]
    InvalidVariable { variable: String, message: String },
    /// Backend configuration error.
    #[error("Backend configuration error: {message}")]
    BackendConfig { message: String },
    /// Resource operation failed.
    #[error("Resource operation failed for '{resource}': {message}")]
    ResourceOperation { resource: String, message: String },
}

pub type Result<T> = std::result::Result<T, TerraformError>;
