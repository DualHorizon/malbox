use malbox_database::{repositories::machinery::MachinePlatform, PgPool};
use std::sync::Arc;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Semaphore,
};

use malbox_database::repositories::{
    machinery::{fetch_machine, MachineFilter},
    tasks::{fetch_pending_tasks, TaskEntity},
};

use malbox_machinery::machinery::kvm::{shutdown_machine, start_machine};

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
        tracing::info!("[STARTUP] running tasks scheduler");

        loop {
            let pending_tasks = fetch_pending_tasks(&self.db).await;

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
    db: PgPool,
    semaphore: Arc<Semaphore>,
}

impl TaskWorker {
    pub fn new(rx: Receiver<TaskEntity>, db: PgPool, max_workers: usize) -> Self {
        TaskWorker {
            rx,
            db,
            semaphore: Arc::new(Semaphore::new(max_workers)),
        }
    }

    pub async fn worker(mut self) {
        tracing::info!("[STARTUP] launching workers");

        while let Some(task) = self.rx.recv().await {
            let permit = self.semaphore.clone().acquire_owned().await.unwrap();

            let db = self.db.clone();

            tokio::spawn(async move {
                tracing::info!("[WORKER] processing task: {:#?}", task.id);

                tracing::info!("[WORKER] Checking for free machine...");

                let free_machine = fetch_machine(
                    &db,
                    Some(MachineFilter {
                        locked: Some(false),
                        platform: Some(MachinePlatform::Linux),
                        ..MachineFilter::default()
                    }),
                )
                .await
                .unwrap();

                if free_machine.is_none() {
                    tracing::info!("[WORKER] No free machine found, waiting...");
                }

                if let Some(free_machine) = free_machine {
                    if free_machine.snapshot.is_none() {
                        start_machine(&free_machine.name, None).await.unwrap();
                    } else {
                        start_machine(&free_machine.name, free_machine.snapshot)
                            .await
                            .unwrap();
                    }

                    shutdown_machine(&free_machine.name).await.unwrap();
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                tracing::info!("[WORKER] completed task: {:#?}", task.id);

                drop(permit);
            });
        }
    }
}
