use super::pool::WorkerPool;
use crate::error::Result;
use crate::task::executor::TaskExecutor;
use malbox_database::repositories::tasks::Task;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

struct Job {
    task: Task,
    // resources: ResourceAllocation,
    result_tx: oneshot::Sender<Result<TaskResult>>,
}
