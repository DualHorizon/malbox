use anyhow::Context;
use malbox_config::machinery::{
    MachineArch as MachineArchConfig, MachinePlatform as MachinePlatformConfig,
};
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

#[derive(sqlx::Type, Debug, Serialize, Deserialize, Default)]
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
            MachinePlatformConfig::MacOs => MachinePlatform::MacOs,
            MachinePlatformConfig::Windows => MachinePlatform::Windows,
        }
    }
}

#[derive(Default)]
pub struct Machine {
    pub name: String,
    pub label: String,
    pub arch: MachineArch,
    pub platform: MachinePlatform,
    pub ip: String,
    pub tags: Option<String>,
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

#[derive(FromRow, Debug)]
pub struct MachineEntity {
    pub id: i32,
    pub name: String,
    pub label: String,
    pub arch: MachineArch,
    pub platform: MachinePlatform,
    pub ip: String,
    pub tags: Option<String>,
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

#[derive(Default)]
pub struct MachineFilter {
    pub locked: Option<bool>,
    pub label: Option<String>,
    pub platform: Option<MachinePlatform>,
    pub tags: Option<String>,
    pub arch: Option<MachineArch>,
    pub include_reserved: bool,
    pub os_version: Option<String>,
}

pub async fn insert_machine(pool: &PgPool, machine: Machine) -> anyhow::Result<MachineEntity> {
    query_as!(
        MachineEntity,
        r#"
        INSERT into "machines" (name, label, arch, platform, ip, tags, interface, snapshot, locked, locked_changed_on, status, status_changed_on, result_server_ip, result_server_port, reserved)
        values ($1::varchar, $2::varchar, $3::machine_arch, $4::machine_platform, $5::varchar, $6::varchar, $7::varchar,
            $8::varchar, $9::boolean, $10::timestamp, $11::varchar, $12::timestamp, $13::varchar, $14::varchar, $15::bool)
        returning id, name, label, arch AS "arch!: MachineArch", platform AS "platform!: MachinePlatform", ip, tags, interface, snapshot, locked, locked_changed_on, status, status_changed_on, result_server_ip, result_server_port, reserved
        "#,
        machine.name,
        machine.label,
        machine.arch as MachineArch,
        machine.platform as MachinePlatform,
        machine.ip,
        machine.tags,
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
    .context("failed to insert sample")
}

pub async fn clean_machines(pool: &PgPool) -> anyhow::Result<(), anyhow::Error> {
    query!(
        r#"
        TRUNCATE "machines";
        "#
    )
    .execute(pool)
    .await
    .context("failed to truncate `machines` table")?;

    query!(
        r#"
        DELETE FROM "machines";
        "#
    )
    .execute(pool)
    .await
    .context("failed to delete from `machines` table")?;

    Ok(())
}

pub async fn fetch_machines(
    pool: &PgPool,
    filter: Option<MachineFilter>,
) -> anyhow::Result<Vec<MachineEntity>> {
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
        .context("failed to fetch machines");

    query
}

pub async fn fetch_machine(
    pool: &PgPool,
    filter: Option<MachineFilter>,
) -> anyhow::Result<Option<MachineEntity>> {
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
        .context("failed to fetch machines");

    query
}
