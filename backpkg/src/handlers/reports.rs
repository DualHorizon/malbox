use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use create_report::create_report;

mod create_report;

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
