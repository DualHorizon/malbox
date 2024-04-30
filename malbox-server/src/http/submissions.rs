use crate::http::error::Error;
use crate::http::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/v1/submit-file", post(submit_file))
}

async fn submit_file() {}
