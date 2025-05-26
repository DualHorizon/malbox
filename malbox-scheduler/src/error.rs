use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("Notification service error: {0}")]
    NotificationServiceError(String),
    #[error("Task error: {0}")]
    Task(#[from] TaskError),
    #[error("Worker error: {0}")]
    Worker(#[from] WorkerError),
    #[error("Resource error: {0}")]
    Resource(#[from] crate::resource::ResourceError),
    #[error("Database error: {0}")]
    Database(#[from] malbox_database::error::DatabaseError),
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("Worker unavailable")]
    WorkerUnavailable,
    #[error("Max workers reached")]
    MaxWorkersReached,
    #[error("Worker execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Worker timeout")]
    Timeout,
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("Task not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] malbox_database::error::DatabaseError),
    #[error("Resource error: {0}")]
    Resource(#[from] crate::resource::ResourceError),
    #[error("Plugin error: {0}")]
    Plugin(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Task canceled")]
    Canceled,
    #[error("Task timeout")]
    Timeout,
    #[error("Invalid task state transition")]
    InvalidStateTransition,
}

pub type Result<T> = std::result::Result<T, SchedulerError>;
