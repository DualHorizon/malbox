use crate::Provider;
use crate::{machinery::MachineryConfig, profiles::ProfileConfig, Environment, LogLevel, Paths};
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct Config {
    pub paths: Paths,
    pub general: GeneralConfig,
    pub http: HttpConfig,
    pub database: DatabaseConfig,
    pub machinery: MachineryConfig,
    pub profiles: ProfileConfig,
    pub analysis: AnalysisConfig,
    #[serde(default)]
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct GeneralConfig {
    pub environment: Environment,
    pub provider: Provider,
    #[serde(default = "default_log_level")]
    pub log_level: LogLevel,
    #[builder(default = false)]
    pub debug: bool,
    #[builder(default = 4)]
    pub worker_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct HttpConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub tls_enabled: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    #[serde(default)]
    pub cors_origins: Vec<String>,
    #[serde(default)]
    pub max_upload_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    // pub username: String,
    // pub password: Option<String>,
    // pub password_env: Option<String>,
    // pub database: String,
    // #[serde(default = 10)]
    // pub max_connections: u32,
    // #[serde(default = true)]
    // pub ssl_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct AnalysisConfig {
    pub timeout: u32,
    pub max_vms: u32,
    pub default_profile: String,
    pub windows: PlatformAnalysisConfig,
    pub linux: PlatformAnalysisConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct PlatformAnalysisConfig {
    pub default_profile: String,
    pub timeout: Option<u32>,
    pub max_vms: Option<u32>,
}

fn default_log_level() -> LogLevel {
    LogLevel::Info
}
