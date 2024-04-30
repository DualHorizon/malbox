use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::config;
use crate::errors::internal_error;
use crate::router::app_router;

mod config;
mod domain;
mod errors;
mod handlers;
mod infra;
mod router;
mod utils;

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
}

#[tokio::main]
async fn main() {
    init_tracing();

    let config = config().await;

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(config.db_url())
        .await
        .unwrap(); // TODO: handle error properly

    sqlx::migrate!().run(&db).await.unwrap();

    let state = AppState { pool: db };
    let app = app_router(state);

    let host = config.server_host();
    let port = config.server_port();

    let address = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();

    tracing::info!("listening on http://{}", address);

    axum::serve(listener, app)
        .await
        .map_err(internal_error)
        .unwrap();
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "back_end=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
