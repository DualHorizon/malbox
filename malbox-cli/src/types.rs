use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, ValueEnum, Debug, Serialize, Deserialize)]
pub enum PlatformType {
    Windows,
    Linux,
}

#[derive(Clone, ValueEnum, Debug, Serialize, Deserialize)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
}
