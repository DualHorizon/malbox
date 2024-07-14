use std::{fs, path::Path};

use serde::Deserialize;

pub use kvm::KvmConfig;
use virtualbox::VirtualBoxConfig;
use vmware::VmwareConfig;

use super::malbox::MachineryType;

mod kvm;
mod virtualbox;
mod vmware;

#[derive(Debug, Deserialize, Clone)]
pub enum MachineArch {
    #[serde(alias = "x86")]
    X86,
    #[serde(alias = "x64")]
    X64,
}

#[derive(Debug, Deserialize, Clone)]
pub enum MachinePlatform {
    #[serde(alias = "windows")]
    Windows,
    #[serde(alias = "linux")]
    Linux,
    #[serde(alias = "macos", alias = "Macos")]
    MacOs,
}

#[derive(Debug)]
pub enum MachineryConfig {
    Vmware(VmwareConfig),
    VirtualBox(VirtualBoxConfig),
    Kvm(KvmConfig),
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommonHypervisor {
    pub interface: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommonMachine {
    pub name: String,
    pub snapshot: Option<String>,
    pub platform: MachinePlatform,
    pub ip: String, // todo
    pub arch: MachineArch,
    pub tags: Option<Vec<String>>,
    pub interface: Option<String>,
    pub result_server_ip: Option<String>,
    pub result_server_port: Option<String>,
    pub reserved: Option<bool>,
}

trait HypervisorConfig {
    fn get_common_machine(&self) -> Vec<&CommonMachine>;
    fn get_common_hypervisor(&self) -> &CommonHypervisor;
}

impl MachineryConfig {
    pub fn get_common_machine(&self) -> Vec<&CommonMachine> {
        match &self {
            MachineryConfig::Vmware(config) => config.get_common_machine(),
            MachineryConfig::VirtualBox(config) => config.get_common_machine(),
            MachineryConfig::Kvm(config) => config.get_common_machine(),
        }
    }

    pub fn get_common_hypervisor(&self) -> &CommonHypervisor {
        match &self {
            MachineryConfig::Vmware(config) => config.get_common_hypervisor(),
            MachineryConfig::VirtualBox(config) => config.get_common_hypervisor(),
            MachineryConfig::Kvm(config) => config.get_common_hypervisor(),
        }
    }
}

pub fn load_config(machinery_type: &MachineryType) -> Result<MachineryConfig, String> {
    let machinery_type_str = machinery_type.to_string().to_lowercase();
    let specific_path = format!("./configuration/machinery/{}.toml", machinery_type_str);
    let default_path = format!(
        "./configuration/machinery/default/{}.default.toml",
        machinery_type_str
    );

    let config_path = if Path::new(&specific_path).exists() {
        specific_path
    } else if Path::new(&default_path).exists() {
        default_path
    } else {
        return Err(format!(
            "No configuration file found for hypervisor `{}`",
            machinery_type_str
        ));
    };

    let contents = fs::read_to_string(&config_path)
        .map_err(|e| format!("Could not read file `{}`: {}", config_path, e))?;

    match machinery_type {
        MachineryType::Vmware => {
            let config: VmwareConfig = toml::from_str(&contents)
                .map_err(|e| format!("Failed to parse Vmware config: {}", e))?;
            Ok(MachineryConfig::Vmware(config))
        }
        MachineryType::VirtualBox => {
            let config: VirtualBoxConfig = toml::from_str(&contents)
                .map_err(|e| format!("Failed to parse VirtualBox config: {}", e))?;
            Ok(MachineryConfig::VirtualBox(config))
        }
        MachineryType::Kvm => {
            let config: KvmConfig = toml::from_str(&contents)
                .map_err(|e| format!("Failed to parse Kvm config: {}", e))?;
            Ok(MachineryConfig::Kvm(config))
        }
    }
}
