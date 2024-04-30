use axum::extract::{Multipart, State};
use axum::Json;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::{uuid, Uuid};

use crate::domain::models::report::ReportError;
use crate::handlers::analysis::*;
use crate::infra::repositories::report_repository;

use crate::utils::custom_extractors::json_extractor::JsonExtractor;
use crate::AppState;

pub async fn submit_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ReportResponse>, ReportError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .expect("Failed to get next field!")
    {
        if field.name().unwrap() != "fileupload" {
            continue;
        }

        println!("Got file!");

        let file_name = field.file_name().unwrap();

        let file_path = format!("files/{}", file_name);

        let data = field.bytes().await.unwrap();

        let mut file_handle = File::create(file_path)
            .await
            .expect("Failed to open file handle!");

        file_handle.write_all(&data).await.expect("Failed to write");
    }

    let new_report = report_repository::Report {
        title: new_report.title,
        body: new_report.body,
        published: false,
    };

    let created_port = report_repository::insert(state, new_report)
        .await
        .map_err(ReportError::InfraError)?;

    let report_response = ReportResponse {
        id: uuid!("9f5696c3-4604-4996-9205-1400075827c6"),
        title: String::from("asd"),
        body: String::from("asd"),
        published: false,
    };

    Ok(Json(report_response))
}
