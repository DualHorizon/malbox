use super::types::WorkspaceConfig;
use crate::{Error, Result};
use tokio::process::Command;
use tracing::{debug, info};

pub struct StateManager {
    config: malbox_config::Config,
}

impl StateManager {
    pub fn new(config: malbox_config::Config) -> Self {
        Self { config }
    }

    pub async fn import(&self, config: &WorkspaceConfig, address: &str, id: &str) -> Result<()> {
        let mut cmd = Command::new("terraform");
        cmd.current_dir(&config.working_dir);
        cmd.arg("import");

        for (key, value) in &config.backend_config {
            cmd.arg("-backend-config").arg(format!("{}={}", key, value));
        }

        for (key, value) in &config.variables {
            cmd.arg("-var").arg(format!("{}={}", key, value));
        }

        cmd.arg(address).arg(id);

        info!("Running terraform import command");
        let output = cmd.output().await?;

        if !output.status.success() {
            debug!("Import output: {}", String::from_utf8_lossy(&output.stdout));
            return Err(Error::Terraform(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    pub async fn show(&self, config: &WorkspaceConfig) -> Result<String> {
        let mut cmd = Command::new("terraform");
        cmd.current_dir(&config.working_dir);
        cmd.arg("show");

        let output = cmd.output().await?;

        if !output.status.success() {
            return Err(Error::Terraform(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
