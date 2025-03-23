use crate::resource::{self, ResourceManager};

use super::executor::{self, TaskExecutor};
use super::storage::TaskStore;
use super::Result;
use super::{queue::TaskQueue, TaskCommand};
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

    // Configuration
    // config: Config,

    // Communication channels
    feedback_tx: mpsc::Sender<TaskCommand>,
    feedback_rx: Option<mpsc::Receiver<TaskCommand>>,
}

impl TaskCoordinator {
    /// Create a new TaskCoordinator.
    pub fn new(
        db: PgPool,
        // config: Config,
        resource_manager: Arc<ResourceManager>,
    ) -> Self {
        // Create communication channels.
        let (worker_tx, _worker_rx) = mpsc::channel(100);
        let (feedback_tx, feedback_rx) = mpsc::channel(100);

        // Create core components.
        let store = Arc::new(TaskStore::new(db));
        let queue = Arc::new(TaskQueue::new());
        let executor = Arc::new(TaskExecutor::new(
            resource_manager,
            worker_tx.clone(),
            feedback_tx.clone(),
        ));

        Self {
            store,
            queue,
            executor,
            feedback_tx,
            feedback_rx: Some(feedback_rx),
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

        Ok(())
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
                        // NOTE: we might consider including error message in task state.
                        if let Err(e) = store.update_task_state(task_id, TaskState::Failed).await {
                            error!("Failed to update task state: {}", e);
                        }
                    }
                    TaskCommand::TaskProgress {
                        task_id,
                        progress,
                        message,
                    } => {
                        debug!("Task {} progress: {}% - {}", task_id, progress, message);

                        // We could store progress updates if desired.
                        // For now, we just log them.
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
