use anyhow::Context;
use malbox_config::machinery::KvmConfig;
use malbox_database::PgPool;
use virt::connect::Connect;
use virt::domain::Domain;

pub async fn start(db: PgPool) -> anyhow::Result<()> {
    // let mut conn = Connect::open(&config.kvm.dsn)
    //     .with_context(|| format!("Failed to connect to the KVM DSN: {}", &config.kvm.dsn))?;
    // tracing::debug!("Successfully connected to dsn: {}", &config.kvm.dsn);

    // for machine in &config.machines {
    //     Domain::lookup_by_name(&conn, &machine.label)
    //         .with_context(|| format!("Failed to find domain: {}", &machine.label))?;

    //     tracing::debug!("Successfully found domain: {}", &machine.label);
    // }

    Ok(())
}
