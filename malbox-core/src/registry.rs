use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::types::PluginType;

#[derive(Clone, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String, // TODO: implement semver
    pub path: PathBuf,   // TODO: get information from config loader
    pub plugin_type: PluginType,
    pub dependencies: HashSet<String>,              // TODO
    pub config: HashMap<String, serde_json::Value>, // TODO we should decide on a plugin configuration format!
}

pub struct PluginRegistry {
    plugins: HashMap<String, PluginInfo>,
}

impl PluginRegistry {
    pub fn new(config_path: PathBuf) -> anyhow::Result<Self> {
        // let config = std::fs::read_to_string(config_path)?;
        // let plugins: HashMap<String, PluginInfo> = serde_json::from_str(&config)?;
        // Ok(Self { plugins })

        todo!()
    }

    pub fn get_plugin(&self, id: &str) -> Option<&PluginInfo> {
        self.plugins.get(id)
    }

    pub fn verify_plugin(&self, id: &str) -> anyhow::Result<()> {
        let info = self
            .get_plugin(id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found {}", id))?;

        if !info.path.exists() {
            return Err(anyhow::anyhow!("Plugin binary not found: {:?}", info.path));
        }

        for dep in &info.dependencies {
            if !self.plugins.contains_key(dep) {
                return Err(anyhow::anyhow!("Missing dependency: {}", dep));
            }
        }

        Ok(())
    }
}
