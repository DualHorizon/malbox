use super::Result;
//use malbox_core::{ExecutionMode, PluginEvent, PluginManager, PluginRegistry, PluginType};
use super::TaskError;
use malbox_database::repositories::tasks::Task;
use std::sync::Arc;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
};
use tokio::sync::{mpsc, oneshot, Mutex, RwLock, Semaphore};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::TaskCommand;

/// Represents a single worker instance.
struct WorkerInstance {
    id: Uuid,
    tx: mpsc::Sender<TaskCommand>,
    busy: Arc<AtomicBool>,
}

/// Workser pool that manages multiple workers.
pub struct WorkerPool {
    workers: Vec<WorkerInstance>,
    active_tasks: Arc<RwLock<HashMap<i32, Uuid>>>,
    feedback_tx: mpsc::Sender<TaskCommand>,
    semaphore: Arc<Semaphore>,
}

impl WorkerPool {
    pub fn new(num_workers: usize, feedback_tx: mpsc::Sender<TaskCommand>) -> Self {
        let mut workers = Vec::with_capacity(num_workers);
        let active_tasks = Arc::new(RwLock::new(HashMap::new()));
        let semaphore = Arc::new(Semaphore::new(num_workers));

        for id in 0..num_workers {
            let (tx, rx) = mpsc::channel(10);

            let worker_instance = WorkerInstance {
                id: Uuid::new_v4(),
                tx,
                busy: Arc::new(AtomicBool::new(false)),
            };

            let worker_feedback_tx = feedback_tx.clone();

            tokio::spawn(async move {
                info!("Starting worker {}", id);
                let mut worker = TaskWorker::new(rx, worker_feedback_tx);
                worker.run().await;
                warn!("Worker {} terminated", id);
            });

            workers.push(worker_instance);
        }

        Self {
            workers,
            active_tasks,
            feedback_tx,
            semaphore,
        }
    }

    /// Assign a task to an available worker
    pub async fn assign_task(&self, task: Task) -> Result<()> {
        let task_id = task.id.expect("Task must have ID");

        // Acquire a permit from the semaphore to limit concurrent tasks
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| TaskError::Internal("Failed to acquire worker permit".into()))?;

        // Find an available worker
        for worker in &self.workers {
            if !worker.busy.load(Ordering::Relaxed) {
                // Mark worker as busy
                worker.busy.store(true, Ordering::Relaxed);

                // Track this assignment
                {
                    let mut active = self.active_tasks.write().await;
                    active.insert(task_id, worker.id);
                }

                // Send task to this worker
                let (response_tx, response_rx) = oneshot::channel();

                match worker
                    .tx
                    .send(TaskCommand::StartTask {
                        task: task.clone(),
                        response: response_tx,
                    })
                    .await
                {
                    Ok(_) => {
                        // Check worker's acceptance of the task
                        match response_rx.await {
                            Ok(Ok(())) => {
                                // Task accepted - the permit will be dropped when this task completes
                                // We don't need a separate task for cleanup - we'll handle it in feedback
                                return Ok(());
                            }
                            Ok(Err(e)) => {
                                // Worker rejected the task
                                worker.busy.store(false, Ordering::Relaxed);

                                let mut active = self.active_tasks.write().await;
                                active.remove(&task_id);

                                // Permit automatically dropped here
                                return Err(e);
                            }
                            Err(_) => {
                                // Worker disconnected
                                worker.busy.store(false, Ordering::Relaxed);

                                let mut active = self.active_tasks.write().await;
                                active.remove(&task_id);

                                // Permit automatically dropped here
                                return Err(TaskError::Internal("Worker disconnected".into()));
                            }
                        }
                    }
                    Err(_) => {
                        // Channel closed, try next worker
                        worker.busy.store(false, Ordering::Relaxed);
                        continue;
                    }
                }
            }
        }

        // No available workers - permit automatically dropped here
        Err(TaskError::Internal("No available workers".into()))
    }

    /// Cancel a task that's currently running
    pub async fn cancel_task(&self, task_id: i32) -> Result<()> {
        // Find which worker is handling this task
        let worker_id = {
            let active = self.active_tasks.read().await;
            active.get(&task_id).copied()
        };

        if let Some(worker_id) = worker_id {
            if let Some(worker) = self.workers.iter().find(|w| w.id == worker_id) {
                // Send cancel command to the specific worker
                let (response_tx, response_rx) = oneshot::channel();

                worker
                    .tx
                    .send(TaskCommand::CancelTask {
                        task_id,
                        response: response_tx,
                    })
                    .await
                    .map_err(|_| TaskError::Internal("Worker channel closed".into()))?;

                // Wait for worker's response
                match response_rx.await {
                    Ok(result) => result,
                    Err(_) => Err(TaskError::Internal("Worker disconnected".into())),
                }
            } else {
                Err(TaskError::Internal(format!(
                    "Worker for task {} not found",
                    task_id
                )))
            }
        } else {
            Err(TaskError::NotFound(task_id.to_string()))
        }
    }
}

/// Manages task execution.
pub struct TaskWorker {
    //plugin_registry: Arc<PluginRegistry>,
    //plugin_managers: Mutex<HashMap<i32, Arc<PluginManager>>>,
    command_rx: mpsc::Receiver<TaskCommand>,
    feedback_tx: mpsc::Sender<TaskCommand>,
    // config: Arc<Config>,
}

impl TaskWorker {
    /// Create a new TaskWorker.
    pub fn new(
        //plugin_registry: Arc<PluginRegistry>,
        command_rx: mpsc::Receiver<TaskCommand>,
        feedback_tx: mpsc::Sender<TaskCommand>,
    ) -> Self {
        Self {
            // plugin_registry,
            //plugin_managers: Mutex::new(HashMap::new()),
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
