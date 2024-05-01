use crate::http::error::Error;
use crate::http::AppState;
use axum::{
    body::Bytes,
    extract::Multipart,
    routing::{get, post},
    Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/tasks/create/file", post(tasks_create_file))
}

#[derive(TryFromMultipart)]
struct CreateTaskRequest {
    file: FieldData<Bytes>,
    package: String,
}

async fn tasks_create_file(multipart: TypedMultipart<CreateTaskRequest>) {
    tracing::info!("Upload name: {:#?}", multipart.file.metadata);
}
