use crate::resource::{ResourceError, ResourceManager};
use malbox_config::Config;
use malbox_database::{
    repositories::{
        machinery::MachinePlatform,
        samples::SampleEntity,
        tasks::{fetch_pending_tasks, fetch_task, update_task_status, StatusType, TaskEntity},
    },
    PgPool,
};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use thiserror::Error;
use time::OffsetDateTime;
use tokio::sync::{mpsc, oneshot, RwLock};
use tracing::{debug, error, info, warn};

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("Task not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] malbox_database::error::DatabaseError),
    #[error("Resource error: {0}")]
    Resource(#[from] ResourceError),
    #[error("Plugin error: {0}")]
    Plugin(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Task canceled")]
    Canceled,
}

pub type Result<T> = std::result::Result<T, TaskError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskState {
    Pending,
    Initializing,
    PreparingResources,
    Running,
    Stopping,
    Completed,
    Failed(String),
    Canceled,
}

#[derive(Debug, Clone)]
pub struct TaskOptions {
    pub platform: Option<MachinePlatform>,
    pub timeout: Option<u64>,
    pub machine: Option<String>,
    pub memory: bool,
    pub enforce_timeout: bool,
    pub packages: Vec<String>,
    pub plugins: Vec<String>,
    pub custom_options: HashMap<String, String>,
}

impl Default for TaskOptions {
    fn default() -> Self {
        Self {
            platform: None,
            timeout: None,
            machine: None,
            memory: false,
            enforce_timeout: false,
            packages: Vec::new(),
            plugins: Vec::new(),
            custom_options: HashMap::new(),
        }
    }
}

impl From<TaskEntity> for TaskOptions {
    fn from(entity: TaskEntity) -> Self {
        let platform = if let Some(platform) = &entity.platform {
            match platform.as_str() {
                "windows" => Some(MachinePlatform::Windows),
                "linux" => Some(MachinePlatform::Linux),
                _ => None,
            }
        } else {
            None
        };

        let mut packages = Vec::new();
        if let Some(package) = &entity.package {
            packages.push(package.clone());
        }

        let options = if let Some(opts) = &entity.options {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(opts) {
                map
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

        Self {
            platform,
            timeout: Some(entity.timeout as u64),
            machine: entity.machine.clone(),
            memory: entity.memory,
            enforce_timeout: entity.enforce_timeout,
            packages,
            plugins: Vec::new(),
            custom_options: options,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: i32,
    pub target: String,
    pub module: String,
    pub sample_id: Option<i64>,
    pub sample: Option<SampleEntity>,
    pub state: TaskState,
    pub options: TaskOptions,
    pub created_at: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub completed_at: Option<OffsetDateTime>,
    pub result: Option<String>,
}

impl From<TaskEntity> for Task {
    fn from(entity: TaskEntity) -> Self {
        Task {
            id: entity.id,
            target: entity.target.clone(),
            module: entity.module.clone(),
            sample_id: entity.sample_id,
            sample: None,
            state: match entity.status {
                StatusType::Pending => TaskState::Pending,
                StatusType::Processing => TaskState::Running,
                StatusType::Success => TaskState::Completed,
                StatusType::Failure => TaskState::Failed("Unknown failure".to_string()),
            },
            options: TaskOptions::from(entity.clone()),
            created_at: entity
                .added_on
                .map(|dt| dt.assume_utc())
                .unwrap_or_else(OffsetDateTime::now_utc),
            started_at: entity.started_on.map(|dt| dt.assume_utc()),
            completed_at: entity.completed_on.map(|dt| dt.assume_utc()),
            result: None,
        }
    }
}

#[derive(Debug)]
pub enum TaskCommand {
    StartTask {
        task: Task,
        response: oneshot::Sender<Result<()>>,
    },
    CancelTask {
        task_id: i32,
        response: oneshot::Sender<Result<()>>,
    },
    TaskCompleted {
        task_id: i32,
        result: Result<String>,
    },
    TaskFailed {
        task_id: i32,
        error: Arc<TaskError>,
    },
    TaskProgress {
        task_id: i32,
        progress: u8,
        message: String,
    },
}

pub struct TaskManager {
    db: PgPool,
    config: Config,
    resource_manager: Arc<ResourceManager>,
    tasks: RwLock<HashMap<i32, Task>>,
    task_queue: RwLock<VecDeque<i32>>,
    task_tx: mpsc::Sender<TaskCommand>,
    task_feedback_rx: mpsc::Receiver<TaskCommand>,
}

impl TaskManager {
    pub fn new(
        db: PgPool,
        config: Config,
        resource_manager: Arc<ResourceManager>,
    ) -> (Self, mpsc::Receiver<TaskCommand>, mpsc::Sender<TaskCommand>) {
        let (task_tx, worker_rx) = mpsc::channel(100);
        let (worker_tx, task_feedback_rx) = mpsc::channel(100);

        (
            Self {
                db,
                config,
                resource_manager,
                tasks: RwLock::new(HashMap::new()),
                task_queue: RwLock::new(VecDeque::new()),
                task_tx,
                task_feedback_rx,
            },
            worker_rx,
            worker_tx,
        )
    }

    pub async fn start(&self) -> Result<()> {
        self.load_pending_tasks().await?;

        self.process_feedback_loop();

        self.scheduler_loop();

        Ok(())
    }

    async fn load_pending_tasks(&self) -> Result<()> {
        let pending_tasks = fetch_pending_tasks(&self.db).await?;

        let mut tasks = self.tasks.write().await;
        let mut queue = self.task_queue.write().await;

        for entity in pending_tasks {
            let task = Task::from(entity);
            let task_id = task.id;

            tasks.insert(task_id, task);
            queue.push_back(task_id);
        }

        info!("Loaded {} pending tasks from database", queue.len());
        Ok(())
    }

    fn process_feedback_loop(&self) {
        tokio::spawn(async move {
            while let Some(command) = self.task_feedback_rx.recv().await {
                match command {
                    TaskCommand::TaskCompleted { task_id, result } => {
                        if let Err(e) = self.handle_task_completed(task_id, result).await {
                            error!("Error handling task completion: {}", e);
                        }
                    }
                    TaskCommand::TaskFailed { task_id, error } => {
                        if let Err(e) = self
                            .handle_task_failed(task_id, Arc::try_unwrap(error).unwrap())
                            .await
                        {
                            error!("Error handling task failure: {}", e);
                        }
                    }
                    TaskCommand::TaskProgress {
                        task_id,
                        progress,
                        message,
                    } => {
                        debug!("Task {} progress: {}% - {}", task_id, progress, message);
                    }
                    _ => { /* Ignore other commands */ }
                }
            }
        });
    }

    fn scheduler_loop(&self) {
        let task_manager = self.clone();

        tokio::spawn(async move {
            loop {
                if let Err(e) = task_manager.schedule_next_task().await {
                    error!("Error scheduling task: {}", e);
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            }
        });
    }

    async fn schedule_next_task(&self) -> Result<()> {
        let task_id = {
            let mut queue = self.task_queue.write().await;
            queue.pop_front()
        };

        if let Some(task_id) = task_id {
            let task = {
                let tasks = self.tasks.read().await;
                tasks.get(&task_id).cloned()
            };

            if let Some(task) = task {
                info!("Scheduling task {} for execution", task_id);

                let (response_tx, response_rx) = oneshot::channel();
                self.task_tx
                    .send(TaskCommand::StartTask {
                        task,
                        response: response_tx,
                    })
                    .await
                    .map_err(|_| {
                        TaskError::Internal("Failed to send task to worker".to_string())
                    })?;

                match response_rx.await {
                    Ok(Ok(())) => {
                        debug!("Task {} accepted by worker", task_id);

                        update_task_status(&self.db, task_id, StatusType::Processing).await?;
                    }
                    Ok(Err(e)) => {
                        error!("Worker rejected task {}: {}", task_id, e);

                        let mut queue = self.task_queue.write().await;
                        queue.push_back(task_id);

                        return Err(e);
                    }
                    Err(_) => {
                        error!("Worker channel closed while waiting for response");

                        let mut queue = self.task_queue.write().await;
                        queue.push_back(task_id);

                        return Err(TaskError::Internal("Worker channel closed".to_string()));
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_task_completed(&self, task_id: i32, result: Result<String>) -> Result<()> {
        match result {
            Ok(result_str) => {
                info!("Task {} completed successfully", task_id);

                {
                    let mut tasks = self.tasks.write().await;
                    if let Some(task) = tasks.get_mut(&task_id) {
                        task.state = TaskState::Completed;
                        task.completed_at = Some(OffsetDateTime::now_utc());
                        task.result = Some(result_str.clone());
                    }
                }

                update_task_status(&self.db, task_id, StatusType::Success).await?;

                self.resource_manager
                    .release_resources(&task_id.to_string())
                    .await?;

                Ok(())
            }
            Err(e) => {
                error!("Task {} completed with error: {}", task_id, e);
                self.handle_task_failed(task_id, e).await
            }
        }
    }

    async fn handle_task_failed(&self, task_id: i32, error: TaskError) -> Result<()> {
        error!("Task {} failed: {}", task_id, error);

        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.state = TaskState::Failed(error.to_string());
                task.completed_at = Some(OffsetDateTime::now_utc());
            }
        }

        update_task_status(&self.db, task_id, StatusType::Failure).await?;

        self.resource_manager
            .release_resources(&task_id.to_string())
            .await?;

        Ok(())
    }

    pub async fn submit_task(&self, task_entity: TaskEntity) -> Result<i32> {
        let task = Task::from(task_entity);
        let task_id = task.id;

        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id, task);
        }

        {
            let mut queue = self.task_queue.write().await;
            queue.push_back(task_id);
        }

        info!("Task {} submitted to queue", task_id);
        Ok(task_id)
    }

    pub async fn cancel_task(&self, task_id: i32) -> Result<()> {
        {
            let mut queue = self.task_queue.write().await;
            if let Some(pos) = queue.iter().position(|id| *id == task_id) {
                queue.remove(pos);

                {
                    let mut tasks = self.tasks.write().await;
                    if let Some(task) = tasks.get_mut(&task_id) {
                        task.state = TaskState::Canceled;
                    }
                }

                info!("Task {} removed from queue and canceled", task_id);
                return Ok(());
            }
        }

        let (response_tx, response_rx) = oneshot::channel();
        self.task_tx
            .send(TaskCommand::CancelTask {
                task_id,
                response: response_tx,
            })
            .await
            .map_err(|_| {
                TaskError::Internal("Failed to send cancel command to worker".to_string())
            })?;

        match response_rx.await {
            Ok(result) => result,
            Err(_) => Err(TaskError::Internal(
                "Worker channel closed while waiting for response".to_string(),
            )),
        }
    }

    pub async fn get_task(&self, task_id: i32) -> Result<Task> {
        {
            let tasks = self.tasks.read().await;
            if let Some(task) = tasks.get(&task_id) {
                return Ok(task.clone());
            }
        }

        let entity = fetch_task(&self.db, task_id)
            .await?
            .ok_or_else(|| TaskError::NotFound(task_id.to_string()))?;

        let task = Task::from(entity);

        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id, task.clone());
        }

        Ok(task)
    }
}
