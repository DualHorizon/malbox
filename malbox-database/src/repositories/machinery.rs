use crate::error::{MachineError, Result};
use bon::Builder;
use malbox_config::machinery::MachineArch as MachineArchConfig;
use malbox_config::types::Platform as MachinePlatformConfig;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, FromRow, PgPool, Postgres, QueryBuilder};
use time::PrimitiveDateTime;

#[derive(sqlx::Type, Debug, Serialize, Deserialize, Default)]
#[sqlx(type_name = "machine_arch", rename_all = "lowercase")]
pub enum MachineArch {
    X86,
    #[default]
    X64,
}

#[derive(sqlx::Type, Debug, Serialize, Deserialize, Default, Clone)]
#[sqlx(type_name = "machine_platform", rename_all = "lowercase")]
pub enum MachinePlatform {
    #[default]
    Windows,
    Linux,
    MacOs,
}

impl From<MachineArchConfig> for MachineArch {
    fn from(value: MachineArchConfig) -> Self {
        match value {
            MachineArchConfig::X64 => MachineArch::X64,
            MachineArchConfig::X86 => MachineArch::X86,
        }
    }
}

impl From<MachinePlatformConfig> for MachinePlatform {
    fn from(value: MachinePlatformConfig) -> Self {
        match value {
            MachinePlatformConfig::Linux => MachinePlatform::Linux,
            MachinePlatformConfig::Windows => MachinePlatform::Windows,
        }
    }
}

#[derive(Builder, Default)]
pub struct Machine {
    pub name: String,
    pub label: String,
    #[builder(default = MachineArch::X64)]
    pub arch: MachineArch,
    #[builder(default = MachinePlatform::Windows)]
    pub platform: MachinePlatform,
    pub ip: String,
    pub tags: Option<Vec<String>>,
    pub interface: Option<String>,
    pub snapshot: Option<String>,
    #[builder(default = false)]
    pub locked: bool,
    pub locked_changed_on: Option<PrimitiveDateTime>,
    pub status: Option<String>,
    pub status_changed_on: Option<PrimitiveDateTime>,
    pub result_server_ip: Option<String>,
    pub result_server_port: Option<String>,
    #[builder(default = false)]
    pub reserved: bool,
}

#[derive(FromRow, Debug)]
pub struct MachineEntity {
    pub id: i32,
    pub name: String,
    pub label: String,
    pub arch: MachineArch,
    pub platform: MachinePlatform,
    pub ip: String,
    pub tags: Option<Vec<String>>,
    pub interface: Option<String>,
    pub snapshot: Option<String>,
    pub locked: bool,
    pub locked_changed_on: Option<PrimitiveDateTime>,
    pub status: Option<String>,
    pub status_changed_on: Option<PrimitiveDateTime>,
    pub result_server_ip: Option<String>,
    pub result_server_port: Option<String>,
    pub reserved: bool,
}

#[derive(Builder, Default)]
pub struct MachineFilter {
    pub locked: Option<bool>,
    pub label: Option<String>,
    pub platform: Option<MachinePlatform>,
    pub tags: Option<String>,
    pub arch: Option<MachineArch>,
    #[builder(default = false)]
    pub include_reserved: bool,
    pub os_version: Option<String>,
}

pub async fn insert_machine(pool: &PgPool, machine: Machine) -> Result<MachineEntity> {
    query_as!(
        MachineEntity,
        r#"
        INSERT into "machines" (name, label, arch, platform, ip, tags, interface, snapshot, locked, locked_changed_on, status, status_changed_on, result_server_ip, result_server_port, reserved)
        values ($1::varchar, $2::varchar, $3::machine_arch, $4::machine_platform, $5::varchar, $6::varchar[], $7::varchar,
            $8::varchar, $9::boolean, $10::timestamp, $11::varchar, $12::timestamp, $13::varchar, $14::varchar, $15::bool)
        returning id, name, label, arch AS "arch!: MachineArch", platform AS "platform!: MachinePlatform", ip, tags, interface, snapshot, locked, locked_changed_on, status, status_changed_on, result_server_ip, result_server_port, reserved
        "#,
        machine.name,
        machine.label,
        machine.arch as MachineArch,
        machine.platform as MachinePlatform,
        machine.ip,
        machine.tags.as_deref(),
        machine.interface,
        machine.snapshot,
        machine.locked,
        machine.locked_changed_on,
        machine.status,
        machine.status_changed_on,
        machine.result_server_ip,
        machine.result_server_port,
        machine.reserved
    )
    .fetch_one(pool)
    .await
    .map_err(|e| MachineError::InsertFailed {
        name: machine.name,
        message: "failed to insert machine record".to_string(),
        source: e
    }.into())
}

