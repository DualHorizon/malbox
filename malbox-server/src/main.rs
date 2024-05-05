use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::config;

mod actor;
mod config;
mod http;
mod repositories;

use actor::{ActorMessage, MyActor};

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

    tracing::info!("Launching scheduler");

    let (tx, mut rx) = mpsc::channel::<ActorMessage>(100);

    let max_concurrency = 4;
    let permits = Arc::new(Semaphore::new(max_concurrency));

    let mut actor = MyActor::new(rx, permits.clone(), db.clone());

    tokio::spawn(async move {
        actor.run().await;
    });

    tokio::spawn({
        let db_pool = db.clone();
        async move {
            loop {
                tracing::info!("fetching tasks");

                tokio::time::sleep(std::time::Duration::from_secs(4)).await;
            }
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
