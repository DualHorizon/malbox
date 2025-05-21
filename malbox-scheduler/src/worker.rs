use crate::error::Result;
use uuid::Uuid;

pub mod handle;
pub mod job;
pub mod pool;

/// Unique identifier for a worker instance.
///
/// Used to track and reference individual workers throughout the system.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WorkerId(Uuid);

impl WorkerId {
    /// Create a new unique worker ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

pub struct Worker {
    id: WorkerId,
    executor: Arc<TaskExecutor>,
    pool: Arc<WorkerPool>,
    job_tx: mpsc::Sender<Job>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Worker {
    pub async fn new(
        config: WorkerConfig,
        executor: Arc<TaskExecutor>,
        pool: Arc<WorkerPool>,
    ) -> Result<Self> {
        let (job_tx, mut job_rx) = mpsc::channel(16);
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();

        let id = WorkerId::new();
        let executor_clone = executor.clone();
        let pool_clone = pool.clone();
        let worker_id = id.clone();

        // Spawn worker task
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(job) = job_rx.recv() => {
                        let result = executor_clone.execute(job.task, job.resources).await;
                        let _ = job.result_tx.send(result);

                        // Return worker to pool
                        let _ = pool_clone.release_worker(worker_id.clone()).await;
                    }

                    _ = &mut shutdown_rx => {
                        break;
                    }
                }
            }
        });

        Ok(Self {
            id,
            executor,
            pool,
            job_tx,
            shutdown_tx: Some(shutdown_tx),
        })
    }

    pub async fn execute(
        &self,
        context: ExecutionContext,
    ) -> Result<oneshot::Receiver<Result<TaskResult>>> {
        let (result_tx, result_rx) = oneshot::channel();

        let job = Job {
            task: context.task,
            resources: context.resources,
            result_tx,
        };

        self.job_tx.send(job).await?;

        Ok(result_rx)
    }

    pub fn from_handle(handle: WorkerHandle) -> Self {
        Self {
            id: handle.id().clone(),
            executor: Arc::new(TaskExecutor::default()), // Placeholder, should be passed in
            pool: Arc::new(WorkerPool::default()),       // Placeholder, should be passed in
            job_tx: handle.sender.clone(),
            shutdown_tx: None,
        }
    }

    pub fn handle(&self) -> WorkerHandle {
        WorkerHandle::new(self.id.clone(), self.job_tx.clone())
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}
