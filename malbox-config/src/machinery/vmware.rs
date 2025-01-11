use super::{MachineConfig, MachineProvider};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct VmwareConfig {
    pub vcenter: VCenterConfig,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    pub machines: Vec<MachineConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VCenterConfig {
    pub server: String,
    pub username: String,
    pub password: String,
    pub datacenter: String,
    pub cluster: String,
    pub resource_pool: Option<String>,
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

impl MachineProvider for VmwareConfig {
    fn get_machines(&self) -> &Vec<MachineConfig> {
        &self.machines
    }
}