pub async fn clean_machines(pool: &PgPool) -> Result<()> {
    query!(
        r#"
        TRUNCATE "machines";
        "#
    )
    .execute(pool)
    .await
    .map_err(|e| MachineError::TruncateFailed { source: e });

    query!(
        r#"
        DELETE FROM "machines";
        "#
    )
    .execute(pool)
    .await
    .map_err(|e| MachineError::DeleteFailed { source: e });

    Ok(())
}

pub async fn fetch_machines(
    pool: &PgPool,
    filter: Option<MachineFilter>,
) -> Result<Vec<MachineEntity>> {
    // the query will be adjusted depending on other params to filter out specific machines

    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
            SELECT id, name, label, arch, platform AS "platform!: MachinePlatform, ip, tags, interface, snapshot, locked, locked_changed_on, status, status_changed_on,
            result_server_ip, result_server_port, reserved
            FROM "machines"
        "#,
    );

    if let Some(filter) = filter {
        if let Some(locked) = filter.locked {
            query_builder.push(" AND locked = ");
            query_builder.push_bind(locked);
        }
        if let Some(label) = filter.label {
            query_builder.push(" AND label = ");
            query_builder.push_bind(label);
        }
        if let Some(platform) = filter.platform {
            query_builder.push(" AND platform = ");
            query_builder.push_bind(platform);
        }
        if let Some(tags) = filter.tags {
            query_builder.push(" AND tags @> ");
            query_builder.push_bind(tags);
        }
        if let Some(arch) = filter.arch {
            query_builder.push(" AND arch = ");
            query_builder.push_bind(arch);
        }
        if let Some(os_version) = filter.os_version {
            query_builder.push(" AND os_version = ");
            query_builder.push_bind(os_version);
        }
        if !filter.include_reserved {
            query_builder.push(" AND reserved = false");
        }
    }

    let query = query_builder
        .build_query_as::<MachineEntity>()
        .fetch_all(pool)
        .await
        .map_err(|e| MachineError::FetchFailed { source: e })?;

    Ok(query)
}

pub async fn fetch_machine(
    pool: &PgPool,
    filter: Option<MachineFilter>,
) -> Result<Option<MachineEntity>> {
    // the query will be adjusted depending on other params to filter out specific machines

    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
            SELECT id, name, label, arch, platform, ip, tags, interface, snapshot, locked, locked_changed_on, status, status_changed_on,
            result_server_ip, result_server_port, reserved
            FROM "machines" WHERE 1 = 1
        "#,
    );

    if let Some(filter) = filter {
        if let Some(locked) = filter.locked {
            query_builder.push(" AND locked = ");
            query_builder.push_bind(locked);
        }
        if let Some(label) = filter.label {
            query_builder.push(" AND label = ");
            query_builder.push_bind(label);
        }
        if let Some(platform) = filter.platform {
            query_builder.push(" AND platform = ");
            query_builder.push_bind(platform);
        }
        if let Some(tags) = filter.tags {
            query_builder.push(" AND tags @> ");
            query_builder.push_bind(tags);
        }
        if let Some(arch) = filter.arch {
            query_builder.push(" AND arch = ");
            query_builder.push_bind(arch);
        }
        if let Some(os_version) = filter.os_version {
            query_builder.push(" AND os_version = ");
            query_builder.push_bind(os_version);
        }
        if !filter.include_reserved {
            query_builder.push(" AND reserved = false");
        }
    }

    let query = query_builder
        .build_query_as::<MachineEntity>()
        .fetch_optional(pool)
        .await
        .map_err(|e| MachineError::FetchFailed { source: e })?;
    //.context("failed to fetch machines");

    Ok(query)
}

pub async fn update_machine(pool: &PgPool, id: i32, machine: Machine) -> Result<MachineEntity> {
    query_as!(
        MachineEntity,
        r#"
        UPDATE "machines"
        SET
            name = $1::varchar,
            label = $2::varchar,
            arch = $3::machine_arch,
            platform = $4::machine_platform,
            ip = $5::varchar,
            tags = $6::varchar[],
            interface = $7::varchar,
            snapshot = $8::varchar,
            locked = $9::boolean,
            locked_changed_on = $10::timestamp,
            status = $11::varchar,
            status_changed_on = $12::timestamp,
            result_server_ip = $13::varchar,
            result_server_port = $14::varchar,
            reserved = $15::bool
        WHERE id = $16
        RETURNING id, name, label, arch AS "arch!: MachineArch", platform AS "platform!: MachinePlatform", ip, tags, interface, snapshot, locked, locked_changed_on, status, status_changed_on, result_server_ip, result_server_port, reserved
        "#,
        machine.name,
        machine.label,
        machine.arch as MachineArch,
        machine.platform as MachinePlatform,
        machine.ip,
        machine.tags.as_deref(),
        machine.interface,
        machine.snapshot,
        machine.locked,
        machine.locked_changed_on,
        machine.status,
        machine.status_changed_on,
        machine.result_server_ip,
        machine.result_server_port,
        machine.reserved,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| MachineError::UpdateFailed { message: "Failed to update machine".to_string(), source: e }.into())
}

