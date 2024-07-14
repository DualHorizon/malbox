use anyhow::Context;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow, PgPool};
use time::{macros::date, PrimitiveDateTime};

#[derive(sqlx::Type, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "status_type", rename_all = "lowercase")]
pub enum StatusType {
    Pending,
    Processing,
    Failure,
    Success,
}

pub struct Task {
    pub target: String,
    pub module: String,
    pub tags: Option<String>,
    pub owner: Option<String>,
    pub timeout: i64,
    pub priority: i64,
    pub custom: Option<String>,
    pub machine: Option<String>,
    pub package: Option<String>,
    pub options: Option<String>,
    pub platform: Option<String>,
    pub memory: bool,
    pub unique: Option<bool>,
    pub enforce_timeout: bool,
    pub added_on: PrimitiveDateTime,
    pub started_on: Option<PrimitiveDateTime>,
    pub completed_on: Option<PrimitiveDateTime>,
    pub status: StatusType,
    pub sample_id: i64,
}

#[derive(FromRow, Debug)]
pub struct TaskEntity {
    pub id: i64,
    pub target: String,
    pub module: String,
    pub timeout: i64,
    pub priority: i64,
    pub custom: Option<String>,
    pub machine: Option<String>,
    pub package: Option<String>,
    pub options: Option<String>,
    pub platform: Option<String>,
    pub memory: bool,
    pub enforce_timeout: bool,
    pub added_on: Option<PrimitiveDateTime>,
    pub started_on: Option<PrimitiveDateTime>,
    pub completed_on: Option<PrimitiveDateTime>,
    pub status: StatusType,
    pub sample_id: Option<i64>,
}

impl Default for TaskEntity {
    fn default() -> Self {
        TaskEntity {
            id: 1,
            target: String::from("default"),
            module: String::from("default"),
            timeout: 0,
            priority: 1,
            custom: Some(String::from("default")),
            machine: Some(String::from("default")),
            package: Some(String::from("default")),
            options: Some(String::from("default")),
            platform: Some(String::from("default")),
            memory: false,
            enforce_timeout: false,
            added_on: Some(PrimitiveDateTime::new(
                date!(2019 - 01 - 01),
                time::macros::time!(0:00),
            )),
            started_on: Some(PrimitiveDateTime::new(
                date!(2019 - 01 - 01),
                time::macros::time!(0:00),
            )),
            completed_on: Some(PrimitiveDateTime::new(
                date!(2019 - 01 - 01),
                time::macros::time!(0:00),
            )),
            status: StatusType::Pending,
            sample_id: Some(1),
        }
    }
}

pub async fn insert_task(pool: &PgPool, task: Task) -> anyhow::Result<TaskEntity> {
    query_as!(
        TaskEntity,
        r#"
        INSERT into "tasks" (target, module, timeout, priority, custom, machine, package, options, platform, memory, enforce_timeout, added_on, started_on, completed_on, status, sample_id)
        values ($1::varchar, $2::varchar, $3::bigint, $4::bigint, $5::varchar, $6::varchar, $7::varchar,
            $8::varchar, $9::varchar, $10::boolean, $11::boolean, $12::timestamp, $13::timestamp, $14::timestamp, $15::status_type, $16::bigint)
        returning id, target, module, timeout, priority, custom, machine, package, options, platform, memory, enforce_timeout, added_on, started_on, completed_on, status AS "status!: StatusType", sample_id
        "#,
        task.target,
        task.module,
        task.timeout,
        task.priority,
        task.custom,
        task.machine,
        task.package,
        task.options,
        task.platform,
        task.memory,
        task.enforce_timeout,
        task.added_on,
        task.started_on,
        task.completed_on,
        task.status as StatusType,
        task.sample_id
    )
    .fetch_one(pool)
    .await
    .context("failed to insert sample")
}

pub async fn fetch_pending_tasks(pool: &PgPool) -> anyhow::Result<Vec<TaskEntity>> {
    query_as!(
        TaskEntity,
        r#"
        SELECT id, target, module, timeout, priority, custom, machine, package, options, platform, memory, enforce_timeout, added_on, started_on, completed_on, status AS "status!: StatusType", sample_id
        from "tasks" WHERE status = 'pending'
        "#,
    )
    .fetch_all(pool)
    .await
    .context("failed to fetch pending tasks")
}

