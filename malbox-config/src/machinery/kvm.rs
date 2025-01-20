use super::{MachineConfig, MachineProvider};
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct KvmConfig {
    pub uri: String,
    pub network: KvmNetwork,
    pub storage: StorageConfig,
    pub machines: Vec<MachineConfig>,
    #[builder(default = "4")]
    pub cpus: u32,
    #[builder(default = "8192")]
    pub memory: u32,
    #[builder(default = "128")]
    pub video_memory: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct KvmNetwork {
    pub name: String,
    pub interface: String,
    pub address_range: String,
    #[builder(default = "None")]
    pub bridge: Option<String>,
    #[builder(default = "false")]
    pub nat_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct StorageConfig {
    pub path: PathBuf,
    #[builder(default = "StorageType::Raw")]
    pub storage_type: StorageType,
    #[builder(default = "100")]
    pub default_size_gb: u32,
    #[builder(default = "\"virtio\"")]
    pub bus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    Raw,
    Qcow2,
}

impl MachineProvider for KvmConfig {
    fn get_machines(&self) -> &[MachineConfig] {
        &self.machines
    }
}
