use malbox_config::Config;
use malbox_database::{init_database, init_machines};
use malbox_http::http;
use malbox_scheduler::init_scheduler;

mod error;
pub use error::DaemonError;

pub async fn run(config: Config) -> error::Result<()> {
    let db = init_database(&config.database).await;

    init_machines(&db, &config.machinery).await.unwrap();

    init_scheduler(db.clone(), 1).await;

    http::serve(config.clone(), db)
        .await
        .map_err(|e| DaemonError::Internal(e.to_string()))
}
