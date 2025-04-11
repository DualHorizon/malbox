use malbox_config::Config;
use malbox_database::PgPool;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};

mod error;
mod resource;
mod task;

pub use resource::ResourceManager;
use task::coordinator::TaskCoordinator;
pub use task::notification::TaskNotificationService;

pub async fn init_scheduler(
    config: Config,
    db: PgPool,
    resource_manager: Arc<ResourceManager>,
    task_notifications: mpsc::Receiver<i32>,
) {
    let mut coordinator = TaskCoordinator::new(db, resource_manager, task_notifications, 2);
    coordinator.init().await.unwrap();
}
