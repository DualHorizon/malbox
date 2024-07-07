use malbox_config::machinery::MachineryConfig;

use crate::machinery::{kvm, qemu};

pub async fn setup_machinery(config: &MachineryConfig) {
    match &config {
        MachineryConfig::VirtualBox(config) => {
            todo!()
        }
        MachineryConfig::Vmware(config) => {
            todo!()
        }
        MachineryConfig::Kvm(config) => {
            if let Err(err) = kvm::libvirt_init(config).await {
                tracing::error!("Failed to initialize KVM machinery: {:?}", err);
                std::process::exit(1);
            }
        }
    }
}
