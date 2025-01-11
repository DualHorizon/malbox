use super::{MachineConfig, MachineProvider};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct VirtualBoxConfig {
    pub machine_path: String,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    pub machines: Vec<MachineConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    pub interface: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub path: String,
}

impl MachineProvider for VirtualBoxConfig {
    fn get_machines(&self) -> &Vec<MachineConfig> {
        &self.machines
    }
}
