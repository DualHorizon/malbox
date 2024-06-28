use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    Semaphore,
};

use crate::repositories::{self, tasks_repository::TaskEntity};

pub struct TaskScheduler {
    tx: Sender<TaskEntity>,
    db: PgPool,
    sent_tasks: Vec<i64>,
}

impl TaskScheduler {
    pub fn new(tx: Sender<TaskEntity>, db: PgPool) -> Self {
        TaskScheduler {
            tx,
            db,
            sent_tasks: Vec::new(),
        }
    }

    pub async fn scheduler(mut self) {
        tracing::info!("[INIT] running tasks scheduler");

        loop {
            let pending_tasks = repositories::tasks_repository::fetch_pending_tasks(&self.db).await;

            if let Ok(pending) = pending_tasks {
                for task in pending {
                    if !self.sent_tasks.contains(&task.id) {
                        self.sent_tasks.push(task.id);

                        if let Err(e) = self.tx.send(task).await {
                            tracing::error!("[ERR] scheduler failed to send task: {}", e);
                        }
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}

pub struct TaskWorker {
    rx: Receiver<TaskEntity>,
    semaphore: Arc<Semaphore>,
}

impl TaskWorker {
    pub fn new(rx: Receiver<TaskEntity>, max_workers: usize) -> Self {
        TaskWorker {
            rx,
            semaphore: Arc::new(Semaphore::new(max_workers)),
        }
    }

    pub async fn worker(mut self) {
        tracing::info!("[INIT] launching workers");

        while let Some(task) = self.rx.recv().await {
            let permit = self.semaphore.clone().acquire_owned().await.unwrap();
            let task_id = task.id;
            tokio::spawn(async move {
                tracing::info!("[WORKER] processing task: {:#?}", task_id);
                // Simulate some work with the task
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                tracing::info!("[WORKER] completed task: {:#?}", task_id);

                // Permit is dropped here, allowing another task to start
                drop(permit);
            });
        }
    }
}
