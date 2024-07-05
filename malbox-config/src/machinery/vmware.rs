use super::{CommonHypervisor, CommonMachine, HypervisorConfig};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct VmwareConfig {
    pub vmware: Vmware,
    pub machines: Vec<MachineConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Vmware {
    pub mode: String,
    pub vmrun_path: String,
    #[serde(flatten)]
    pub common: CommonHypervisor,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MachineConfig {
    pub vmx_path: String,
    #[serde(flatten)]
    pub common: CommonMachine,
}

impl HypervisorConfig for VmwareConfig {
    fn get_common_machine(&self) -> Vec<&CommonMachine> {
        let mut vec = Vec::new();
        for machine in &self.machines {
            vec.push(&machine.common)
        }
        vec
    }
    fn get_common_hypervisor(&self) -> &CommonHypervisor {
        &self.vmware.common
    }
}
