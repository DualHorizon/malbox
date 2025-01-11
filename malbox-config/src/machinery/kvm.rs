use super::{MachineConfig, MachineProvider};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct KvmConfig {
    pub uri: String,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    pub machines: Vec<MachineConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    pub interface: String,
    pub address_range: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub path: String,
}

impl MachineProvider for KvmConfig {
    fn get_machines(&self) -> &Vec<MachineConfig> {
        &self.machines
    }
}
