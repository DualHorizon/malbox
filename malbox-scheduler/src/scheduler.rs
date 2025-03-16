use crate::resource::{Resource, ResourceError, ResourceKind, ResourceManager};
use crate::task::{Task, TaskError, TaskManager, TaskOptions, TaskState};
use crate::worker::WorkerPool;
use malbox_core::plugin::{LoadedPlugin, PluginError, PluginManager, PluginState};

use malbox_config::Config;
use malbox_database::PgPool;
use std::sync::Arc;
use tracing::info;

pub struct Scheduler {
    pub resource_manager: Arc<ResourceManager>,
    pub task_manager: Arc<TaskManager>,
    pub plugin_manager: Arc<PluginManager>,
    worker_pool: Option<WorkerPool>,
}

impl Scheduler {
    pub async fn new(config: Config, db: PgPool) -> Result<Self, Box<dyn std::error::Error>> {
        let resource_manager = Arc::new(ResourceManager::new(db.clone(), config.clone()));
        resource_manager.initialize().await?;

        let plugin_manager = Arc::new(PluginManager::new(config.clone())?);
        plugin_manager.initialize().await?;

        let (task_manager, worker_rx, worker_tx) =
            TaskManager::new(db.clone(), config.clone(), resource_manager.clone());
        let task_manager = Arc::new(task_manager);

        let worker_pool = WorkerPool::new(
            config.clone(),
            resource_manager.clone(),
            plugin_manager.clone(),
            worker_rx,
            worker_tx,
        );

        Ok(Self {
            resource_manager,
            task_manager,
            plugin_manager,
            worker_pool: Some(worker_pool),
        })
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting malbox scheduler system");

        self.task_manager.start().await?;

        if let Some(worker_pool) = self.worker_pool.take() {
            tokio::spawn(async move {
                let mut pool = worker_pool;
                pool.start().await;
            });
        }

        info!("Scheduler system started successfully");
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Shutting down malbox scheduler");

        self.plugin_manager.unload_all_plugins().await?;

        info!("Scheduler shutdown complete");
        Ok(())
    }
}
