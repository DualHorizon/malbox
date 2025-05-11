use super::executor::TaskExecutor;
use super::storage::TaskStore;
use super::worker::{TaskWorker, WorkerPool};
use super::Result;
use super::{queue::TaskQueue, TaskCommand};
use crate::resource::ResourceManager;
use malbox_database::repositories::tasks::TaskState;
use malbox_database::PgPool;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// The TaskCoordinator orchestrates the entire task-management system.
pub struct TaskCoordinator {
    // Core components
    store: Arc<TaskStore>,
    queue: Arc<TaskQueue>,
    executor: Arc<TaskExecutor>,
    worker_pool: Arc<WorkerPool>,

    // Configuration
    // config: Config,

    // Communication channels
    feedback_tx: mpsc::Sender<TaskCommand>,
    feedback_rx: Option<mpsc::Receiver<TaskCommand>>,
    task_notifications: Option<mpsc::Receiver<i32>>,
}

impl TaskCoordinator {
    /// Create a new TaskCoordinator.
    pub fn new(
        db: PgPool,
        // config: Config,
        resource_manager: Arc<ResourceManager>,
        task_notifications: mpsc::Receiver<i32>,
        max_workers: usize,
    ) -> Self {
        // Create communication channels.
        let (feedback_tx, feedback_rx) = mpsc::channel(100);

        let worker_pool = Arc::new(WorkerPool::new(max_workers, feedback_tx.clone()));

        // Create core components.
        let store = Arc::new(TaskStore::new(db));
        let queue = Arc::new(TaskQueue::new());
        let executor = Arc::new(TaskExecutor::new(
            resource_manager,
            worker_pool.clone(),
            feedback_tx.clone(),
        ));

        Self {
            store,
            queue,
            executor,
            worker_pool,
            feedback_tx,
            feedback_rx: Some(feedback_rx),
            task_notifications: Some(task_notifications),
        }
    }

    /// Initialize the coordinator - load pending tasks and start processing loops.
    pub async fn init(&mut self) -> Result<()> {
        // Load pending tasks from database.
        info!("Loading pending tasks from database");
        let pending_tasks = self.store.load_pending_tasks().await?;

        // Add tasks IDs to queue with their priorities.
        if !pending_tasks.is_empty() {
            info!("Adding {} pending tasks to queue", pending_tasks.len());

            // Extract task IDs and priorities for the queue.
            let task_entries: Vec<(i32, i64)> = pending_tasks
                .iter()
                .filter_map(|task| task.id.map(|id| (id, task.priority)))
                .collect();

            // Bulk enqueue the pending tasks.
            self.queue.enqueue_batch(task_entries).await;
        }

        // Start the feedback processing loop.
        self.start_feedback_loop();
        // Start the scheduler loop.
        self.start_scheduler_loop();
        // Start the task notification listener.
        self.start_notification_listener();

        Ok(())
    }

    /// Start the notification listener.
    fn start_notification_listener(&mut self) {
        let store = self.store.clone();
        let queue = self.queue.clone();

        let mut task_notifications = self
            .task_notifications
            .take()
            .expect("Task notifications already taken");

        tokio::spawn(async move {
            info!("Starting task notification listener");

            while let Some(task_id) = task_notifications.recv().await {
                debug!("Received notification for new task: {}", task_id);

                match store.load_task(task_id).await {
                    Ok(task) => {
                        info!("Adding new task {} to queue", task_id);
                        queue.enqueue(task_id, task.priority).await;
                    }
                    Err(e) => {
                        error!("Failed to load notified task {}: {}", task_id, e);
                    }
                }
            }

            warn!("Task notification listener terminated - channel closed");
        });
    }

    /// Start the loop that processes feedback from task execution.
    fn start_feedback_loop(&mut self) {
        // Clone the necesasry components for the async task.
        let mut feedback_rx = self
            .feedback_rx
            .take()
            .expect("Feedback receiver already taken");

        let store = self.store.clone();

        // Spawn an async task to handle feedback.
        tokio::spawn(async move {
            info!("Starting task feedback loop");

            while let Some(command) = feedback_rx.recv().await {
                match command {
                    TaskCommand::TaskCompleted { task_id, result } => {
                        match result {
                            Ok(result_data) => {
                                info!("Task {} completed successfully", task_id);

                                // Update task state in the store.
                                if let Err(e) =
                                    store.update_task_state(task_id, TaskState::Completed).await
                                {
                                    error!("Failed to update task state: {}", e);
                                }

                                // Store the task result.
                                if let Err(e) = store.update_task_result(task_id, result_data).await
                                {
                                    error!("Failed to update task result: {}", e);
                                }
                            }
                            // NOTE: If we actually fallback to that branch - there's an issue with our system.
                            Err(e) => {
                                error!("Task {} Failed: {}", task_id, e);

                                // Update the task state to failed.
                                // NOTE: we might consider including error message in task state:
                                // let error_message = e.to_string();
                                if let Err(e) =
                                    store.update_task_state(task_id, TaskState::Failed).await
                                {
                                    error!("Failed to update task state: {}", e);
                                }
                            }
                        }
                    }
                    TaskCommand::TaskFailed { task_id, error } => {
                        error!("Task {} failed: {}", task_id, error);

                        // Update task state to failed.
                        // NOTE: we might consider including error message in task state through an additional field in DB.
                        if let Err(e) = store.update_task_state(task_id, TaskState::Failed).await {
                            error!("Failed to update task state: {}", e);
                        }
                        // TODO:
                        // Add setting to retry tasks.
                    }
                    TaskCommand::TaskProgress { task_id, message } => {
                        debug!("Task {} progress: {}", task_id, message);

                        // TODO:
                        // Progress should be propagated through UI.
                    }
                    _ => {
                        // For now, we don't need to handle other command types here.
                    }
                }
            }

            warn!("Task feedback loop terminated - channel closed");
        });
    }

    /// Start the loop that schedules tasks from the queue.
    fn start_scheduler_loop(&self) {
        let store = self.store.clone();
        let queue = self.queue.clone();
        let executor = self.executor.clone();

        // Spawn an async task to handle scheduling.
        tokio::spawn(async move {
            info!("Starting task scheduler loop");

            let mut consecutive_errors = 0;
            let max_consecutive_errors = 3;

            loop {
                // Check if there are tasks in the queue.
                match queue.dequeue().await {
                    Some(task_id) => {
                        // Reset error counter when we successfully
                        // dequeue a task.
                        consecutive_errors = 0;

                        // Try to load the task.
                        match store.load_task(task_id).await {
                            Ok(task) => {
                                info!("Starting execution of task: {}", task_id);

                                // Update task state to running.
                                if let Err(e) =
                                    store.update_task_state(task_id, TaskState::Running).await
                                {
                                    error!("Failed to update task state: {}", e);
                                }

                                // Try to execute the task.
                                if let Err(e) = executor.execute_task(task).await {
                                    error!("Failed to start task execution: {}", e);

                                    // Update task state to failed.
                                    if let Err(store_err) =
                                        store.update_task_state(task_id, TaskState::Failed).await
                                    {
                                        error!("Failed to update task state: {}", store_err);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to load task {}: {}", task_id, e);
                                // TODO: Consider what to do with invalid tasks.
                                // For now, we just log the error and continue.
                            }
                        }
                    }
                    None => {
                        // NOTE: We should have triggers for scheduling.
                        // For now:
                        // No tasks in queue, sleep before checking again.
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }
                }

                // NOTE: Possible CPU spinning?
                // For now, sleep a short time between task processing.
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });
    }
}
