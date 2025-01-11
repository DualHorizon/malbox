use crate::machinery::MachineryConfig;
use crate::profiles::ProfileConfig;
use crate::templates::TemplateConfig;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub malbox: MalboxConfig,
    pub machinery: MachineryConfig,
    pub profiles: ProfileConfig,
    pub templates: TemplateConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MalboxConfig {
    pub http: HttpConfig,
    pub database: DatabaseConfig,
    pub debug: DebugConfig,
    pub analysis: AnalysisConfig,
    pub infrastructure: InfrastructureConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HttpConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DebugConfig {
    pub rust_log: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InfrastructureConfig {
    pub provider: String,
    pub environment: String,
}

// NOTE: this will maybe be moved out in the future,
// Analysis configuration could grow - it may be better to place it in seperate folders
#[derive(Debug, Clone, Deserialize)]
pub struct AnalysisConfig {
    pub timeout: u32,
    pub max_vms: u32,
    pub default_profile: String,
    pub windows: PlatformAnalysisConfig,
    pub linux: PlatformAnalysisConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlatformAnalysisConfig {
    pub default_profile: String,
    pub timeout: Option<u32>,
    pub max_vms: Option<u32>,
}
