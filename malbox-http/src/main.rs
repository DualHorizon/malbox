use malbox_config::load_config;
use malbox_core::load_plugin;
use malbox_database::{init_database, init_machines};
use malbox_scheduler::init_scheduler;
use malbox_tracing::init_tracing;
use std::path::Path;
mod http;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config().await;

    init_tracing(&config.malbox.debug.rust_log);

    let _ = load_plugin(Path::new(
        "./plugins/dummy_plugin/target/debug/libdummy_plugin.so",
    ))
    .unwrap();

    let db = init_database(&config.malbox.postgres).await;

    // init_machines(&db, &config.machinery).await;

    init_scheduler(db.clone(), 1).await;

    http::serve(config.malbox.clone(), db).await?;

    Ok(())
}
