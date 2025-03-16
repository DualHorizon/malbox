use malbox_config::Config;
use malbox_database::PgPool;
use scheduler::Scheduler;
use tracing::{error, info};

mod error;
mod resource;
mod scheduler;
mod task;
mod worker;

pub async fn init_scheduler(config: Config, db: PgPool, max_workers: usize) {
    match Scheduler::new(config.clone(), db.clone()).await {
        Ok(mut scheduler) => {
            if let Err(e) = scheduler.start().await {
                error!("Failed to start scheduler: {}", e);
                return;
            }
            info!("Scheduler initialized with {} max workers", max_workers);
        }
        Err(e) => {
            error!("Failed to create scheduler: {}", e);
        }
    }
}
