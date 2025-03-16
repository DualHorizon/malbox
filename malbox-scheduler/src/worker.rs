use crate::{
    resource::{Resource, ResourceError, ResourceKind, ResourceManager},
    task::{Result, Task, TaskCommand, TaskError, TaskState},
};
use malbox_config::Config;
use malbox_core::plugin::{PluginError, PluginManager};
use malbox_machinery::machinery::kvm::{shutdown_machine, start_machine};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

const MAX_WORKERS: usize = 10;

pub struct WorkerPool {
    config: Config,
    resource_manager: Arc<ResourceManager>,
    plugin_manager: Arc<PluginManager>,
    task_rx: mpsc::Receiver<TaskCommand>,
    feedback_tx: mpsc::Sender<TaskCommand>,
    semaphore: Arc<Semaphore>,
    running_tasks: Arc<RwLock<HashMap<i32, RunningTask>>>,
}

struct RunningTask {
    pub task: Task,
    pub vm: Option<Resource>,
    pub state: TaskState,
    pub plugins: HashSet<String>,
    pub start_time: std::time::Instant,
}

impl WorkerPool {
    pub fn new(
        config: Config,
        resource_manager: Arc<ResourceManager>,
        plugin_manager: Arc<PluginManager>,
        task_rx: mpsc::Receiver<TaskCommand>,
        feedback_tx: mpsc::Sender<TaskCommand>,
    ) -> Self {
        let max_workers = config.analysis.max_vms as usize;

        Self {
            config,
            resource_manager,
            plugin_manager,
            task_rx,
            feedback_tx,
            semaphore: Arc::new(Semaphore::new(max_workers)),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(&mut self) {
        info!("Starting worker pool with {} workers", MAX_WORKERS);

        while let Some(command) = self.task_rx.recv().await {
            match command {
                TaskCommand::StartTask { task, response } => {
                    match self.semaphore.clone().try_acquire_owned() {
                        Ok(permit) => {
                            let worker = self.create_worker(
                                task.clone(),
                                self.resource_manager.clone(),
                                self.plugin_manager.clone(),
                                self.feedback_tx.clone(),
                                self.running_tasks.clone(),
                            );

                            {
                                let mut running_tasks = self.running_tasks.write().await;
                                running_tasks.insert(
                                    task.id,
                                    RunningTask {
                                        task: task.clone(),
                                        vm: None,
                                        state: TaskState::Initializing,
                                        plugins: HashSet::new(),
                                        start_time: std::time::Instant::now(),
                                    },
                                );
                            }

                            let _ = response.send(Ok(()));

                            tokio::spawn(async move {
                                let result = worker.execute().await;

                                if let Err(e) = &result {
                                    error!("Worker for task {} failed: {}", task.id, e);
                                }

                                drop(permit);
                            });
                        }
                        Err(_) => {
                            let _ = response.send(Err(TaskError::Internal(
                                "No worker available, at maximum capacity".to_string(),
                            )));
                        }
                    }
                }
                TaskCommand::CancelTask { task_id, response } => {
                    let is_running = {
                        let running_tasks = self.running_tasks.read().await;
                        running_tasks.contains_key(&task_id)
                    };

                    if is_running {
                        {
                            let mut running_tasks = self.running_tasks.write().await;
                            if let Some(running_task) = running_tasks.get_mut(&task_id) {
                                running_task.state = TaskState::Canceled;
                            }
                        }

                        let _ = self
                            .feedback_tx
                            .send(TaskCommand::TaskFailed {
                                task_id,
                                error: TaskError::Canceled,
                            })
                            .await;

                        let _ = response.send(Ok(()));
                    } else {
                        let _ = response.send(Err(TaskError::NotFound(task_id.to_string())));
                    }
                }
                _ => {}
            }
        }
    }

    fn create_worker(
        &self,
        task: Task,
        resource_manager: Arc<ResourceManager>,
        plugin_manager: Arc<PluginManager>,
        feedback_tx: mpsc::Sender<TaskCommand>,
        running_tasks: Arc<RwLock<HashMap<i32, RunningTask>>>,
    ) -> Worker {
        Worker {
            task,
            resource_manager,
            plugin_manager,
            feedback_tx,
            running_tasks,
        }
    }
}

struct Worker {
    task: Task,
    resource_manager: Arc<ResourceManager>,
    plugin_manager: Arc<PluginManager>,
    feedback_tx: mpsc::Sender<TaskCommand>,
    running_tasks: Arc<RwLock<HashMap<i32, RunningTask>>>,
}

impl Worker {
    async fn execute(self) -> Result<()> {
        let task_id = self.task.id;

        info!("Starting execution of task {}", task_id);

        self.update_task_state(TaskState::Initializing).await?;

        self.send_progress(0, "Initializing task").await?;

        if self.is_task_canceled().await {
            return Err(TaskError::Canceled);
        }

        self.update_task_state(TaskState::PreparingResources)
            .await?;
        self.send_progress(10, "Allocating resources").await?;

        let vm = match self.allocate_resources().await {
            Ok(vm) => {
                {
                    let mut running_tasks = self.running_tasks.write().await;
                    if let Some(running_task) = running_tasks.get_mut(&task_id) {
                        running_task.vm = Some(vm.clone());
                    }
                }

                vm
            }
            Err(e) => {
                error!("Failed to allocate resources for task {}: {}", task_id, e);
                self.send_task_failed(e).await?;
                return Err(e);
            }
        };

        if self.is_task_canceled().await {
            self.cleanup_resources().await?;
            return Err(TaskError::Canceled);
        }

        self.send_progress(20, "Starting VM").await?;

        let vm_name = vm.name.clone();
        if let Err(e) = self.start_vm(&vm_name, vm.snapshot()).await {
            error!("Failed to start VM for task {}: {}", task_id, e);
            self.send_task_failed(TaskError::Resource(ResourceError::VMOperation(
                e.to_string(),
            )))
            .await?;
            self.cleanup_resources().await?;
            return Err(TaskError::Resource(ResourceError::VMOperation(
                e.to_string(),
            )));
        }

        self.send_progress(30, "Loading analysis plugins").await?;

        let plugins = self.task.options.plugins.clone();

        if let Err(e) = self.start_plugins(&plugins).await {
            error!("Failed to start plugins for task {}: {}", task_id, e);
            self.send_task_failed(TaskError::Plugin(e.to_string()))
                .await?;
            self.cleanup_resources().await?;
            return Err(TaskError::Plugin(e.to_string()));
        }

        self.update_task_state(TaskState::Running).await?;
        self.send_progress(40, "Running analysis").await?;

        for progress in (40..=90).step_by(10) {
            if self.is_task_canceled().await {
                self.cleanup_resources().await?;
                return Err(TaskError::Canceled);
            }

            sleep(Duration::from_millis(1000)).await;

            self.send_progress(progress, &format!("Analysis in progress: {}%", progress))
                .await?;
        }

        self.send_progress(95, "Finalizing analysis").await?;

        self.stop_plugins().await?;
        self.cleanup_resources().await?;

        self.send_progress(100, "Analysis completed").await?;

        let result = format!("Analysis of task {} completed successfully", task_id);
        self.feedback_tx
            .send(TaskCommand::TaskCompleted {
                task_id,
                result: Ok(result),
            })
            .await
            .map_err(|_| TaskError::Internal("Failed to send task completion".to_string()))?;

        {
            let mut running_tasks = self.running_tasks.write().await;
            running_tasks.remove(&task_id);
        }

        info!("Task {} completed successfully", task_id);
        Ok(())
    }

    async fn allocate_resources(&self) -> Result<Resource> {
        let platform = self.task.options.platform.clone();

        let machine_name = self.task.options.machine.as_deref();

        let vm = self
            .resource_manager
            .allocate_vm_for_task(&self.task.id.to_string(), platform, machine_name)
            .await?;

        info!("Allocated VM '{}' for task {}", vm.name, self.task.id);
        Ok(vm)
    }

    async fn start_vm(&self, vm_name: &str, snapshot: Option<&str>) -> Result<()> {
        info!("Starting VM '{}' for task {}", vm_name, self.task.id);

        start_machine(vm_name, snapshot.map(String::from))
            .await
            .map_err(|e| TaskError::Resource(ResourceError::VMOperation(e.to_string())))
    }

    async fn start_plugins(&self, plugin_names: &[String]) -> Result<()> {
        info!(
            "Starting plugins for task {}: {:?}",
            self.task.id, plugin_names
        );

        if let Err(e) = self
            .plugin_manager
            .start_plugins_for_task(&self.task.id.to_string(), plugin_names.to_vec())
            .await
        {
            return Err(TaskError::Plugin(e.to_string()));
        }

        {
            let mut running_tasks = self.running_tasks.write().await;
            if let Some(running_task) = running_tasks.get_mut(&self.task.id) {
                running_task.plugins = self
                    .plugin_manager
                    .get_plugins_for_task(&self.task.id.to_string());
            }
        }

        Ok(())
    }

    async fn stop_plugins(&self) -> Result<()> {
        info!("Stopping plugins for task {}", self.task.id);

        if let Err(e) = self
            .plugin_manager
            .stop_plugins_for_task(&self.task.id.to_string())
            .await
        {
            return Err(TaskError::Plugin(e.to_string()));
        }

        Ok(())
    }

    async fn cleanup_resources(&self) -> Result<()> {
        let task_id = self.task.id;
        info!("Cleaning up resources for task {}", task_id);

        let vm_name = {
            let running_tasks = self.running_tasks.read().await;
            running_tasks
                .get(&task_id)
                .and_then(|t| t.vm.as_ref())
                .map(|vm| vm.name.clone())
        };

        if let Some(vm_name) = vm_name {
            info!("Stopping VM '{}' for task {}", vm_name, task_id);

            if let Err(e) = shutdown_machine(&vm_name).await {
                warn!("Error shutting down VM '{}': {}", vm_name, e);
            }
        }

        self.resource_manager
            .release_resources(&task_id.to_string())
            .await?;

        Ok(())
    }

    async fn update_task_state(&self, state: TaskState) -> Result<()> {
        let mut running_tasks = self.running_tasks.write().await;

        if let Some(running_task) = running_tasks.get_mut(&self.task.id) {
            running_task.state = state;
            Ok(())
        } else {
            Err(TaskError::NotFound(self.task.id.to_string()))
        }
    }

    async fn is_task_canceled(&self) -> bool {
        let running_tasks = self.running_tasks.read().await;

        if let Some(running_task) = running_tasks.get(&self.task.id) {
            running_task.state == TaskState::Canceled
        } else {
            false
        }
    }

    async fn send_progress(&self, progress: u8, message: &str) -> Result<()> {
        debug!(
            "Task {} progress: {}% - {}",
            self.task.id, progress, message
        );

        self.feedback_tx
            .send(TaskCommand::TaskProgress {
                task_id: self.task.id,
                progress,
                message: message.to_string(),
            })
            .await
            .map_err(|_| TaskError::Internal("Failed to send progress update".to_string()))
    }

    async fn send_task_failed(&self, error: &TaskError) -> Result<()> {
        self.feedback_tx
            .send(TaskCommand::TaskFailed {
                task_id: self.task.id,
                error,
            })
            .await
            .map_err(|_| TaskError::Internal("Failed to send task failure".to_string()))
    }
}
