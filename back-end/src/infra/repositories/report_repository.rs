use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::domain::models::report::ReportModel;
use crate::infra::errors::{adapt_infra_error, InfraError};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReportBody<T> { // NOTE: Currenlty using Generic as there may be more strucs concerning reports, for ex. OpenReport/ClosedReport
    report: T,
}

// NOTE: Temporary placeholder fields for the reports
#[derive(Serialize)]
pub struct Report {
    pub title: String,
    pub body: String,
    pub published: bool,
}

pub async fn insert(
    ctx: AppState,
    new_report: Report,
) -> Result<ReportModel, InfraError> {

    // NOTE: Temporary placeholder values for the reports
    let report = sqlx::query_scalar!(
        r#"insert into "report" (title, body, published) values ($1, $2, $3)"#,
        new_report.title,
        new_report.body,
        new_report.published
    )
    .fetch_one(&ctx.pool)
    .await
    .map_err(adapt_infra_error)?;

    Ok(ReportModel {
        title: new_report.title,
        body: new_report.body,
        published: new_report.published
    })
}
