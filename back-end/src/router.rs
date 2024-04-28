use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;

// NOTE: Maybe it's worth changing to ServiceBuilder and merging the routes later

use crate::AppState;

pub fn app_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .fallback(handler_404)
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
