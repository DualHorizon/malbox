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
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/tasks/create/file", post(tasks_create_file))
}

#[derive(TryFromMultipart)]
struct CreateTaskRequest {
    #[form_data(limit = "unlimited")]
    file: FieldData<Bytes>,
    package: Option<String>,
}

async fn tasks_create_file(multipart: TypedMultipart<CreateTaskRequest>) {
    let file_name = multipart
        .file
        .metadata
        .file_name
        .as_ref()
        .map(|name| name.as_str())
        .unwrap_or("data.bin");

    let mut file = File::create(file_name).await;
    file.write_all(multipart.file.contents).await
}
