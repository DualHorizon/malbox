use axum::extract::DefaultBodyLimit;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use axum_macros::debug_handler;
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use magic::cookie::DatabasePaths;
use malbox_hashing::*;
use std::io::Read;
use tempfile::NamedTempFile;
use time::OffsetDateTime;
use time::PrimitiveDateTime;

use crate::http;
use crate::http::AppState;
use crate::http::Result;

use malbox_database::repositories::samples::{insert_sample, Sample};
use malbox_database::repositories::tasks::{insert_task, StatusType, Task};

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
    task_id: i64,
}

#[derive(TryFromMultipart)]
struct CreateTaskRequest {
    #[form_data(limit = "unlimited")]
    file: FieldData<NamedTempFile>,
    package: Option<String>,
    module: Option<String>,
    timeout: Option<i64>,
    priority: Option<i64>,
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

#[debug_handler]
async fn tasks_create_file(
    State(state): State<AppState>,
    TypedMultipart(multipart): TypedMultipart<CreateTaskRequest>,
) -> Result<Json<TaskBody<NewTask>>, http::error::Error> {
    let file_name = multipart
        .file
        .metadata
        .file_name
        .unwrap_or(String::from("data.bin"));

    let file_path = std::env::temp_dir().join(&file_name);

    let temp_file = multipart.file.contents.persist(&file_path);

    // NOTE: there's probably a better way to do this
    let mut file_handle = temp_file.unwrap();

    let mut file_contents: Vec<u8> = Vec::new();
    file_handle.read_to_end(&mut file_contents).unwrap();

    let _file_size = file_contents.len() as i64;
    // very slow
    let md5_hash = get_md5(&mut file_contents);
    let sha1_hash = get_sha1(&mut file_contents);
    let sha256_hash = get_sha256(&mut file_contents);
    let sha512_hash = get_sha512(&mut file_contents);
    let crc32_hash = get_crc32(&mut file_contents);
    let ssdeep_hash = get_ssdeep(&mut file_contents);

    tracing::debug!("md5: {:#?}", md5_hash);
    tracing::debug!("sha256: {:#?}", sha256_hash);
    tracing::debug!("sha512: {:#?}", sha512_hash);
    tracing::debug!("crc32: {:#?}", crc32_hash);
    tracing::debug!("ssdeep: {:#?}", ssdeep_hash);

    let _file_type = {
        let cookie = magic::Cookie::open(magic::cookie::Flags::default()).unwrap();
        let cookie = cookie.load(&DatabasePaths::default()).unwrap();
        cookie.buffer(&file_contents).unwrap()
    };

    tracing::debug!("file type: {_file_type}");
    tracing::debug!("file name: {file_name}");
    tracing::debug!("file size: {_file_size}");

    let sample_entity = Sample {
        file_size: _file_size,
        file_type: _file_type,
        md5: md5_hash,
        crc32: crc32_hash,
        sha1: sha1_hash,
        sha256: sha256_hash,
        sha512: sha512_hash,
        ssdeep: ssdeep_hash,
    };

    let created_sample = insert_sample(&state.pool, sample_entity)
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert or fetch existing sample: {:#?}", e);
            http::error::Error::from(e)
        })?;

    let now = OffsetDateTime::now_utc();
    let current_primitive_datetime = PrimitiveDateTime::new(now.date(), now.time());

    let task_entity = Task {
        target: file_path.into_os_string().into_string().unwrap(),
        module: multipart.module.unwrap_or("file_analysis".to_string()),
        timeout: multipart.timeout.unwrap_or(1),
        priority: multipart.priority.unwrap_or(1),
        custom: multipart.custom,
        machine: multipart.machine,
        package: multipart.package,
        options: multipart.options,
        platform: multipart.platform,
        unique: multipart.unique,
        tags: multipart.tags,
        owner: multipart.owner,
        memory: multipart.memory.unwrap_or(false),
        enforce_timeout: multipart.enforce_timeout.unwrap_or(false),
        added_on: current_primitive_datetime,
        started_on: None,
        completed_on: None,
        status: StatusType::Pending,
        sample_id: created_sample.id,
    };

    let created_task = insert_task(&state.pool, task_entity).await.map_err(|e| {
        tracing::error!("Failed to insert task: {:#?}", e);
        http::error::Error::from(e)
    })?;

    tracing::debug!("{:#?}", created_task);

    Ok(Json(TaskBody {
        task: NewTask {
            task_id: created_task.id,
        },
    }))
}
