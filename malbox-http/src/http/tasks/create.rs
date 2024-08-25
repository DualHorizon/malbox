use anyhow::Context;
use axum::body::Bytes;
use axum::{
    extract::{DefaultBodyLimit, State},
    routing::post,
    Json, Router,
};
use axum_macros::debug_handler;
use axum_typed_multipart::{FieldData, TryFromField, TryFromMultipart, TypedMultipart};
use magic::cookie::DatabasePaths;
use malbox_hashing::*;
use tempfile::Builder;
use time::{OffsetDateTime, PrimitiveDateTime};

use crate::http::{error::Error, AppState, Result};
use malbox_database::repositories::{
    samples::{insert_sample, Sample, SampleEntity},
    tasks::{insert_task, StatusType, Task, TaskEntity},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/tasks/create/file", post(create_task_from_file))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 10000000))
}

#[derive(serde::Serialize)]
struct TaskResponse {
    task_id: i64,
}

#[derive(Debug)]
struct FileInfo {
    name: String,
    size: i64,
    file_type: String,
    md5: String,
    sha1: String,
    sha256: String,
    sha512: String,
    crc32: String,
    ssdeep: String,
}

#[derive(TryFromMultipart)]
struct CreateTaskRequest {
    #[form_data(limit = "unlimited")]
    file: FieldData<Bytes>,
    package: Option<String>,
    module: Option<String>,
    timeout: Option<i64>,
    priority: Option<i64>,
    options: Option<String>,
    machine: Option<String>, // needs to be checked via typed struct or conditions instead of String
    platform: Option<String>,
    tags: Option<String>,
    custom: Option<String>,
    owner: Option<String>,
    memory: Option<bool>,
    unique: Option<bool>,
    enforce_timeout: Option<bool>,
}

#[debug_handler]
async fn create_task_from_file(
    State(state): State<AppState>,
    TypedMultipart(request): TypedMultipart<CreateTaskRequest>,
) -> Result<Json<TaskResponse>> {
    write_file(&request.file).context("Failed to read file content")?;

    let file_info = get_file_info(&request.file).context("Failed to get file information")?;

    let sample = create_sample(&state, &file_info)
        .await
        .context("Failed to create sample")?;
    let task = create_task(&state, &request, &file_info, sample.id)
        .await
        .context("Failed to create task")?;

    Ok(Json(TaskResponse { task_id: task.id }))
}

// NOTE: This is temporary, file storage should be handled by the malbox_storage
// crate (new plugin system needed in order to do the crate implementation)
fn write_file(file: &FieldData<Bytes>) -> anyhow::Result<()> {
    let file_name = file
        .metadata
        .file_name
        .clone()
        .unwrap_or_else(|| "data.bin".to_string());

    Builder::new().prefix(&file_name).keep(true).tempfile()?;

    Ok(())
}

fn get_file_info(file: &FieldData<Bytes>) -> anyhow::Result<FileInfo> {
    let file_type = {
        let cookie = magic::Cookie::open(magic::cookie::Flags::default())
            .context("Failed to open magic cookie")?;
        let cookie = cookie.load(&DatabasePaths::default()).unwrap();
        cookie
            .buffer(&file.contents)
            .context("Failed to analyze file type")?
    };

    Ok(FileInfo {
        name: file
            .metadata
            .file_name
            .as_deref()
            .unwrap_or("data.bin")
            .to_string(),
        size: file.contents.len() as i64,
        file_type,
        md5: get_md5(&mut file.contents.to_vec()),
        sha1: get_sha1(&mut file.contents.to_vec()),
        sha256: get_sha256(&mut file.contents.to_vec()),
        sha512: get_sha512(&mut file.contents.to_vec()),
        crc32: get_crc32(&mut file.contents.to_vec()),
        ssdeep: get_ssdeep(&mut file.contents.to_vec()),
    })
}

async fn create_sample(state: &AppState, file_info: &FileInfo) -> Result<SampleEntity> {
    let sample = Sample {
        file_size: file_info.size,
        file_type: file_info.file_type.clone(),
        md5: file_info.md5.clone(),
        crc32: file_info.crc32.clone(),
        sha1: file_info.sha1.clone(),
        sha256: file_info.sha256.clone(),
        sha512: file_info.sha512.clone(),
        ssdeep: file_info.ssdeep.clone(),
    };

    insert_sample(&state.pool, sample)
        .await
        .map_err(Error::from)
}

async fn create_task(
    state: &AppState,
    request: &CreateTaskRequest,
    file_info: &FileInfo,
    sample_id: i64,
) -> Result<TaskEntity> {
    let utc_now = OffsetDateTime::now_utc();
    let current_primitive_datetime = PrimitiveDateTime::new(utc_now.date(), utc_now.time());

    let task = Task {
        target: file_info.name.to_string(),
        module: request
            .module
            .as_deref()
            .unwrap_or("file_analysis")
            .to_string(),
        timeout: request.timeout.unwrap_or(1),
        priority: request.priority.unwrap_or(1),
        custom: request.custom.clone(),
        machine: request.machine.clone(),
        package: request.package.clone(),
        options: request.options.clone(),
        platform: request.platform.clone(),
        unique: request.unique,
        tags: request.tags.clone(),
        owner: request.owner.clone(),
        memory: request.memory.unwrap_or(false),
        enforce_timeout: request.enforce_timeout.unwrap_or(false),
        added_on: current_primitive_datetime,
        started_on: None,
        completed_on: None,
        status: StatusType::Pending,
        sample_id,
    };

    insert_task(&state.pool, task).await.map_err(Error::from)
}