pub async fn update_machine_status(
    pool: &PgPool,
    id: i32,
    locked: bool,
    status: Option<&str>,
) -> Result<MachineEntity> {
    let now = time::OffsetDateTime::now_utc();
    let primitive_now = time::PrimitiveDateTime::new(now.date(), now.time());

    query_as!(
        MachineEntity,
        r#"
        UPDATE "machines"
        SET
            locked = $1,
            locked_changed_on = $2,
            status = $3,
            status_changed_on = $2
        WHERE id = $4
        RETURNING id, name, label, arch AS "arch!: MachineArch", platform AS "platform!: MachinePlatform", ip, tags, interface, snapshot, locked, locked_changed_on, status, status_changed_on, result_server_ip, result_server_port, reserved
        "#,
        locked,
        primitive_now,
        status,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| MachineError::UpdateFailed { message: "Failed to update status".to_string(), source: e }.into())
}

pub async fn lock_machine(pool: &PgPool, id: i32, status: Option<&str>) -> Result<MachineEntity> {
    update_machine_status(pool, id, true, status).await
}

pub async fn unlock_machine(pool: &PgPool, id: i32) -> Result<MachineEntity> {
    update_machine_status(pool, id, false, None).await
}

pub async fn assign_snapshot(pool: &PgPool, id: i32, snapshot: &str) -> Result<MachineEntity> {
    query_as!(
        MachineEntity,
        r#"
        UPDATE "machines"
        SET snapshot = $1
        WHERE id = $2
        RETURNING id, name, label, arch AS "arch!: MachineArch", platform AS "platform!: MachinePlatform", ip, tags, interface, snapshot, locked, locked_changed_on, status, status_changed_on, result_server_ip, result_server_port, reserved
        "#,
        snapshot,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| MachineError::UpdateFailed { message: "Failed to assign snapshot".to_string(), source: e }.into())
}

pub async fn update_machine_tags(
    pool: &PgPool,
    id: i32,
    tags: Vec<String>,
) -> Result<MachineEntity> {
    query_as!(
        MachineEntity,
        r#"
        UPDATE "machines"
        SET tags = $1
        WHERE id = $2
        RETURNING id, name, label, arch AS "arch!: MachineArch", platform AS "platform!: MachinePlatform", ip, tags, interface, snapshot, locked, locked_changed_on, status, status_changed_on, result_server_ip, result_server_port, reserved
        "#,
        &tags as &[String],
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| MachineError::UpdateFailed { message: "Failed to update machine tags".to_string(), source: e }.into())
}

pub async fn update_machine_network(
    pool: &PgPool,
    id: i32,
    ip: &str,
    interface: Option<&str>,
) -> Result<MachineEntity> {
    query_as!(
        MachineEntity,
        r#"
        UPDATE "machines"
        SET
            ip = $1,
            interface = $2
        WHERE id = $3
        RETURNING id, name, label, arch AS "arch!: MachineArch", platform AS "platform!: MachinePlatform", ip, tags, interface, snapshot, locked, locked_changed_on, status, status_changed_on, result_server_ip, result_server_port, reserved
        "#,
        ip,
        interface,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| MachineError::UpdateFailed { message: "Failed to update machine network".to_string(), source: e }.into())
    // .context("failed to update machine network configuration")
}

pub async fn update_result_server(
    pool: &PgPool,
    id: i32,
    server_ip: &str,
    server_port: &str,
) -> Result<MachineEntity> {
    query_as!(
        MachineEntity,
        r#"
        UPDATE "machines"
        SET
            result_server_ip = $1,
            result_server_port = $2
        WHERE id = $3
        RETURNING id, name, label, arch AS "arch!: MachineArch", platform AS "platform!: MachinePlatform", ip, tags, interface, snapshot, locked, locked_changed_on, status, status_changed_on, result_server_ip, result_server_port, reserved
        "#,
        server_ip,
        server_port,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| MachineError::UpdateFailed { message: "Failed to update machine result server configuration".to_string(), source: e }.into())
    // .context("failed to update machine result server configuration")
}
