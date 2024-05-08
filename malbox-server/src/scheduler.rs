use sqlx::PgPool;
use tokio::sync::mpsc::{self, Receiver, Sender};

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
}

impl TaskWorker {
    pub fn new(rx: Receiver<TaskEntity>) -> Self {
        TaskWorker { rx }
    }

    pub async fn worker(mut self) {
        tracing::info!("[INIT] launching workers");

        while let Some(task) = self.rx.recv().await {
            tracing::info!("[WORKER] received task: {:#?}", task.id);
        }
    }
}
