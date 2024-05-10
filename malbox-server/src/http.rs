use crate::config::Config;
use anyhow::Context;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

mod error;
mod submissions;
mod tasks;

pub use error::{Error, ResultExt};
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug)]
struct AppState {
    config: Config,
    pool: PgPool,
}

pub async fn serve(conf: Config, db: PgPool) -> anyhow::Result<()> {
    let shared_state = AppState {
        config: conf,
        pool: db,
    };

    let app = api_router()
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state.clone());

    let host = shared_state.config.server_host();
    let port = shared_state.config.server_port();

    let address = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&address)
        .await
        .context("error binding TcpListener")
        .unwrap();

    tracing::info!("[INIT] listening on http://{}", address);

    axum::serve(listener, app)
        .await
        .context("error running HTTP server!")
}

fn api_router() -> Router<AppState> {
    Router::new()
        .route("/", get(root))
        .fallback(handler_404)
        .merge(tasks::create::router())
}

async fn root() -> &'static str {
    "Server is running!"
}

async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "The requested resource was not found",
    )
}
