use clap::ValueEnum;
use malbox_infra::Platform as InfraPlatformType;
use serde::{Deserialize, Serialize};

#[derive(Clone, ValueEnum, Debug, Serialize, Deserialize, PartialEq)]
pub enum PlatformType {
    Windows,
    Linux,
}

impl From<PlatformType> for InfraPlatformType {
    fn from(value: PlatformType) -> Self {
        match value {
            PlatformType::Linux => InfraPlatformType::Linux,
            PlatformType::Windows => InfraPlatformType::Windows,
        }
    }
}

#[derive(Clone, ValueEnum, Debug, Serialize, Deserialize)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
}
