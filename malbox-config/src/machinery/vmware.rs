use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub struct VmwareConfig {
    pub vmware: Vmware,
    pub machines: HashMap<String, MachineConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Vmware {
    pub mode: String,
    pub interface: String,
    pub vmrun_path: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MachineConfig {
    pub name: String,
    pub vmx_path: String,
    pub snapshot: String,
    pub platform: String,
    pub ip: String, // todo
    pub arch: String,
    pub tags: Option<Vec<String>>,
    pub interface: Option<String>,
    pub result_server_ip: Option<String>,
    pub result_server_port: Option<String>,
}
