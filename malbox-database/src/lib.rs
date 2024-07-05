use malbox_config::machinery::MachineryConfig;
use malbox_config::malbox::Postgres;
use repositories::machinery::{
    clean_machines, insert_machine, Machine, MachineArch, MachinePlatform,
};
pub use sqlx::error::DatabaseError;
use sqlx::postgres::PgPoolOptions;
pub use sqlx::Error;
pub use sqlx::PgPool;

pub mod repositories;

pub async fn init_database(config: &Postgres) -> sqlx::Pool<sqlx::Postgres> {
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .unwrap();

    sqlx::migrate!().run(&db).await.unwrap();

    db
}

pub async fn init_machines(pool: &PgPool, machine_config: &MachineryConfig) {
    clean_machines(pool).await.unwrap();
    let machines = machine_config.get_common_machine();
    for machine in machines {
        let db_machine = Machine {
            name: machine.name.clone(),
            label: "".to_string(),
            arch: MachineArch::from(machine.arch.clone()),
            platform: MachinePlatform::from(machine.platform.clone()),
            ip: machine.ip.clone(),
            interface: machine.interface.clone(),
            snapshot: machine.snapshot.clone(),
            result_server_ip: machine.result_server_ip.clone(),
            result_server_port: machine.result_server_port.clone(),
            ..Machine::default()
        };

        insert_machine(pool, db_machine).await.unwrap();
    }
}
