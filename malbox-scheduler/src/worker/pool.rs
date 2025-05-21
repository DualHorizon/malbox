use super::job::Worker;
use crate::error::Result;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify, RwLock};

/// TODO:
/// - Worker metadata, workers can be created for
/// specific tasks only!

/// Worker pool that manages all workers.
pub struct WorkerPool {
    workers: RwLock<HashMap<WorkerId, WorkerHandle>>,
    idle_workers: Mutex<VecDeque<WorkerId>>,
    worker_available_notifier: Arc<Notify>,
    max_workers: usize,
    config: WorkerConfig,
}

impl WorkerPool {
    /// Acquire a worker from the pool.
    /// If no idle workers are available, it either:
    /// - Creates a new worker if the number of total
    /// workers in under the max_workers limit.
    /// - Waits for a worker to become available.
    pub async fn acquire_worker(&self) -> Result<Worker> {
        // Try to get an idle worker
        {
            let mut idle = self.idle_workers.lock().await;
            if let Some(worker_id) = idle.pop_front() {
                let workers = self.workers.read().await;
                if let Some(handle) = workers.get(&worker_id) {
                    return Ok(Worker::from_handle(handle.clone()));
                }
            }
        }

        // Try to create a new worker if under limit
        {
            let workers = self.workers.read().await;
            if workers.len() < self.max_workers {
                drop(workers);
                return self.create_worker().await;
            }
        }

        // Wait for a worker to become available
        loop {
            let notifier = self.worker_available_notifier.clone();
            notifier.notified().await;

            let mut idle = self.idle_workers.lock().await;
            if let Some(worker_id) = idle.pop_front() {
                let workers = self.workers.read().await;
                if let Some(handle) = workers.get(&worker_id) {
                    return Ok(Worker::from_handle(handle.clone()));
                }
            }
        }
    }

    /// Release a worker and notify that it's available again.
    pub async fn release_worker(&self, worker_id: WorkerId) -> Result<()> {
        let mut idle = self.idle_workers.lock().await;
        idle.push_back(worker_id);

        // Notify waiters
        self.worker_available_notifier.notify_one();

        Ok(())
    }

    /// Create a worker and insert it in the worker pool.
    async fn create_worker(&self) -> Result<Worker> {
        let worker = Worker::new(self.config.clone()).await?;

        {
            let mut workers = self.workers.write().await;
            workers.insert(worker.id(), worker.handle());
        }

        Ok(worker)
    }
}
