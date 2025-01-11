use crate::shared_types::MachinePlatform;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct ProfileConfig {
    pub defaults: HashMap<String, Profile>,
    pub custom: HashMap<String, Profile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Profile {
    pub name: String,
    pub description: String,
    pub timeout: Option<u32>,
    pub max_vms: Option<u32>,
    pub platform: MachinePlatform,
    pub analysis_options: HashMap<String, String>,
}

impl ProfileConfig {
    pub async fn load(config_root: &Path) -> anyhow::Result<Self> {
        let defaults = Self::load_profiles(config_root.join("profiles").join("default")).await?;
        let custom = Self::load_profiles(config_root.join("profiles").join("custom")).await?;

        Ok(ProfileConfig { defaults, custom })
    }

    async fn load_profiles(path: PathBuf) -> anyhow::Result<HashMap<String, Profile>> {
        let mut profiles = HashMap::new();
        let mut entries = tokio::fs::read_dir(path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension() == Some("toml".as_ref()) {
                let content = tokio::fs::read_to_string(entry.path()).await?;
                let profile: Profile = toml::from_str(&content)?;
                profiles.insert(profile.name.clone(), profile);
            }
        }

        Ok(profiles)
    }
}
