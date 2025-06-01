use crate::{
    error::Result,
    task::{
        batch::{BatchCollector, TaskBatch},
        executor::TaskExecutor,
    },
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use config::WorkerConfig;
use event::{ShutdownReason, WorkerEvent};
use handle::WorkerHandle;
use job::Job;
use pool::WorkerPool;

pub mod config;
pub mod event;
pub mod handle;
pub mod job;
pub mod pool;

/// Unique identifier for a worker instance.
///
/// Used to track and reference individual workers throughout the system.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WorkerId(Uuid);

impl WorkerId {
    /// Create a new unique worker ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the string represntation of the worker ID.
    pub fn as_string(&self) -> String {
        self.0.to_string()
    }
}

/// A worker that executes tasks.
///
/// Workers are responsible for executing tasks with their associated plugins
/// in the appropriate environment. Each worker runs in its own tokio task and
/// can be manager through a WorkerHandle.
pub struct Worker {
    /// Unique identifier for this worker.
    id: WorkerId,
    /// Task executor used to run tasks.
    executor: Arc<TaskExecutor>,
    /// Channel for receiving jobs.
    job_rx: mpsc::Receiver<Job>,
    /// Channel for signaling worker shutdown.
    shutdown_rx: oneshot::Receiver<()>,
    /// Channel for sending completion notifications back to the pool.
    completion_tx: mpsc::Sender<WorkerEvent>,
    /// Batch collector for this worker (if batch processing enabled).
    batch_collector: Option<BatchCollector>,
    /// Configuration for this worker.
    config: WorkerConfig,
}

impl Worker {
    /// Create a new worker with the given configuration.
    pub fn new(
        config: WorkerConfig,
        executor: Arc<TaskExecutor>,
    ) -> (Self, WorkerHandle, mpsc::Receiver<WorkerEvent>) {
        let (job_tx, job_rx) = mpsc::channel(16);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (completion_tx, completion_rx) = mpsc::channel(32);

        let id = WorkerId::new();

        let handle = WorkerHandle::new(id.clone(), job_tx, shutdown_tx);

        let batch_collector = if config.batch_processing {
            Some(BatchCollector::new(config.clone()))
        } else {
            None
        };

        let worker = Self {
            id: id.clone(),
            executor,
            config,
            job_rx,
            shutdown_rx,
            completion_tx,
            batch_collector,
        };

        (worker, handle, completion_rx)
    }

    /// Run the worker's main execution loop.
    ///
    /// This method contains the worker's entire lifecycle and should be
    /// spawned in a tokio task.
    pub async fn run(mut self) -> Result<()> {
        let idle_timeout = Duration::from_millis(self.config.batch_timeout_ms);
        let mut batch_timer = if self.config.batch_processing {
            Some(tokio::time::interval(idle_timeout / 2))
        } else {
            None
        };

        let mut last_activity = Instant::now();

        loop {
            tokio::select! {
                // Handle incoming jobs
                Some(job) = self.job_rx.recv() => {
                    last_activity = Instant::now();
                    self.handle_job(job).await?;
                }

                // Handle batch timeout (if batch processing enabled)
                _ = async {
                    if let Some(ref mut timer) = batch_timer {
                        timer.tick().await;
                    } else {
                        // If no batch processing, sleep forever
                        std::future::pending::<()>().await;
                    }
                }, if batch_timer.is_some() => {
                    self.check_batch_timeout().await?;
                }

                // Handle shutdown signal
                _ = &mut self.shutdown_rx => {
                    self.notify_shutdown(ShutdownReason::Requested).await;
                    break;
                }

                // Handle idle timeout
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    if self.config.idle_timeout_ms > 0 {
                        let idle_duration = Duration::from_millis(self.config.idle_timeout_ms);
                        if last_activity.elapsed() >= idle_duration {
                            self.notify_shutdown(ShutdownReason::IdleTimeout).await;
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle a job.
    async fn handle_job(&mut self, job: Job) -> Result<()> {
        let start_time = Instant::now();

        if self.config.batch_processing {
            self.handle_batch_job(job).await
        } else {
            self.handle_single_job(job, start_time).await
        }
    }

    /// Handle a job that might be batched.
    async fn handle_batch_job(&mut self, job: Job) -> Result<()> {
        if let Some(ref mut collector) = self.batch_collector {
            if let Some(batch) = collector
                .add_task(job.task, job.resources, job.result_tx)
                .await
            {
                self.execute_batch(batch).await?;
            }
        } else {
            // Fallback to single job if no batch collector
            let start_time = Instant::now();
            self.handle_single_job(job, start_time).await?;
        }

        Ok(())
    }

    /// Handle a single job execution.
    async fn handle_single_job(&self, job: Job, start_time: Instant) -> Result<()> {
        let result = self.executor.execute(job.task, job.resources).await;
        let duration = start_time.elapsed();

        // Send result back to caller
        let _ = job.result_tx.send(result.clone());

        // Notify pool of completion
        let event = WorkerEvent::JobCompleted {
            worker_id: self.id.clone(),
            job_result: result,
            duration,
        };

        let _ = self.completion_tx.send(event).await;

        Ok(())
    }

    /// Execute a bathc of tasks.
    async fn execute_batch(&self, batch: TaskBatch) -> Result<()> {
        let start_time = Instant::now();

        // Execute all tasks in the batch
        let results = self
            .executor
            .execute_batch(batch.tasks, batch.resources)
            .await?;
        let duration = start_time.elapsed();

        // Send individual results back
        for (result, result_tx) in results.iter().zip(batch.result_channels.into_iter()) {
            let _ = result_tx.send(result.clone());
        }

        // Notify pool of batch completion
        let event = WorkerEvent::BatchCompleted {
            worker_id: self.id.clone(),
            batch_results: results,
            duration,
        };

        let _ = self.completion_tx.send(event).await;

        Ok(())
    }

    /// Check if current batch should be finalized due to timeout.
    async fn check_batch_timeout(&mut self) -> Result<()> {
        if let Some(ref mut collector) = self.batch_collector {
            if collector.should_finalize_batch() {
                if let Some(batch) = collector.finalize_current_batch() {
                    self.execute_batch(batch).await?;
                }
            }
        }

        Ok(())
    }

    /// Notify the pool that this worker is shutting down.
    async fn notify_shutdown(&self, reason: ShutdownReason) {
        let event = WorkerEvent::WorkerShutdown {
            worker_id: self.id.clone(),
            reason,
        };

        let _ = self.completion_tx.send(event).await;
    }

    /// Create a worker from an existing handle.
    ///
    /// This method creates a Worker instance that can use an existing worker
    /// process through its handle. This is typically used when retrieving a
    /// worker from the pool.
    pub fn from_handle(
        handle: WorkerHandle,
        executor: Arc<TaskExecutor>,
        pool: Arc<WorkerPool>,
    ) -> Self {
        todo!()
    }

    /// Get a handle to this worker.
    ///
    /// Creates a new handle that can be used to control this worker.
    pub fn handle(&self) -> WorkerHandle {
        todo!()
    }
}
