use core::fmt;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug, Clone)]
pub struct MalboxConfig {
    pub http: Http,
    pub postgres: Postgres,
    pub debug: Debug,
    pub machinery: Machinery,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Http {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Postgres {
    pub database_url: String,
    pub port: u16,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Debug {
    pub rust_log: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Machinery {
    #[serde(rename = "type")]
    pub _type: MachineryType,
}

#[derive(Deserialize, Debug, Clone)]
pub enum MachineryType {
    #[serde(alias = "kvm")]
    Kvm,
    #[serde(alias = "Virtualbox", alias = "virtualbox")]
    VirtualBox,
    #[serde(alias = "vmware")]
    Vmware,
}

impl fmt::Display for MachineryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn load_config() -> Result<MalboxConfig, String> {
    let file_name = "./configuration/malbox.toml";

    let contents = fs::read_to_string(file_name)
        .map_err(|e| format!("Could not read file `{}`: {}", file_name, e))?;

    toml::from_str(&contents).map_err(|e| format!("Failed to parse malbox config: {}", e))
}
