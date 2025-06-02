use crate::error::{AnsibleError, Result};
use malbox_io_utils::process::AsyncCommand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
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

        let mut cmd = AsyncCommand::new("ansible-playbook").arg(playbook_path.to_str().unwrap());

        if config.r#become {
            cmd = cmd.arg("--become");
        }

        if let Some(inventory) = config.inventory {
            cmd = cmd.arg("-i").arg(inventory);
        }

        for (key, value) in config.variables {
            cmd = cmd.arg("-e").arg(format!("{}={}", key, value));
        }

        if !config.tags.is_empty() {
            cmd = cmd.arg("--tags").arg(config.tags.join(","));
        }

        info!("Running ansible-playbook command");
        let output = cmd
            .run()
            .await
            .map_err(|e| AnsibleError::Execution { source: e })?;

        if !output.success() {
            debug!("Playbook output: {}", output.stdout());
            return Err(AnsibleError::AnsibleNotFound);
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
