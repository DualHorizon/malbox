use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryConfig {
    pub name: String,
    pub hosts: Vec<Host>,
    pub groups: Vec<Group>,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    pub name: String,
    pub address: String,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub hosts: Vec<String>,
    pub variables: HashMap<String, String>,
}

pub struct InventoryManager {
    config: malbox_config::Config,
}

impl InventoryManager {
    pub fn new(config: malbox_config::Config) -> Self {
        Self { config }
    }

    pub async fn create(&self, inventory: InventoryConfig) -> Result<()> {
        let path = self.get_inventory_path(&inventory.name);
        let content = self.generate_inventory(&inventory)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    fn generate_inventory(&self, inventory: &InventoryConfig) -> Result<String> {
        let mut content = String::new();
        content.push_str("all:\n  hosts:\n");

        for host in &inventory.hosts {
            content.push_str(&format!("    {}:\n", host.name));
            content.push_str(&format!("      ansible_host: {}\n", host.address));
            for (key, value) in &host.variables {
                content.push_str(&format!("      {}: {}\n", key, value));
            }
        }

        content.push_str("  children:\n");
        for group in &inventory.groups {
            content.push_str(&format!("    {}:\n", group.name));
            content.push_str("      hosts:\n");
            for host in &group.hosts {
                content.push_str(&format!("        {}:\n", host));
            }
            if !group.variables.is_empty() {
                content.push_str("      vars:\n");
                for (key, value) in &group.variables {
                    content.push_str(&format!("        {}: {}\n", key, value));
                }
            }
        }

        if !inventory.variables.is_empty() {
            content.push_str("  vars:\n");
            for (key, value) in &inventory.variables {
                content.push_str(&format!("    {}: {}\n", key, value));
            }
        }

        Ok(content)
    }

    fn get_inventory_path(&self, name: &str) -> PathBuf {
        self.config
            .paths
            .ansible_dir
            .join("inventories")
            .join(format!("{}.yml", name))
    }
}
