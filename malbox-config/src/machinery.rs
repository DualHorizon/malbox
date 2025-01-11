use anyhow::Context;
use serde::Deserialize;
use std::path::Path;

pub mod kvm;
pub mod virtualbox;
pub mod vmware;

pub use kvm::KvmConfig;
pub use virtualbox::VirtualBoxConfig;
pub use vmware::VmwareConfig;

#[derive(Debug, Clone, Deserialize)]
pub enum MachineArch {
    #[serde(alias = "x86")]
    X86,
    #[serde(alias = "x64")]
    X64,
}

#[derive(Debug, Clone, Deserialize)]
pub enum MachinePlatform {
    #[serde(alias = "windows")]
    Windows,
    #[serde(alias = "linux")]
    Linux,
    #[serde(alias = "macos")]
    MacOs,
}

// NOTE: Some of the fields are still just placeholders,
// those are NOT final configuration structs and vars, will need to be worked on in the future,
// but for now, it is enough so that we can proceed with development on other parts of the project
#[derive(Debug, Clone, Deserialize)]
pub struct MachineConfig {
    pub name: String,
    pub platform: MachinePlatform,
    pub arch: MachineArch,
    pub ip: String,
    pub tags: Option<String>,
    pub label: Option<String>,
    pub snapshot: Option<String>,
    pub interface: Option<String>,
    pub result_server: Option<ResultServer>,
    pub reserved: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResultServer {
    pub ip: String,
    pub port: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MachineryConfig {
    pub provider: ProviderConfig,
    pub terraform: TerraformConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ProviderConfig {
    #[serde(rename = "vmware")]
    Vmware(VmwareConfig),
    #[serde(rename = "kvm")]
    Kvm(KvmConfig),
    #[serde(rename = "virtualbox")]
    VirtualBox(VirtualBoxConfig),
}

#[derive(Debug, Clone, Deserialize)]
pub struct TerraformConfig {
    #[serde(default = "default_state_dir")]
    pub state_dir: String,
    pub variables: std::collections::HashMap<String, String>,
}

fn default_state_dir() -> String {
    "./machinery/terraform".to_string()
}

pub trait MachineProvider {
    fn get_machines(&self) -> &Vec<MachineConfig>;
}

impl MachineryConfig {
    pub async fn load(config_root: &Path, provider_type: &str) -> anyhow::Result<Self> {
        let provider_path = config_root
            .join("machinery")
            .join("providers")
            .join(provider_type)
            .join(format!("{}.default.toml", provider_type));

        let config_str = tokio::fs::read_to_string(&provider_path)
            .await
            .context("Failed to read provider config")?;

        let provider_config: ProviderConfig =
            toml::from_str(&config_str).context("Failed to parse provider config")?;

        Ok(MachineryConfig {
            provider: provider_config,
            terraform: TerraformConfig {
                state_dir: default_state_dir(),
                variables: std::collections::HashMap::new(),
            },
        })
    }
}
