use super::{MachineConfig, MachineProvider};
use bon::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct VmwareConfig {
    pub vcenter: VCenterConfig,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    pub machines: Vec<MachineConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct VCenterConfig {
    pub server: String,
    pub username: String,
    pub password: Option<String>,
    pub password_env: Option<String>,
    pub datacenter: String,
    pub cluster: String,
    pub resource_pool: Option<String>,
    #[builder(default = false)]
    pub insecure_ssl: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct NetworkConfig {
    pub name: String,
    pub interface: String,
    pub vlan: Option<u16>,
    #[builder(default = false)]
    pub promiscuous: bool,
    #[builder(default = "\"vmxnet3\"".to_string())]
    pub adapter_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct StorageConfig {
    pub datastore: String,
    #[builder(default = 100)]
    pub default_size_gb: u32,
    #[builder(default = DiskFormat::Vmdk)]
    pub format: DiskFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiskFormat {
    #[serde(rename = "vmdk")]
    Vmdk,
    #[serde(rename = "vhd")]
    Vhd,
}

impl MachineProvider for VmwareConfig {
    fn get_machines(&self) -> &[MachineConfig] {
        &self.machines
    }
}
