use sqlx::PgPool;
use std::sync::{mpsc::Receiver, Arc};
use tokio::sync::{mpsc, Semaphore};

#[derive(Debug)]
pub enum ActorMessage {
    RunTask { task_id: u32 },
}

impl ActorMessage {
    pub fn display(&self) -> String {
        match self {
            ActorMessage::RunTask { task_id } => {
                format!("ActorMessage::RunTask: {{ task_id: {} }}", task_id)
            }
        }
    }
}

pub struct MyActor {
    receiver: mpsc::Receiver<ActorMessage>,
    semaphore: Arc<Semaphore>,
    db_pool: PgPool,
}

impl MyActor {
    pub fn new(
        receiver: mpsc::Receiver<ActorMessage>,
        semaphore: Arc<Semaphore>,
        db_pool: PgPool,
    ) -> Self {
        Self {
            receiver,
            semaphore,
            db_pool,
        }
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            let permit = self.semaphore.acquire().await.unwrap();
            self.handle_message(msg).await;
            drop(permit);
        }
    }

    async fn handle_message(&mut self, msg: ActorMessage) {
        match msg {
            ActorMessage::RunTask { task_id } => {
                tracing::info!("Processing task with task_id: {task_id}");

                match process_task(&self.db_pool, task_id).await {
                    Ok(_) => tracing::info!("Task processed succesfully."),
                    Err(err) => tracing::info!("Tasl execution error: {}", err),
                }
            }
        }
    }
}

async fn process_task(db_pool: &PgPool, task_id: u32) -> Result<(), anyhow::Error> {
    Ok(())
}
