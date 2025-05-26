use super::config::WorkerConfig;
use super::handle::WorkerHandle;
use super::WorkerEvent;
use super::{Worker, WorkerId};
use crate::error::Result;
use crate::task::executor::TaskExecutor;
use malbox_database::repositories::tasks::Task;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Weak};
use tokio::sync::mpsc;
use tokio::sync::{Mutex, Notify, RwLock};

/// Pool of workers for task execution.
///
/// The worker pool manages a collection of workers, allocating
/// them to tasks as needed and maintaining a balance between
/// resource utilization and responsiveness. It handles worker
/// lifecycle management, including creation, allocation, and
/// cleanup of idle workers.
pub struct WorkerPool {
    /// All worker handles, indexed by ID.
    workers: RwLock<HashMap<WorkerId, WorkerHandle>>,
    /// Queue of idle worker IDs.
    idle_workers: Mutex<VecDeque<WorkerId>>,
    /// Notifier for when a worker becomes available.
    worker_available_notifier: Arc<Notify>,
    /// Maximum number of workers to create.
    max_workers: usize,
    /// Worker configurations.
    worker_configs: RwLock<HashMap<WorkerId, WorkerConfig>>,
    /// Task executor for workers to use.
    executor: Arc<TaskExecutor>,
    /// Channel for receiving worker events.
    event_rx: Mutex<mpsc::Receiver<WorkerEvent>>,
    /// Channel for sending worker events.
    event_tx: mpsc::Sender<WorkerEvent>,
}

impl WorkerPool {
    /// Create a new worker pool.
    ///
    /// Initializes the pool with the specified executor and
    /// maximum number of workers.
    pub fn new(max_workers: usize, executor: Arc<TaskExecutor>) -> Self {
        let (event_tx, event_rx) = mpsc::channel(100);

        Self {
            workers: RwLock::new(HashMap::new()),
            idle_workers: Mutex::new(VecDeque::new()),
            worker_configs: RwLock::new(HashMap::new()),
            worker_available_notifier: Arc::new(Notify::new()),
            event_rx: Mutex::new(event_rx),
            event_tx,
            executor,
            max_workers,
        }
    }

    /// Start the pool's event processing loop.
    ///
    /// This should be spawned in a tokio task to handle worker events.
    pub async fn run_event_loop(&self) -> Result<()> {
        let mut event_rx = self.event_rx.lock().await;

        while let Some(event) = event_rx.recv().await {
            self.handle_worker_event(event).await?;
        }

        Ok(())
    }

    /// Handle events from workers.
    async fn handle_worker_event(&self, event: WorkerEvent) -> Result<()> {
        match event {
            WorkerEvent::JobCompleted { worker_id, .. }
            | WorkerEvent::BatchCompleted { worker_id, .. } => {
                // Mark worker as idle and add to queue
                self.mark_worker_idle(worker_id).await?;
            }

            WorkerEvent::WorkerShutdown { worker_id, reason } => {
                // Remove worker from pool
                self.remove_worker(worker_id).await?;
                tracing::info!("Worker shutdown: {:?}", reason);
            }

            WorkerEvent::WorkerError { worker_id, error } => {
                tracing::error!("Worker {} error: {:?}", worker_id.as_string(), error);
                // TODO: Handle error - maybe restart worker or mark as failed
            }
        }

        Ok(())
    }

    /// Create a new worker with the given configuration.
    pub async fn create_worker(&self, config: WorkerConfig) -> Result<()> {
        if self.workers.read().await.len() >= self.max_workers {
            return Err(Error::MaxWorkersReached);
        }

        // Create worker
        let (worker, handle, mut event_rx) = Worker::new(config.clone(), self.executor.clone());
        let worker_id = handle.id().clone();

        // Store worker handle and config
        {
            let mut workers = self.workers.write().await;
            workers.insert(worker_id.clone(), handle);
        }

        {
            let mut configs = self.worker_configs.write().await;
            configs.insert(worker_id.clone(), config);
        }

        // Forward worker events to pool
        let event_tx = self.event_tx.clone();
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                let _ = event_tx.send(event).await;
            }
        });

        // Start worker
        tokio::spawn(async move {
            if let Err(e) = worker.run().await {
                tracing::error!("Worker execution error: {:?}", e);
            }
        });

        // Add to idle queue
        {
            let mut idle = self.idle_workers.lock().await;
            idle.push_back(worker_id);
        }

        self.worker_available_notifier.notify_one();

        Ok(())
    }

    /// Acquire a worker for a specific task.
    pub async fn acquire_worker_for_task(&self, task: &Task) -> Result<WorkerHandle> {
        todo!()
    }

    /// Mark a worker as idle.
    async fn mark_worker_idle(&self, worker_id: WorkerId) -> Result<()> {
        {
            let mut idle = self.idle_workers.lock().await;
            idle.push_back(worker_id);
        }

        self.worker_available_notifier.notify_one();
        Ok(())
    }

    /// Remove a worker from the pool.
    async fn remove_worker(&self, worker_id: WorkerId) -> Result<()> {
        // Remove from workers map
        {
            let mut workers = self.workers.write().await;
            workers.remove(&worker_id);
        }

        // Remove from configs
        {
            let mut configs = self.worker_configs.write().await;
            configs.remove(&worker_id);
        }

        // Remove from idle queue
        {
            let mut idle = self.idle_workers.lock().await;
            idle.retain(|id| id != &worker_id);
        }

        Ok(())
    }
}
