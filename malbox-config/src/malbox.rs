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
    pub hypervisor: String,
}

pub fn load_config() -> Result<MalboxConfig, String> {
    let file_name = "./configuration/malbox.toml";

    let contents = fs::read_to_string(file_name)
        .map_err(|e| format!("Could not read file `{}`: {}", file_name, e))?;

    toml::from_str(&contents).map_err(|e| format!("Failed to parse malbox config: {}", e))
}
