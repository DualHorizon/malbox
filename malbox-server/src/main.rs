use anyhow::Context;
use repositories::tasks_repository::TaskEntity;
use scheduler::{TaskScheduler, TaskWorker};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, Semaphore};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::config;
use malbox_config::load_config;
mod config;
mod http;
mod repositories;
mod scheduler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    load_config();

    let config = config().await;

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(config.db_url())
        .await
        .unwrap(); // TODO: handle error properly

    sqlx::migrate!().run(&db).await.unwrap();

    let db_clone = db.clone();

    let (tx, rx) = mpsc::channel::<TaskEntity>(8);

    let scheduler = TaskScheduler::new(tx, db_clone);
    let worker = TaskWorker::new(rx);

    tokio::spawn(scheduler.scheduler());
    tokio::spawn(worker.worker());
    http::serve(config.clone(), db).await?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "malbox_server=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
