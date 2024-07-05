use super::{CommonHypervisor, CommonMachine, HypervisorConfig};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VirtualBoxConfig {
    pub virtualbox: VirtualBox,
    pub machines: Vec<MachineConfig>,
}

#[derive(Debug, Deserialize)]
pub struct VirtualBox {
    pub headless: bool,
    pub vboxmanage_path: String,
    #[serde(flatten)]
    pub common: CommonHypervisor,
}

#[derive(Debug, Deserialize)]
pub struct MachineConfig {
    pub vdi_path: String,
    pub common: CommonMachine,
}

impl HypervisorConfig for VirtualBoxConfig {
    fn get_common_machine(&self) -> Vec<&CommonMachine> {
        let mut vec = Vec::new();
        for machine in &self.machines {
            vec.push(&machine.common)
        }
        vec
    }
    fn get_common_hypervisor(&self) -> &CommonHypervisor {
        &self.virtualbox.common
    }
}
