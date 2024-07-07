use super::{CommonHypervisor, CommonMachine, HypervisorConfig};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct KvmConfig {
    pub kvm: Kvm,
    pub machines: Vec<MachineConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Kvm {
    pub dsn: String,
    #[serde(flatten)]
    pub common: CommonHypervisor,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MachineConfig {
    pub label: String,
    #[serde(flatten)]
    pub common: CommonMachine,
}

impl HypervisorConfig for KvmConfig {
    fn get_common_machine(&self) -> Vec<&CommonMachine> {
        let mut vec = Vec::new();
        for machine in &self.machines {
            vec.push(&machine.common)
        }
        vec
    }
    fn get_common_hypervisor(&self) -> &CommonHypervisor {
        &self.kvm.common
    }
}
