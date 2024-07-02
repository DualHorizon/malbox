use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VirtualBoxConfig {
    pub headless: bool,
    pub base_folder: String,
    pub vboxmanage_path: String,
    pub machines: std::collections::HashMap<String, MachineConfig>,
}

#[derive(Debug, Deserialize)]
pub struct MachineConfig {
    pub vdi_path: String,
    pub state: String,
    pub platform: String,
    pub ip: String,
    pub arch: String,
    pub tags: Option<Vec<String>>,
}
