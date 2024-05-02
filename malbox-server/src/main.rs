use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::config;

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
