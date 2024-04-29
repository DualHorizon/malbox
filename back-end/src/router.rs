use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;

// NOTE: Maybe it's worth changing to ServiceBuilder and merging the routes later

use crate::handlers::analysis::submit_file::submit_file;
use crate::AppState;

pub fn app_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(root))
        .nest("/v1/submit-file", analysis_routes(state.clone()))
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

fn analysis_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/:file", post(submit_file))
}
