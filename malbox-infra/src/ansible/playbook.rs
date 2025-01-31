use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookConfig {
    pub name: String,
    pub inventory: Option<String>,
    pub r#become: bool,
    pub tags: Vec<String>,
    pub variables: HashMap<String, String>,
}

pub struct PlaybookManager {
    config: malbox_config::Config,
}

impl PlaybookManager {
    pub fn new(config: malbox_config::Config) -> Self {
        Self { config }
    }

    pub async fn run(&self, config: PlaybookConfig) -> Result<()> {
        let playbook_path = self.get_playbook_path(&config.name);

        let mut cmd = Command::new("ansible-playbook");
        cmd.arg(&playbook_path);

        if config.r#become {
            cmd.arg("--become");
        }

        if let Some(inventory) = config.inventory {
            cmd.arg("-i").arg(inventory);
        }

        for (key, value) in config.variables {
            cmd.arg("-e").arg(format!("{}={}", key, value));
        }

        if !config.tags.is_empty() {
            cmd.arg("--tags").arg(config.tags.join(","));
        }

        info!("Running ansible-playbook command");
        let output = cmd.output().await?;

        if !output.status.success() {
            debug!(
                "Playbook output: {}",
                String::from_utf8_lossy(&output.stdout)
            );
            return Err(Error::Ansible(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    fn get_playbook_path(&self, name: &str) -> PathBuf {
        self.config
            .paths
            .ansible_dir
            .join("playbooks")
            .join(format!("{}.yml", name))
    }
}
