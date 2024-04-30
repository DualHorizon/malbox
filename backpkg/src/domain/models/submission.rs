use crate::infra::errors::InfraError;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use uuid::Uuid;

#[derive(Debug)]
pub enum SubmissionError {
    InternalServerError,
    NotFound(Uuid),
    InfraError(InfraError),
}

impl IntoResponse for ReportError {
    fn into_response(self) -> axum::response::Response {
        let (status, err_msg) = match self {
            Self::InfraError(db_error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal server error: {}", db_error),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("Internal server error"),
            ),
        };
        (
            status,
            Json(
                json!({"resource":"SubmissionModel", "message": err_msg, "happened_at": chrono::Utc::now() }),
            ),
        )
            .into_response()
    }
}
