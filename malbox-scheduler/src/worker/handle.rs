use super::job::Job;
use super::WorkerId;
use crate::error::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tokio::sync::{Mutex, RwLock};

/// Handle to a worker instance that allows control over the worker.
///
/// The handle provides a lightweight way to reference and control a worker
/// without owning the worker itself. Multiple components can hold handles
/// to the same worker, allowing for distributed control and management.
#[derive(Clone)]
pub struct WorkerHandle {
    /// Unique identifier for the worker.
    pub id: WorkerId,
    /// Channel for sending jobs to the worker.
    pub job_tx: mpsc::Sender<Job>,
    /// Channel for signaling worker shutdown.
    pub shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl WorkerHandle {
    pub fn new(id: WorkerId, job_tx: mpsc::Sender<Job>, shutdown_tx: oneshot::Sender<()>) -> Self {
        Self {
            id,
            job_tx,
            shutdown_tx: Arc::new(Mutex::new(Some(shutdown_tx))),
        }
    }

    /// Send a job to the worker.
    pub async fn send_job(&self, job: Job) -> Result<()> {
        self.job_tx
            .send(job)
            .await
            .map_err(|_| Error::WorkerUnavailable)
    }

    /// Request worker shutdown.
    pub async fn shutdown(&self) -> Result<()> {
        let mut shutdown_opt = self.shutdown_tx.lock().await;
        if let Some(tx) = shutdown_opt.take() {
            tx.send(()).map_err(|_| Error::WorkerUnavailable)?;
        }
        Ok(())
    }

    /// Get worker ID.
    pub fn id(&self) -> &WorkerId {
        &self.id
    }

    /// Check if worker is busy.
    pub async fn is_busy(&self) -> bool {
        // let status = self.status.read().await;
        // status.is_busy
        todo!()
    }
}
