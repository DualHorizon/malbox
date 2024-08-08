use anyhow::Context;
use malbox_config::machinery::KvmConfig;
use malbox_database::PgPool;
use virt::connect::Connect;
use virt::domain::Domain;
use virt::domain_snapshot::DomainSnapshot;

// TODOs for KVM:
// setup init checks for DSN
// setup checks for interface
// store VNC port for guac

pub async fn start_machine(label: &str, snapshot: Option<String>) -> anyhow::Result<()> {
    let mut conn = Connect::open(Some("qemu:///system"))
        .with_context(|| format!("Failed to connect to the KVM DSN: qemu:///system"))?;

    tracing::debug!("Successfully connected to dsn: qemu:///system");

    let mut dom = Domain::lookup_by_name(&conn, &label)
        .with_context(|| format!("Failed to find domain: {}", &label))?;

    tracing::debug!("Successfully found domain: {}", &label);

    // TODO Here, handle errors correctly, close the connection if errors, check for snapshots even when not configured and use the first parent.
    if let Some(snapshot) = snapshot {
        tracing::debug!(
            "Found snapshot `{}` for domain `{}` in database, reverting..",
            snapshot,
            label
        );

        let dom_snapshot = DomainSnapshot::lookup_by_name(&dom, &snapshot, 0)
            .with_context(|| format!("Failed to find snapshot {}", snapshot))?;

        DomainSnapshot::revert(&dom_snapshot, 0)
            .with_context(|| format!("Failed to revert to snapshot {}", snapshot))?;
    }
    // need to handle the case where machine is already started

    Domain::create(&dom)?;

    dom.free()?;

    tracing::debug!("Sucessfully started the machine");

    Ok(())
}

pub async fn shutdown_machine(label: &str) -> anyhow::Result<()> {
    let mut conn = Connect::open(Some("qemu:///system"))
        .with_context(|| format!("Failed to connect to the KVM DSN: qemu:///system"))?;

    let mut dom = Domain::lookup_by_name(&conn, &label)
        .with_context(|| format!("Failed to find domain: {}", &label))?;

    Domain::destroy(&dom)?;

    dom.free()?;

    tracing::info!("Successfully shutdown for label: {}", label);

    Ok(())
}
