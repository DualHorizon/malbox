use super::{store::TaskStore, worker::WorkerPool, Result, TaskError};
use crate::resource::{self, ResourceError, ResourceManager};
use malbox_core::PluginRegistry;
use malbox_database::repositories::tasks::{Task, TaskState};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, warn};

/// The TaskExecutor manages the actual execution of tasks and their resources.
pub struct TaskExecutor {
    store: Arc<TaskStore>,
    plugin_registry: Arc<PluginRegistry>,
}

impl TaskExecutor {
    pub async fn execute(&self, task: Task, resources: ResourceAllocation) -> Result<TaskResult> {
        // Prepare execution environment
        // let sandbox = self.machinery.create_sandbox(&resources).await?;

        // Update task status
        self.store
            .update_task_state(task.id.expect("Task ID required"), TaskState::Running)
            .await?;

        // Get plugins for this task
        let plugins = self.plugin_registry.get_plugins_for_task(&task).await?;

        // Execute plugins in order
        let mut result = TaskResult::new(task.id.clone());

        for plugin in plugins {
            let context = PluginContext {
                task: task.clone(),
                sandbox: sandbox.clone(),
                resources: resources.clone(),
            };

            let plugin_result = plugin.execute(context).await?;
            result.add_plugin_result(plugin.id(), plugin_result);

            // Check if we should continue
            if plugin_result.status == PluginStatus::Failed && task.stop_on_plugin_failure {
                break;
            }
        }

        // Update task status
        let final_status = if result.has_failures() {
            TaskState::Failed
        } else {
            TaskState::Completed
        };

        self.store
            .update_task_state(task.id.expect("Task ID required"), final_status)
            .await?;

        // Release resources
        self.resource_manager.release(&task.id).await?;

        Ok(result)
    }
}
