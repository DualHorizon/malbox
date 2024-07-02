use anyhow::Context;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use malbox_config::malbox::MalboxConfig;
use malbox_database::PgPool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

mod error;
mod tasks;

pub use error::Error;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug)]
struct AppState {
    config: MalboxConfig,
    pool: PgPool,
}

pub async fn serve(conf: MalboxConfig, db: PgPool) -> anyhow::Result<()> {
    let shared_state = AppState {
        config: conf,
        pool: db,
    };

    let app = api_router()
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state.clone());

    let host = shared_state.config.http.host;
    let port = shared_state.config.http.port;

    let address = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&address)
        .await
        .context("error binding TcpListener")
        .unwrap();

    tracing::info!("[STARTUP] listening on http://{}", address);

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
