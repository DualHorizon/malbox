use super::error::Result;
use crate::task::{queue::TaskQueue, store::TaskStore};
use crate::worker::WorkerPool;
use malbox_database::repositories::tasks::Task;
use malbox_database::PgPool;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

/// The scheduler orchestrates the entire task-management system.
pub struct Scheduler {
    task_store: Arc<TaskStore>,
    task_queue: Arc<TaskQueue>,
    worker_pool: Arc<WorkerPool>,
    task_notifications: mpsc::Receiver<Task>,
    shutdown_notification: oneshot::Receiver<()>,
}

impl Scheduler {
    /// Create a new scheduler.
    // pub fn new(db_pool: PgPool) -> Self {
    //     let task_store = Arc::new(TaskStore::new(db_pool));
    //     let task_queue = Arc::new(TaskQueue::new());
    //     let worker_pool = Arc::new(WorkerPool::new());

    //     Self {
    //         task_store,
    //         task_queue,
    //         worker_pool,
    //     }
    // }

    /// Run the scheduler.
    pub async fn run(mut self) -> Result<()> {
        let queue_notifier = self.task_queue.get_notifier();

        loop {
            tokio::select! {
                // Wait for new task submission
                Some(task) = self.task_notifications.recv() => {
                    self.handle_new_task(task).await?;
                }

                _ = &mut self.shutdown_notification => {
                    break;
                }
            }
        }

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
        let worker = self.worker_pool.acquire_worker().await?;
        Ok(())
    }
}
