use crate::{ConfigError, Platform};
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct ProfileConfig {
    #[builder(default)]
    pub defaults: HashMap<String, Profile>,
    #[builder(default)]
    pub custom: HashMap<String, Profile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct Profile {
    pub name: String,
    pub description: String,
    pub platform: Platform,
    #[builder(default = 300)]
    pub timeout: u32,
    #[builder(default = 5)]
    pub max_vms: u32,
    #[builder(default = HashMap::new())]
    pub analysis_options: HashMap<String, String>,
    #[builder(default = Vec::new())]
    pub tools: Vec<Tool>,
    #[builder(default = false)]
    pub network_isolated: bool,
    pub result_server: Option<ResultServer>,
    #[builder(default)]
    pub environment_vars: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct Tool {
    pub name: String,
    pub version: Option<String>,
    pub source: ToolSource,
    #[builder(default)]
    pub env_vars: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolSource {
    #[serde(rename = "chocolatey")]
    Chocolatey { package: String },
    #[serde(rename = "apt")]
    Apt { package: String },
    #[serde(rename = "url")]
    Url {
        url: String,
        checksum: Option<String>,
    },
    #[serde(rename = "local")]
    Local { path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct ResultServer {
    pub ip: String,
    pub port: u16,
    #[builder(default = Protocol::Https)]
    pub protocol: Protocol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Protocol {
    #[serde(rename = "http")]
    Http,
    #[serde(rename = "https")]
    Https,
}

impl ProfileConfig {
    pub async fn load(config_root: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let defaults =
            Self::load_profiles(config_root.as_ref().join("profiles").join("default")).await?;
        let custom =
            Self::load_profiles(config_root.as_ref().join("profiles").join("custom")).await?;

        Ok(Self::builder().defaults(defaults).custom(custom).build())
    }

    async fn load_profiles(
        path: impl AsRef<Path>,
    ) -> Result<HashMap<String, Profile>, ConfigError> {
        let mut profiles = HashMap::new();
        let mut entries = fs::read_dir(path.as_ref()).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension() == Some("toml".as_ref()) {
                let content = fs::read_to_string(entry.path()).await?;
                let profile: Profile =
                    toml::from_str(&content).map_err(|e| ConfigError::Parse {
                        file: entry.path().display().to_string(),
                        error: e.to_string(),
                    })?;
                profiles.insert(profile.name.clone(), profile);
            }
        }

        Ok(profiles)
    }

    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.custom.get(name).or_else(|| self.defaults.get(name))
    }

    pub fn get_profiles_for_platform(&self, platform: Platform) -> Vec<&Profile> {
        self.defaults
            .values()
            .chain(self.custom.values())
            .filter(|p| p.platform == platform)
            .collect()
    }
}
