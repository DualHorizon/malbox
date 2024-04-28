use axum::extract::State;
use axum::Json;
use uuid::{uuid, Uuid};

use crate::domain::models::report::ReportError;
use crate::handlers::reports::{CreateReportRequest, ReportResponse};
use crate::infra::repositories::report_repository;

use crate::utils::custom_extractors::json_extractor::JsonExtractor;
use crate::AppState;

pub async fn create_report(
    State(state): State<AppState>,
    JsonExtractor(new_report): JsonExtractor<CreateReportRequest>,
) -> Result<Json<ReportResponse>, ReportError> {
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
        published: false
    };

    Ok(Json(report_response))
}
