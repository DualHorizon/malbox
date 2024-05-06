use repositories::tasks_repository::TaskEntity;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, Semaphore};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::config;

mod config;
mod http;
mod repositories;
mod scheduler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = config().await;

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(config.db_url())
        .await
        .unwrap(); // TODO: handle error properly

    sqlx::migrate!().run(&db).await.unwrap();

    let (tx, mut rx) = mpsc::channel::<TaskEntity>(8);

    let db_clone = db.clone();

    let mut sent_tasks: Vec<i64> = Vec::new();

    tokio::spawn(async move {
        tracing::info!("[INIT] launching workers");

        while let Some(task) = rx.recv().await {
            tracing::info!("[WORKER] received task: {:#?}", task.id);
        }
    });

    tokio::spawn(async move {
        tracing::info!("[INIT] running tasks scheduler");
        loop {
            let pending_tasks =
                repositories::tasks_repository::fetch_pending_tasks(&db_clone).await; // sends values to receiver

            if let Ok(pending) = pending_tasks {
                for task in pending {
                    if !sent_tasks.contains(&task.id) {
                        sent_tasks.push(task.id);

                        if let Err(e) = tx.send(task).await {
                            tracing::error!("Failed to send task: {}", e);
                        }
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

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
