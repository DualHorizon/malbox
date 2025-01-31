use super::types::WorkspaceConfig;
use crate::error::{Error, Result};
use tokio::process::Command;
use tracing::{debug, info};

pub struct WorkspaceManager {
    config: malbox_config::Config,
}

impl WorkspaceManager {
    pub fn new(config: malbox_config::Config) -> Self {
        Self { config }
    }

    pub async fn init(&self, config: &WorkspaceConfig) -> Result<()> {
        let mut cmd = Command::new("terraform");
        cmd.current_dir(&config.working_dir);
        cmd.arg("init");

        for (key, value) in &config.backend_config {
            cmd.arg("-backend-config").arg(format!("{}={}", key, value));
        }

        info!("Running terraform init command");
        let output = cmd.output().await?;

        if !output.status.success() {
            debug!("Init output: {}", String::from_utf8_lossy(&output.stdout));
            return Err(Error::Terraform(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    pub async fn apply(&self, config: &WorkspaceConfig) -> Result<()> {
        self.select_workspace(config).await?;

        let mut cmd = Command::new("terraform");
        cmd.current_dir(&config.working_dir);
        cmd.arg("apply");

        if config.auto_approve {
            cmd.arg("-auto-approve");
        }

        for (key, value) in &config.variables {
            cmd.arg("-var").arg(format!("{}={}", key, value));
        }

        if let Some(target) = &config.target {
            cmd.arg("-target").arg(target);
        }

        info!("Running terraform apply command");
        let output = cmd.output().await?;

        if !output.status.success() {
            debug!("Apply output: {}", String::from_utf8_lossy(&output.stdout));
            return Err(Error::Terraform(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    pub async fn destroy(&self, config: &WorkspaceConfig) -> Result<()> {
        self.select_workspace(config).await?;

        let mut cmd = Command::new("terraform");
        cmd.current_dir(&config.working_dir);
        cmd.arg("destroy");

        if config.auto_approve {
            cmd.arg("-auto-approve");
        }

        for (key, value) in &config.variables {
            cmd.arg("-var").arg(format!("{}={}", key, value));
        }

        info!("Running terraform destroy command");
        let output = cmd.output().await?;

        if !output.status.success() {
            debug!(
                "Destroy output: {}",
                String::from_utf8_lossy(&output.stdout)
            );
            return Err(Error::Terraform(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    pub async fn plan(&self, config: &WorkspaceConfig) -> Result<()> {
        self.select_workspace(config).await?;

        let mut cmd = Command::new("terraform");
        cmd.current_dir(&config.working_dir);
        cmd.arg("plan");

        for (key, value) in &config.variables {
            cmd.arg("-var").arg(format!("{}={}", key, value));
        }

        if let Some(target) = &config.target {
            cmd.arg("-target").arg(target);
        }

        info!("Running terraform plan command");
        let output = cmd.output().await?;

        if !output.status.success() {
            debug!("Plan output: {}", String::from_utf8_lossy(&output.stdout));
            return Err(Error::Terraform(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    async fn select_workspace(&self, config: &WorkspaceConfig) -> Result<()> {
        let mut cmd = Command::new("terraform");
        cmd.current_dir(&config.working_dir);
        cmd.arg("workspace").arg("select").arg(&config.workspace);

        let output = cmd.output().await?;
        if !output.status.success() {
            return Err(Error::Terraform(format!(
                "Failed to select workspace: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }
}
