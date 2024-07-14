use malbox_database::repositories::tasks::TaskEntity;
use malbox_database::PgPool;
use scheduler::{TaskScheduler, TaskWorker};
use tokio::sync::mpsc;
mod scheduler;

pub async fn init_scheduler(db: PgPool, max_workers: usize) {
    tracing::info!(
        "[STARTUP] initializing schedulers with {} worker(s)",
        max_workers
    );

    let (tx, rx) = mpsc::channel::<TaskEntity>(8);

    let scheduler = TaskScheduler::new(tx, db.clone());
    tokio::spawn(async move {
        scheduler.scheduler().await;
    });

    let worker = TaskWorker::new(rx, db, max_workers);
    tokio::spawn(async move {
        worker.worker().await;
    });
}
