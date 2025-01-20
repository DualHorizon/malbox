use crate::error::{Error, Result};
use bon::Builder;
use malbox_config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct PlaybookManager {
    config: Config,
    #[builder(default = false)]
    r#become: bool,
    inventory: Option<String>,
    #[builder(default)]
    extra_vars: HashMap<String, String>,
}

impl PlaybookManager {
    pub fn new(config: Config) -> Self {
        Self::builder().config(config).build()
    }

    pub async fn list_playbooks(&self) -> Result<Vec<Playbook>> {
        let playbook_dir = self.config.paths.ansible_dir.join("playbooks");
        let mut playbooks = Vec::new();
        let mut entries = tokio::fs::read_dir(&playbook_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension() == Some("yml".as_ref())
                || entry.path().extension() == Some("yaml".as_ref())
            {
                let content = tokio::fs::read_to_string(entry.path()).await?;
                if let Ok(playbook) = self.parse_playbook(&content, &entry.path()) {
                    playbooks.push(playbook);
                }
            }
        }

        Ok(playbooks)
    }

    pub async fn create_playbook(
        &self,
        name: &str,
        description: &str,
        roles: &[String],
    ) -> Result<()> {
        let playbook_path = self
            .config
            .paths
            .ansible_dir
            .join("playbooks")
            .join(format!("{}.yml", name));
        let content = self.generate_playbook_content(description, roles);
        tokio::fs::write(playbook_path, content).await?;
        Ok(())
    }

    pub async fn edit_playbook(&self, name: &str, editor: Option<&str>) -> Result<()> {
        let editor_string = editor
            .map(ToString::to_string)
            .or_else(|| std::env::var("EDITOR").ok())
            .unwrap_or_else(|| "vim".to_string());

        let playbook_path = self
            .config
            .paths
            .ansible_dir
            .join("playbooks")
            .join(format!("{}.yml", name));

        if !playbook_path.exists() {
            return Err(Error::NotFound(format!("Playbook {} not found", name)));
        }

        let status = Command::new(editor_string)
            .arg(&playbook_path)
            .status()
            .await?;

        if !status.success() {
            return Err(Error::Playbook("Editor exited with error".into()));
        }

        Ok(())
    }

    pub async fn apply_playbook(
        &self,
        name: &str,
        targets: &[String],
        check: bool,
        vars: &[(String, String)],
    ) -> Result<()> {
        let playbook_path = self
            .config
            .paths
            .ansible_dir
            .join("playbooks")
            .join(format!("{}.yml", name));

        if !playbook_path.exists() {
            return Err(Error::NotFound(format!("Playbook {} not found", name)));
        }

        let mut cmd = Command::new("ansible-playbook");
        cmd.arg(&playbook_path);

        if let Some(inventory) = &self.inventory {
            cmd.arg("-i").arg(inventory);
        }

        for target in targets {
            cmd.arg("-l").arg(target);
        }

        if check {
            cmd.arg("--check");
        }

        if self.r#become {
            cmd.arg("--become");
        }

        for (key, value) in vars {
            cmd.arg("-e").arg(format!("{}={}", key, value));
        }

        for (key, value) in &self.extra_vars {
            cmd.arg("-e").arg(format!("{}={}", key, value));
        }

        let output = cmd.output().await?;
        if !output.status.success() {
            return Err(Error::Playbook(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    pub async fn test_playbook(&self, name: &str, environment: &str) -> Result<()> {
        let inventory = self
            .config
            .paths
            .ansible_dir
            .join("inventories")
            .join(environment);

        if !inventory.exists() {
            return Err(Error::NotFound(format!(
                "Test environment {} not found",
                environment
            )));
        }

        self.apply_playbook(name, &[], true, &[]).await
    }

    fn parse_playbook(&self, content: &str, path: &PathBuf) -> Result<Playbook> {
        let yaml: serde_yaml::Value = serde_yaml::from_str(content)?;
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| Error::Playbook("Invalid playbook filename".into()))?
            .to_string();

        Ok(Playbook {
            name,
            description: yaml[0]["name"]
                .as_str()
                .unwrap_or("No description")
                .to_string(),
            roles: yaml[0]["roles"]
                .as_sequence()
                .map(|seq| {
                    seq.iter()
                        .filter_map(|v| v.as_str())
                        .map(String::from)
                        .collect()
                })
                .unwrap_or_default(),
        })
    }

    fn generate_playbook_content(&self, description: &str, roles: &[String]) -> String {
        let mut content = String::new();
        content.push_str("---\n");
        content.push_str(&format!("- name: {}\n", description));
        content.push_str("  hosts: all\n");
        content.push_str("  become: yes\n\n");

        if !roles.is_empty() {
            content.push_str("  roles:\n");
            for role in roles {
                content.push_str(&format!("    - {}\n", role));
            }
        }

        content
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playbook {
    pub name: String,
    pub description: String,
    pub roles: Vec<String>,
}
