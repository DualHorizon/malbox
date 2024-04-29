use axum::extract::State;
use axum::Json;
use uuid::{uuid, Uuid};

use crate::domain::models::report::ReportError;
use crate::handlers::reports::{CreateReportRequest, ReportResponse};
use crate::infra::repositories::report_repository;

use crate::utils::custom_extractors::json_extractor::JsonExtractor;
use crate::AppState;

pub async fn submit_file(
    State(state): State<AppState>,
    JsonExtractor(new_report): JsonExtractor<CreateReportRequest>,
) -> Result<Json<ReportResponse>, ReportError> {

}
