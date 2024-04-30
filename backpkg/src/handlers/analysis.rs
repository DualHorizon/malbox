use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod submission;

pub use submission::submit_file;

// NOTE: Placeholder values
#[derive(Debug, Deserialize)]
pub struct CreateReportRequest {
    title: String,
    body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportResponse {
    id: Uuid,
    title: String,
    body: String,
    published: bool,
}
