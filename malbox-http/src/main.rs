use malbox_config::load_config;
use malbox_core::load_module;
use malbox_database::{init_database, init_machines};
use malbox_scheduler::init_scheduler;
use malbox_tracing::init_tracing;
use std::path::Path;

mod http;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config().await;

    init_tracing(&config.malbox.debug.rust_log);

    let db = init_database(&config.malbox.postgres).await;

    init_machines(&db, &config.machinery).await;

    init_scheduler(db.clone(), 1).await;

    let mut module = load_module(
        Path::new("./modules/dummy_module/target/debug/libdummy_module.so"),
        None,
    )
    .await
    .unwrap();

    module.execute_plugins(vec![0]).await.unwrap();

    http::serve(config.malbox.clone(), db).await?;

    Ok(())
}
