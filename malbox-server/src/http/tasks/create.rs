use crate::http::AppState;
use crate::http::Result;
use axum::{
    body::Bytes,
    extract::Multipart,
    routing::{get, post},
    Json, Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use malbox_shared::hashing::md5;
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/tasks/create/file", post(tasks_create_file))
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TaskBody<T> {
    task: T,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct NewTask {
    task_id: u32,
}

#[derive(TryFromMultipart)]
struct CreateTaskRequest {
    #[form_data(limit = "unlimited")]
    file: FieldData<NamedTempFile>,
    package: Option<String>,
    timeout: Option<String>,
    priority: Option<String>,
    options: Option<String>,
    machine: Option<String>,
    platform: Option<String>,
    tags: Option<String>,
    custom: Option<String>,
    owner: Option<String>,
    memory: Option<bool>,
    unique: Option<bool>,
    enforce_timeout: Option<bool>,
}

async fn tasks_create_file(
    TypedMultipart(multipart): TypedMultipart<CreateTaskRequest>,
) -> Result<Json<TaskBody<NewTask>>> {
    let file_name = multipart
        .file
        .metadata
        .file_name
        .unwrap_or(String::from("data.bin"));

    multipart
        .file
        .contents
        .persist(std::env::temp_dir().join(file_name));

    // let created_sample = sqlx::query_scalar!(
    //     r#"INSERT into "samples" () "#,
    // );

    Ok(Json(TaskBody {
        task: NewTask { task_id: 123 },
    }))
}
