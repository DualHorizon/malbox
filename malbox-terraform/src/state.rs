use crate::error::{Result, TerraformError};
use crate::workspace::WorkspaceConfig;
use malbox_io_utils::process::AsyncCommand;
use tracing::{debug, info};

pub struct StateManager {
    config: malbox_config::Config,
}

impl StateManager {
    pub fn new(config: malbox_config::Config) -> Self {
        Self { config }
    }

    pub async fn import(&self, config: &WorkspaceConfig, address: &str, id: &str) -> Result<()> {
        let mut cmd = AsyncCommand::new("terraform")
            .current_dir(&config.working_dir)
            .arg("import");

        // Add backend config
        for (key, value) in &config.backend_config {
            cmd = cmd.arg("-backend-config").arg(format!("{}={}", key, value));
        }

        // Add variables
        for (key, value) in &config.variables {
            cmd = cmd.arg("-var").arg(format!("{}={}", key, value));
        }

        // Add target resource and ID
        cmd = cmd.arg(address).arg(id);

        info!("Running terraform import: {} {}", address, id);

        let output = cmd.run().await.map_err(|e| TerraformError::CommandFailed {
            command: "terraform import".to_string(),
            source: e,
        })?;

        if !output.success() {
            debug!("Import stdout: {}", output.stdout());
            return Err(TerraformError::CommandExitCode {
                command: format!("terraform import {} {}", address, id),
                exit_code: output.exit_code,
                stderr: output.stderr(),
            });
        }

        debug!("Import completed successfully");
        Ok(())
    }

    pub async fn show(&self, config: &WorkspaceConfig) -> Result<String> {
        let output = AsyncCommand::new("terraform")
            .current_dir(&config.working_dir)
            .arg("show")
            .run()
            .await
            .map_err(|e| TerraformError::CommandFailed {
                command: "terraform show".to_string(),
                source: e,
            })?;

        if !output.success() {
            return Err(TerraformError::CommandExitCode {
                command: "terraform show".to_string(),
                exit_code: output.exit_code,
                stderr: output.stderr(),
            });
        }

        Ok(output.stdout())
    }
}
