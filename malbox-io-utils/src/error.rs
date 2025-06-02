use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during I/O operations.
#[derive(Error, Debug)]
pub enum IoError {
    /// Standard I/O error (file not found, permission denied, etc.).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Command execution failed.
    #[error("Command '{command}' failed with exit code {exit_code}")]
    CommandFailed { command: String, exit_code: i32 },
    /// Command not found or not executable.
    #[error("Command not found: {command}")]
    CommandNotFound { command: String },
    /// Process spawn failed.
    #[error("Failed to spawn process '{command}': {message}")]
    SpawnFailed { command: String, message: String },
    /// Operation timed out.
    #[error("Operation timed out after {timeout_secs} seconds")]
    Timeout { timeout_secs: u64 },
    /// Invalid path provided.
    #[error("Invalid path: {path}")]
    InvalidPath { path: PathBuf },
    /// Output processing error.
    #[error("Output processing error: {message}")]
    OutputProcessing { message: String },
    /// Process was terminated unexpectedly.
    #[error("Process terminated unexpectedly: {message}")]
    ProcessTerminated { message: String },
}

/// Result type alias for I/O operations.
pub type Result<T> = std::result::Result<T, IoError>;
