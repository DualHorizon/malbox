use malbox_config::load_config;
use malbox_database::{init_database, init_machines};
use malbox_machinery::libvirt_connect;
use malbox_scheduler::init_scheduler;
use malbox_tracing::init_tracing;

mod http;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config().await;

    init_tracing(&config.malbox.debug.rust_log);

    libvirt_connect().await;

    let db = init_database(&config.malbox.postgres).await;

    init_machines(&db, &config.machinery).await;

    init_scheduler(db.clone(), 1).await;

    http::serve(config.malbox.clone(), db).await?;

    Ok(())
}
