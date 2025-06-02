use crate::error::{Result, TerraformError};
use malbox_io_utils::process::AsyncCommand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub name: String,
    pub working_dir: PathBuf,
    pub workspace: String,
    pub variables: HashMap<String, String>,
    pub backend_config: HashMap<String, String>,
    pub target: Option<String>,
    pub auto_approve: bool,
}

pub struct WorkspaceManager {
    config: malbox_config::Config,
}

impl WorkspaceManager {
    pub fn new(config: malbox_config::Config) -> Self {
        Self { config }
    }

    pub async fn init(&self, config: &WorkspaceConfig) -> Result<()> {
        let mut cmd = AsyncCommand::new("terraform")
            .current_dir(&config.working_dir)
            .arg("init");

        for (key, value) in &config.backend_config {
            cmd = cmd.arg("-backend-config").arg(format!("{}={}", key, value));
        }

        info!("Running terraform init command");
        let output = cmd.run().await.map_err(|e| TerraformError::CommandFailed {
            command: "terraform init".to_string(),
            source: e,
        })?;

        if !output.success() {
            debug!("Init output: {}", output.stdout());
            return Err(TerraformError::CommandExitCode {
                command: "terraform init".to_string(),
                exit_code: output.exit_code,
                stderr: output.stderr(),
            });
        }

        Ok(())
    }

    pub async fn apply(&self, config: &WorkspaceConfig) -> Result<()> {
        self.select_workspace(config).await?;

        let mut cmd = AsyncCommand::new("terraform")
            .current_dir(&config.working_dir)
            .arg("apply");

        if config.auto_approve {
            cmd = cmd.arg("-auto-approve");
        }

        for (key, value) in &config.variables {
            cmd = cmd.arg("-var").arg(format!("{}={}", key, value));
        }

        if let Some(target) = &config.target {
            cmd = cmd.arg("-target").arg(target);
        }

        info!("Running terraform apply command");
        let output = cmd.run().await.map_err(|e| TerraformError::CommandFailed {
            command: "terraform apply".to_string(),
            source: e,
        })?;

        if !output.success() {
            debug!("Apply output: {}", output.stdout());
            return Err(TerraformError::CommandExitCode {
                command: "terraform apply".to_string(),
                exit_code: output.exit_code,
                stderr: output.stderr(),
            });
        }

        Ok(())
    }

    pub async fn destroy(&self, config: &WorkspaceConfig) -> Result<()> {
        self.select_workspace(config).await?;

        let mut cmd = AsyncCommand::new("terraform")
            .current_dir(&config.working_dir)
            .arg("destroy");

        if config.auto_approve {
            cmd = cmd.arg("-auto-approve");
        }

        for (key, value) in &config.variables {
            cmd = cmd.arg("-var").arg(format!("{}={}", key, value));
        }

        info!("Running terraform destroy command");
        let output = cmd.run().await.map_err(|e| TerraformError::CommandFailed {
            command: "terraform destroy".to_string(),
            source: e,
        })?;

        if !output.success() {
            debug!("Destroy output: {}", output.stdout());
            return Err(TerraformError::CommandExitCode {
                command: "terraform destroy".to_string(),
                exit_code: output.exit_code,
                stderr: output.stderr(),
            });
        }

        Ok(())
    }

    pub async fn plan(&self, config: &WorkspaceConfig) -> Result<()> {
        self.select_workspace(config).await?;

        let mut cmd = AsyncCommand::new("terraform")
            .current_dir(&config.working_dir)
            .arg("plan");

        for (key, value) in &config.variables {
            cmd = cmd.arg("-var").arg(format!("{}={}", key, value));
        }

        if let Some(target) = &config.target {
            cmd = cmd.arg("-target").arg(target);
        }

        info!("Running terraform plan command");
        let output = cmd.run().await.map_err(|e| TerraformError::CommandFailed {
            command: "terraform plan".to_string(),
            source: e,
        })?;

        if !output.success() {
            debug!("Plan output: {}", output.stdout());
            return Err(TerraformError::CommandExitCode {
                command: "terraform plan".to_string(),
                exit_code: output.exit_code,
                stderr: output.stderr(),
            });
        }

        Ok(())
    }

    async fn select_workspace(&self, config: &WorkspaceConfig) -> Result<()> {
        let output = AsyncCommand::new("terraform")
            .current_dir(&config.working_dir)
            .arg("workspace")
            .arg("select")
            .arg(&config.workspace)
            .run()
            .await
            .map_err(|e| TerraformError::CommandFailed {
                command: "terraform workspace select".to_string(),
                source: e,
            })?;

        if !output.success() {
            return Err(TerraformError::CommandExitCode {
                command: "terraform workspace select".to_string(),
                exit_code: output.exit_code,
                stderr: output.stderr(),
            });
        }

        Ok(())
    }
}
