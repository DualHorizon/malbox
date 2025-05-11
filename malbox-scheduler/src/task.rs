use crate::resource::{ResourceError, ResourceManager};
use malbox_config::Config;
use malbox_database::{
    repositories::{
        machinery::MachinePlatform,
        samples::SampleEntity,
        tasks::{fetch_pending_tasks, fetch_task, update_task_status, Task, TaskState},
    },
    PgPool,
};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use thiserror::Error;
use time::{OffsetDateTime, PrimitiveDateTime};
use tokio::sync::{mpsc, oneshot, RwLock};
use tracing::{debug, error, info, warn};

pub mod coordinator;
mod executor;
pub mod notification;
mod queue;
mod storage;
mod worker;

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("Task not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] malbox_database::error::DatabaseError),
    #[error("Resource error: {0}")]
    Resource(#[from] ResourceError),
    #[error("Plugin error: {0}")]
    Plugin(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Task canceled")]
    Canceled,
}

pub type Result<T> = std::result::Result<T, TaskError>;
#[derive(Debug)]
pub enum TaskCommand {
    StartTask {
        task: Task,
        response: oneshot::Sender<Result<()>>,
    },
    CancelTask {
        task_id: i32,
        response: oneshot::Sender<Result<()>>,
    },
    TaskCompleted {
        task_id: i32,
        result: Result<String>,
    },
    TaskFailed {
        task_id: i32,
        // NOTE: we could use a oneshot channel here for error
        // but maybe it would be unnecessary complexity?
        error: Arc<TaskError>,
    },
    TaskProgress {
        task_id: i32,
        message: String,
    },
}
