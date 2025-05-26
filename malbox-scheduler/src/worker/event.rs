use super::WorkerId;
use crate::error::Result;
use tokio::time::Duration;

/// Events that workers send back to the pool for coordination.
#[derive(Debug)]
pub enum WorkerEvent {
    /// Worker has completed a job and is now idle.
    JobCompleted {
        worker_id: WorkerId,
        job_result: Result<TaskResult>,
        duration: Duration,
    },
    /// Worker has processed a batch and is now idle.
    BatchCompleted {
        worker_id: WorkerId,
        batch_results: Vec<Result<TaskResult>>,
        duration: Duration,
    },
    /// Worker is shutting down.
    WorkerShutdown {
        worker_id: WorkerId,
        reason: ShutdownReason,
    },
    /// Worker encountered an error.
    WorkerError {
        worker_id: WorkerId,
        error: WorkerError,
    },
}

#[derive(Debug)]
pub enum ShutdownReason {
    IdleTimeout,
    Requested,
    Error(String),
}
