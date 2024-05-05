use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, Semaphore};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::config;

mod actor;
mod config;
mod http;
mod repositories;

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

    let db_arc = Arc::new(db.clone());
    tracing::info!("Launching scheduler");

    let (tx, rx) = mpsc::channel::<String>(100);
    let permits = Arc::new(Semaphore::new(4));

    // no need to limit threads, tokio seems to handle it fine with balancing
    for _ in 0..4 {
        let sem = permits.clone();

        tokio::spawn(async move {
            while let Some(task_id) = rx.recv().await {
                let permit = sem.acquire().await.unwrap();
                tracing::info!("thread spawned");
            }
        });
    }

    let db_clone = db_arc.clone();
    tokio::spawn(async move {
        loop {
            tracing::info!("checking db");
            fetch_pending_tasks(&db_clone, &tx);
            tokio::time::sleep(Duration::from_secs(10)).await;
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

async fn fetch_pending_tasks(pool: &Arc<PgPool>, tx: &mpsc::Sender<String>) {
    tx.send(String::from("task"));
}
