use malbox_config::Config;
use malbox_database::PgPool;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};

mod error;
mod resource;
mod scheduler;
mod task;
mod worker;

pub async fn init_scheduler() {
    todo!()
}
