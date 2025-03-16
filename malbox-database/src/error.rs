use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("{0}")]
    Machine(#[from] MachineError),
    #[error("{0}")]
    Task(#[from] TaskError),
    #[error("{0}")]
    Sample(#[from] SampleError),
}

#[derive(Error, Debug)]
pub enum MachineError {
    #[error("Failed to insert machine '{name}': {message}")]
    InsertFailed {
        name: String,
        message: String,
        #[source]
        source: sqlx::Error,
    },
    #[error("Failed to truncate `machines` table")]
    TruncateFailed {
        #[source]
        source: sqlx::Error,
    },
    #[error("Failed to delete from `machines` table")]
    DeleteFailed {
        #[source]
        source: sqlx::Error,
    },
    #[error("Failed to fetch machines")]
    FetchFailed {
        #[source]
        source: sqlx::Error,
    },
    #[error("Failed to update machine: {message}")]
    UpdateFailed {
        message: String,
        #[source]
        source: sqlx::Error,
    },
}

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("Failed to insert task '{name}': {message}")]
    InsertFailed {
        name: String,
        message: String,
        #[source]
        source: sqlx::Error,
    },
    #[error("Failed to fetch tasks")]
    FetchFailed {
        message: String,
        #[source]
        source: sqlx::Error,
    },
    #[error("Failed to update task")]
    UpdateFailed {
        task_id: i32,
        message: String,
        #[source]
        source: sqlx::Error,
    },
}

#[derive(Error, Debug)]
pub enum SampleError {
    #[error("Failed to insert sample '{hash}': {message}")]
    InsertFailed {
        hash: String,
        message: String,
        #[source]
        source: sqlx::Error,
    },
    #[error("Failed to fetch sample: '{hash}': {message}")]
    FetchFailed {
        hash: String,
        message: String,
        #[source]
        source: sqlx::Error,
    },
}

pub type Result<T> = std::result::Result<T, DatabaseError>;
