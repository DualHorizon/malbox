use super::{MachineConfig, MachineProvider};
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct VirtualBoxConfig {
    pub machine_path: PathBuf,
    pub network: VboxNetwork,
    pub storage: StorageConfig,
    pub machines: Vec<MachineConfig>,
    #[builder(default = 4)]
    pub cpus: u32,
    #[builder(default = 8192)]
    pub memory: u32,
    #[builder(default = 128)]
    pub vram: u32,
    #[builder(default = false)]
    pub headless: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct VboxNetwork {
    pub name: String,
    pub interface: String,
    #[builder(default = "\"hostonly\"".to_string())]
    pub mode: String,
    pub bridge: Option<String>,
    #[builder(default)]
    pub ip_ranges: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct StorageConfig {
    pub path: PathBuf,
    #[builder(default = DiskFormat::Vdi)]
    pub format: DiskFormat,
    #[builder(default = 100)]
    pub default_size_gb: u32,
    #[builder(default = StorageController::Sata)]
    pub controller: StorageController,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiskFormat {
    #[serde(rename = "vdi")]
    Vdi,
    #[serde(rename = "vhd")]
    Vhd,
    #[serde(rename = "vmdk")]
    Vmdk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageController {
    #[serde(rename = "sata")]
    Sata,
    #[serde(rename = "ide")]
    Ide,
    #[serde(rename = "scsi")]
    Scsi,
}

impl MachineProvider for VirtualBoxConfig {
    fn get_machines(&self) -> &[MachineConfig] {
        &self.machines
    }
}
