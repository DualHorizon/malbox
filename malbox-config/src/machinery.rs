use crate::ConfigError;
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

pub mod kvm;
pub mod virtualbox;
pub mod vmware;

pub use kvm::KvmConfig;
pub use virtualbox::VirtualBoxConfig;
pub use vmware::VmwareConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProviderConfig {
    #[serde(rename = "vmware")]
    Vmware(VmwareConfig),
    #[serde(rename = "kvm")]
    Kvm(KvmConfig),
    #[serde(rename = "virtualbox")]
    VirtualBox(VirtualBoxConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct MachineryConfig {
    pub provider: ProviderConfig,
    #[builder(default)]
    pub terraform: TerraformConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
pub struct TerraformConfig {
    #[builder(default = "./machinery/terraform".to_string())]
    pub state_dir: String,
    #[builder(default)]
    pub variables: HashMap<String, String>,
    #[builder(default)]
    pub backend_config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct MachineConfig {
    pub name: String,
    pub label: Option<String>,
    pub platform: crate::types::Platform,
    #[builder(default = MachineArch::X64)]
    pub arch: MachineArch,
    pub ip: String,
    pub tags: Option<Vec<String>>,
    pub snapshot: Option<String>,
    pub interface: Option<String>,
    pub result_server: Option<ResultServer>,
    #[builder(default = false)]
    pub reserved: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MachineArch {
    X86,
    X64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct ResultServer {
    pub ip: String,
    pub port: u16,
}

pub trait MachineProvider {
    fn get_machines(&self) -> &[MachineConfig];
}

impl MachineryConfig {
    pub async fn load(config_root: &Path, provider_type: &str) -> Result<Self, ConfigError> {
        let provider_path = config_root
            .join("providers")
            .join(provider_type)
            .join(format!("{}.default.toml", provider_type));

        tracing::debug!("current path: {:#?}", provider_path);

        let content = tokio::fs::read_to_string(&provider_path)
            .await
            .map_err(ConfigError::from)?;

        let provider: ProviderConfig =
            toml::from_str(&content).map_err(|e| ConfigError::Parse {
                file: provider_path.display().to_string(),
                error: e.to_string(),
            })?;

        Ok(Self::builder().provider(provider).build())
    }

    pub fn get_provider_config(&self) -> Result<&ProviderConfig, ConfigError> {
        Ok(&self.provider)
    }
}
