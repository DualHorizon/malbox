use super::{worker::WorkerPool, Result, TaskError};
use crate::resource::{self, ResourceError, ResourceManager};
use malbox_database::repositories::tasks::Task;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, warn};

use super::TaskCommand;

/// The TaskExecutor manages the actual execution of tasks and their resources.
pub struct TaskExecutor {
    // Resource manager for allocating VMs, network, etc.
    resource_manager: Arc<ResourceManager>,
    // Channel for sending comands to worker processes.
    worker_pool: Arc<WorkerPool>,
    // Channel for sending results back to the parent system.
    feedback_tx: mpsc::Sender<TaskCommand>,
    // Config for execution settings (timeouts, retry, policies, etc.)
    // config: Config,
}

impl TaskExecutor {
    pub fn new(
        resource_manager: Arc<ResourceManager>,
        worker_pool: Arc<WorkerPool>,
        feedback_tx: mpsc::Sender<TaskCommand>,
        // config: Config,
    ) -> Self {
        Self {
            resource_manager,
            worker_pool,
            feedback_tx,
        }
    }

    /// Execute a task, handling resource allocation and worker delegation.
    pub async fn execute_task(&self, task: Task) -> Result<()> {
        let task_id = task.id.expect("Task must have an ID for execution");

        // Notify that we're starting preparation.
        self.send_progress_update(task_id, 0, "Preparing task execution")
            .await?;

        // Allocate resources for the task based on its requirements/metadata.
        // Might involve provisioning VMs, networks, or other resources.
        self.send_progress_update(task_id, 10, "Allocating resources")
            .await?;

        // NOTE: Should we allocate resources here?!
        // It would be nice if a profile could decide of this - or plugins.

        // let resource_result = self.allocate_resources(&task).await;

        // If resource allocation fails, report failure and exit.
        // if let Err(e) = resource_result {
        //     // Wrap the error in an Arc since we need to clone it.
        //     // Note that we can't implement Clone on our error type.
        //     let error = Arc::new(TaskError::Resource(ResourceError::AllocationFailed(
        //         format!("Failed to allocate resources: {}", e),
        //     )));
        //     self.report_task_failure(task_id, &error).await?;
        //     return Err(Arc::try_unwrap(error).expect("Failed to unwrap error"));
        // }

        self.send_progress_update(task_id, 20, "Resources allocated")
            .await?;

        // Prepare the task for execution (e.g, copy files, set up environment etc.)
        // let prepare_result = self.prepare_task_env(&task).await;
        // if let Err(e) = prepare_result {
        //     // On preparation failure, release resources and report error.
        //     self.release_resources(&task).await?;
        //     let error = Arc::new(TaskError::Internal(format!(
        //         "Failed to prepare task environment: {}",
        //         e
        //     )));
        //     self.report_task_failure(task_id, &error).await?;
        //     return Err(Arc::try_unwrap(error).expect("Failed to unwrap error"));
        // }

        // Send task to worker through the worker_tx channel.
        if let Err(e) = self.worker_pool.assign_task(task.clone()).await {
            self.release_resources(&task).await?;
            return Err(e);
        }

        self.send_progress_update(task_id, 40, "Task dispatched to worker")
            .await?;

        Ok(())
    }

    /// Prepare the environment for task execution.
    async fn prepare_task_env(&self, task: &Task) -> Result<()> {
        todo!();
    }

    /// Send a task to a worker.
    // async fn send_to_worker(&self, task: &Task) -> Result<()> {
    //     // Create a oneshot channel for the worker to respond.
    //     let (response_tx, response_rx) = oneshot::channel();

    //     let command = TaskCommand::StartTask {
    //         task: task.clone(),
    //         response: response_tx,
    //     };

    //     // Send the command to the worker.
    //     self.worker_tx
    //         .send(command)
    //         .await
    //         .map_err(|e| TaskError::Internal(format!("Worker channel closed: {}", e).into()))?;

    //     // Wait for the worker to accept the task.
    //     match response_rx.await {
    //         Ok(Ok(())) => Ok(()),
    //         Ok(Err(e)) => Err(e),
    //         Err(_) => Err(TaskError::Internal("Worker disconnected".into())),
    //     }
    // }

    /// Cancel a running task.
    pub async fn cancel_task(&self, task_id: i32) -> Result<()> {
        self.worker_pool.cancel_task(task_id).await
    }

    /// Allocate resources needed for task execution.
    async fn allocate_resources(&self, task: &Task) -> Result<()> {
        let task_id = task.id.expect("Task must have an ID for execution");

        // Using the resource manager to provision what's needed.
        // This depends on task fields/metadata like platform, memory, etc.

        self.resource_manager
            .allocate_vm_for_task(task_id, Some(task.platform.clone()), None)
            .await?;

        Ok(())
    }

    /// Release resources after task completion or failure.
    async fn release_resources(&self, task: &Task) -> Result<()> {
        let task_id = task.id.expect("Task must have an ID for execution");

        self.resource_manager.release_resources(task_id).await?;
        Ok(())
    }

    /// Send a progress update about a task.
    async fn send_progress_update(&self, task_id: i32, progress: u8, message: &str) -> Result<()> {
        let command = TaskCommand::TaskProgress {
            task_id,
            progress,
            message: message.to_string(),
        };

        self.feedback_tx
            .send(command)
            .await
            .map_err(|_| TaskError::Internal("Feedback channel closed".into()))?;

        Ok(())
    }

    /// Report a task failure.
    async fn report_task_failure(&self, task_id: i32, error: &Arc<TaskError>) -> Result<()> {
        let command = TaskCommand::TaskFailed {
            task_id,
            error: Arc::clone(error),
        };

        self.feedback_tx
            .send(command)
            .await
            .map_err(|_| TaskError::Internal("Feedback channel closed".into()))?;

        Ok(())
    }
}
