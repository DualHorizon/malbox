use malbox_config::core::DatabaseConfig;
use malbox_config::machinery::{MachineProvider, MachineryConfig, ProviderConfig};
use repositories::machinery::{clean_machines, insert_machine, Machine};
pub use sqlx::error::DatabaseError;
use sqlx::postgres::PgPoolOptions;
pub use sqlx::Error;
pub use sqlx::PgPool;

pub mod repositories;

pub async fn init_database(config: &DatabaseConfig) -> sqlx::Pool<sqlx::Postgres> {
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.host)
        .await
        .unwrap();

    sqlx::migrate!().run(&db).await.unwrap();

    db
}

pub async fn init_machines(pool: &PgPool, config: &MachineryConfig) -> anyhow::Result<()> {
    clean_machines(pool).await.unwrap();

    let machines = match &config.provider {
        ProviderConfig::Vmware(vmware_config) => vmware_config.get_machines(),
        ProviderConfig::Kvm(kvm_config) => kvm_config.get_machines(),
        ProviderConfig::VirtualBox(vbox_config) => vbox_config.get_machines(),
    };

    for machine_config in machines {
        let db_machine = Machine {
            name: machine_config.name.clone(),
            label: machine_config.label.clone().unwrap_or_default(),
            arch: machine_config.arch.clone().into(),
            platform: machine_config.platform.clone().into(),
            ip: machine_config.ip.clone(),
            tags: machine_config.tags.clone(),
            interface: machine_config.interface.clone(),
            snapshot: machine_config.snapshot.clone(),
            result_server_ip: machine_config.result_server.as_ref().map(|s| s.ip.clone()),
            result_server_port: machine_config
                .result_server
                .as_ref()
                .map(|s| s.port.to_string()),
            reserved: machine_config.reserved,
            ..Machine::default()
        };

        insert_machine(pool, db_machine).await?;
    }

    Ok(())
}
