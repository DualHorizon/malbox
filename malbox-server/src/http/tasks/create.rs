use crate::http;
use crate::http::AppState;
use crate::http::Result;
use crate::http::ResultExt;
use axum::extract::DefaultBodyLimit;
use axum::{
    body::Bytes,
    extract::{Multipart, State},
    routing::{get, post},
    Json, Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use magic::cookie::DatabasePaths;
use malbox_shared::hash::*;
use std::io::Read;
use tempfile::NamedTempFile;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/tasks/create/file", post(tasks_create_file))
        .layer(DefaultBodyLimit::max(10 * 1000 * 100000)) // NOTE: this should be modified, temporary
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
    State(state): State<AppState>,
    TypedMultipart(multipart): TypedMultipart<CreateTaskRequest>,
) -> Result<Json<TaskBody<NewTask>>, http::error::Error> {
    tracing::info!("{:#?}", state);
    let file_name = multipart
        .file
        .metadata
        .file_name
        .unwrap_or(String::from("data.bin"));

    let temp_file = multipart
        .file
        .contents
        .persist(std::env::temp_dir().join(&file_name));

    // NOTE: there's probably a better way to do this
    let mut file_handle = temp_file.unwrap();

    let mut file_contents: Vec<u8> = Vec::new();
    file_handle.read_to_end(&mut file_contents).unwrap();

    let file_size = file_contents.len() as i32;
    // very slow
    let md5_hash = get_md5(&mut file_contents);
    let sha1_hash = get_sha1(&mut file_contents);
    let sha256_hash = get_sha256(&mut file_contents);
    let sha512_hash = get_sha512(&mut file_contents);
    let crc32_hash = get_crc32(&mut file_contents);
    let ssdeep_hash = get_ssdeep(&mut file_contents);

    tracing::info!("md5: {:#?}", md5_hash);
    tracing::info!("sha256: {:#?}", sha256_hash);
    tracing::info!("sha512: {:#?}", sha512_hash);
    tracing::info!("crc32: {:#?}", crc32_hash);
    tracing::info!("ssdeep: {:#?}", ssdeep_hash);

    let cookie = magic::Cookie::open(magic::cookie::Flags::default()).unwrap();
    let cookie = cookie.load(&DatabasePaths::default()).unwrap();

    let file_type = cookie.buffer(&file_contents).unwrap();
    tracing::info!("{file_type}");
    tracing::info!("file name: {file_name}");
    tracing::info!("file size: {file_size}");

    let created_sample = sqlx::query_scalar!(
        r#"INSERT into "samples" (file_size, file_type, md5, crc32, sha1, sha256, sha512, ssdeep) values ($1, $2, $3, $4, $5, $6, $7, $8) returning id"#,
        file_size,
        file_type,
        md5_hash,
        crc32_hash,
        sha1_hash,
        sha256_hash,
        sha512_hash,
        ssdeep_hash
    ).fetch_one(&state.pool)
    .await;

    Ok(Json(TaskBody {
        task: NewTask { task_id: 123 },
    }))
}
