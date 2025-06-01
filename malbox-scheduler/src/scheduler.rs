use super::error::Result;
use crate::resource::ResourceManager;
use crate::task::{queue::TaskQueue, store::TaskStore};
use crate::worker::event::WorkerEvent;
use crate::worker::pool::WorkerPool;
use malbox_database::repositories::tasks::{Task, TaskState};
use malbox_database::PgPool;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, warn};

/// The scheduler orchestrates the entire task-management system.
pub struct Scheduler {
    task_store: Arc<TaskStore>,
    task_queue: Arc<TaskQueue>,
    resource_manager: Arc<ResourceManager>,
    worker_pool: Arc<WorkerPool>,
    worker_events: mpsc::Receiver<WorkerEvent>,
    task_notifications: mpsc::Receiver<Task>,
    shutdown_notification: oneshot::Receiver<()>,
}

impl Scheduler {
    /// Create a new scheduler.
    pub fn new(
        db_pool: PgPool,
        resource_manager: Arc<ResourceManager>,
        task_notifications: mpsc::Receiver<Task>,
        worker_events: mpsc::Receiver<WorkerEvent>,
        shutdown_notification: oneshot::Receiver<()>,
    ) -> Self {
        let task_store = Arc::new(TaskStore::new(db_pool));
        let task_queue = Arc::new(TaskQueue::new());
        let worker_pool = Arc::new(WorkerPool::new(10, /* executor */ todo!()));

        Self {
            task_store,
            task_queue,
            worker_pool,
            resource_manager,
            task_notifications,
            worker_events,
            shutdown_notification,
        }
    }

    /// Run the scheduler.
    pub async fn run(mut self) -> Result<()> {
        // Load any pending tasks from database on startup
        self.task_store.load_pending_tasks().await?;

        let queue_notifier = self.task_queue.get_notifier();

        loop {
            tokio::select! {
                // Handle new task notifications
                Some(task) = self.task_notifications.recv() => {
                    self.handle_new_task(task).await?;
                }

                // Handle worker completion events
                Some(event) = self.worker_events.recv() => {
                    self.handle_worker_event(event).await?;
                }

                // Process queued tasks when queue has items
                _ = queue_notifier.notified() => {
                    todo!()
                }

                // Handle shutdown signal
                _ = &mut self.shutdown_notification => {
                    info!("Scheduler shutdown requested");
                    break;
                }
            }
        }

        self.shutdown().await?;
        Ok(())
    }

    /// Handle worker events (completion, errors, etc.).
    async fn handle_worker_event(&self, event: WorkerEvent) -> Result<()> {
        match event {
            WorkerEvent::JobCompleted {
                worker_id,
                job_result,
                duration,
            } => {
                info!(
                    "Worker {} completed job in {:?}",
                    worker_id.as_string(),
                    duration
                );

                match job_result {
                    Ok(task_result) => {
                        self.handle_task_completion(task_result).await?;
                    }
                    Err(e) => {
                        error!("Job failed: {}", e);
                        // TODO: Handle task failure
                    }
                }
            }

            WorkerEvent::BatchCompleted {
                worker_id,
                batch_results,
                duration,
            } => {
                info!(
                    "Worker {} completed batch of {} jobs in {:?}",
                    worker_id.as_string(),
                    batch_results.len(),
                    duration
                );

                for result in batch_results {
                    match result {
                        Ok(task_result) => {
                            self.handle_task_completion(task_result).await?;
                        }
                        Err(e) => {
                            error!("Batch job failed: {}", e);
                        }
                    }
                }
            }

            WorkerEvent::WorkerShutdown { worker_id, reason } => {
                info!("Worker {} shut down: {:?}", worker_id.as_string(), reason);
                // TODO: Handle worker replacement if needed
            }

            WorkerEvent::WorkerError { worker_id, error } => {
                error!(
                    "Worker {} reported error: {:?}",
                    worker_id.as_string(),
                    error
                );
                // TODO: Handle worker error recovery
            }
        }

        Ok(())
    }

    /// Handle successful task completion.
    async fn handle_task_completion(&self, task_result: TaskResult) -> Result<()> {
        let task_id = task_result.task_id.expect("Task result must have task ID");

        // Update task state to completed
        self.task_store
            .update_task_state(task_id, TaskState::Completed)
            .await?;

        // Release resources
        self.resource_manager.release_resources(task_id).await?;

        // Store results (implement this)
        // self.result_store.store_task_result(task_result).await?;

        info!("Task {} completed successfully", task_id);
        Ok(())
    }

    /// Handle a new task that has been sent to the scheduler.
    async fn handle_new_task(&self, task: Task) -> Result<()> {
        // TODO:
        // Load balancing!
        self.execute_task(task).await?;
        // In case resources are already exhausted:
        // self.task_queue.enqueue(task).await?;

        Ok(())
    }

    async fn execute_task(&self, task: Task) -> Result<()> {
        let worker = self.worker_pool.acquire_worker_for_task(&task).await?;

        // worker.send_job(job);

        Ok(())
    }

    /// Graceful shutdown.
    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down scheduler...");

        // Shutdown worker pool
        // self.worker_pool.shutdown().await?;

        // Clean up resources
        // self.resource_manager.cleanup().await?;

        info!("Scheduler shutdown complete");
        Ok(())
    }
}
