use anyhow::Context;
use malbox_config::machinery::{
    MachineArch as MachineArchConfig, MachinePlatform as MachinePlatformConfig,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, query, query_as, FromRow, PgPool};
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
    pub id: i64,
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
    locked: Option<bool>,
    label: Option<String>,
    platform: Option<MachinePlatform>,
    tags: Option<String>,
    arch: Option<MachineArch>,
    include_reserved: bool,
    os_version: Option<String>,
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

//TODO
pub async fn fetch_machines(
    pool: &PgPool,
    filter: Option<MachineFilter>,
) -> anyhow::Result<Vec<MachineEntity>> {
    // the query will be adjusted depending on other params to filter out specific machines
    query_as!(
        MachineEntity,
        r#"
            SELECT id, name, label, arch AS "arch!: MachineArch", platform AS "platform!: MachinePlatform", ip, tags, interface, snapshot, locked, locked_changed_on, status, status_changed_on, result_server_ip, result_server_port, reserved
            FROM machines
        "#
    )
    .fetch_all(pool)
    .await
    .context("failed to fetch machines")
}
