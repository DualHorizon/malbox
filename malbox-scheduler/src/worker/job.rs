use super::pool::WorkerPool;
use crate::error::Result;
use crate::task::executor::TaskExecutor;
use malbox_database::repositories::tasks::Task;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

pub struct Job {
    pub task: Task,
    pub resources: ResourceAllocation,
    pub result_tx: oneshot::Sender<Result<TaskResult>>,
}
