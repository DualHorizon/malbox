use super::Result;
use malbox_core::{ExecutionMode, PluginEvent, PluginManager, PluginRegistry, PluginType};
use malbox_database::repositories::tasks::Task;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, error, info, warn};

use super::TaskCommand;

/// Manages task execution.
pub struct TaskWorker {
    plugin_registry: Arc<PluginRegistry>,
    plugin_managers: Mutex<HashMap<i32, Arc<PluginManager>>>,

    command_rx: mpsc::Receiver<TaskCommand>,
    feedback_tx: mpsc::Sender<TaskCommand>,
    // config: Arc<Config>,
}

impl TaskWorker {
    /// Create a new TaskWorker.
    pub fn new(
        plugin_registry: Arc<PluginRegistry>,
        command_rx: mpsc::Receiver<TaskCommand>,
        feedback_tx: mpsc::Sender<TaskCommand>,
    ) -> Self {
        Self {
            plugin_registry,
            plugin_managers: Mutex::new(HashMap::new()),
            command_rx,
            feedback_tx,
        }
    }

    /// Run the worker, processing incoming commands.
    pub async fn run(&mut self) {
        info!("Starting task worker");

        while let Some(command) = self.command_rx.recv().await {
            match command {
                TaskCommand::StartTask { task, response } => {
                    // Acknowledge that we received the task.
                    let task_id = task.id.expect("Task must have ID");
                    let _ = response.send(Ok(()));

                    // Execute the task asynchronously.
                    let result = self.execute_task(task).await;

                    // Send result back through the feedback channel.
                    match result {
                        Ok(result_data) => {
                            if let Err(e) = self
                                .feedback_tx
                                .send(TaskCommand::TaskCompleted {
                                    task_id,
                                    result: Ok(result_data),
                                })
                                .await
                            {
                                error!("Failed to send task completion: {}", e);
                            }
                        }
                        Err(e) => {
                            let error = Arc::new(e);

                            if let Err(send_err) = self
                                .feedback_tx
                                .send(TaskCommand::TaskFailed {
                                    task_id,
                                    error: Arc::clone(&error),
                                })
                                .await
                            {
                                error!("Failed to send task failure: {}", send_err);
                            }
                        }
                    }
                }
                TaskCommand::CancelTask { task_id, response } => {
                    let result = self.cancel_task(task_id).await;
                    let _ = response.send(result);
                }
                _ => {
                    warn!("Worker received unexpected command type");
                }
            }
        }

        info!("Task worker shutting down - channel closed");
    }

    /// Execute a task using plugins.
    async fn execute_task(&self, task: Task) -> Result<String> {
        let task_id = task.id.expect("Task mus have ID for execution");
        todo!();
    }

    /// Cancel a running task.
    async fn cancel_task(&self, task_id: i32) -> Result<()> {
        todo!();
    }
}
