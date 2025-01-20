use crate::{Error, Result};
use super::types::{BuilderConfig, Platform, RefineConfig};
use malbox_config::Config;
use std::{collections::HashMap, path::PathBuf};
use tokio::process::Command;
use tracing::info;

pub struct Builder {
    config: malbox_config::Config,
}

impl Builder {
    pub fn new(config: malbox_config::Config) -> Self {
        Self { config }
    }

    pub async fn init(&self, working_dir: Option<PathBuf>, force: bool) -> Result<()> {
        let dir = working_dir.unwrap_or_else(|| self.config.paths.config_dir.clone());

        if !force && dir.exists() {
            return Err(Error::Config(format!(
                "Directory {} already exists. Use --force to overwrite",
                dir.display()
            )));
        }

        let mut cmd = tokio::process::Command::new("packer");
        cmd.current_dir(&dir)
            .arg("init")
            .arg("templates/packer_plugins.pkr.hcl");

        let output = cmd.output().await?;
        if !output.status.success() {
            return Err(Error::Build(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    pub async fn build(&self, config: BuilderConfig) -> Result<()> {
        let template_path = self.get_template_path(&config.platform);

        let mut cmd = tokio::process::Command::new("packer");
        cmd.arg("build").arg("-timestamp-ui");

        if config.force {
            cmd.arg("-force");
        }

        for (key, value) in config.variables {
            cmd.arg("-var").arg(format!("{}={}", key, value));
        }

        if let Some(iso) = config.iso {
            cmd.arg("-var").arg(format!("iso_path={}", iso));
        }

        cmd.arg(&template_path);

        if let Some(dir) = config.working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd.output().await?;
        if !output.status.success() {
            return Err(Error::Build(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    pub async fn refine(&self, config: RefineConfig) -> Result<()> {
        let base_image = format!("source.{}.{}", self.get_source_type(), config.base);
        let mut cmd = tokio::process::Command::new("packer");

        cmd.arg("build").arg("-timestamp-ui");

        if config.force {
            cmd.arg("-force");
        }

        cmd.arg("-var").arg(format!("source_image={}", base_image));

        for (key, value) in config.variables {
            cmd.arg("-var").arg(format!("{}={}", key, value));
        }

        cmd.arg(format!(
            "templates/{}/refinements/{}.pkr.hcl",
            config.playbook, config.name
        ));

        let output = cmd.output().await?;
        if !output.status.success() {
            return Err(Error::Build(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    fn get_template_path(&self, platform: &Platform) -> PathBuf {
        let platform_str = match platform {
            Platform::Windows => "windows",
            Platform::Linux => "linux",
        };

        self.config
            .paths
            .config_dir
            .join("templates")
            .join(platform_str)
            .join("base.pkr.hcl")
    }

    fn get_source_type(&self) -> &'static str {
        // This would be determined by configuration or environment
        "vsphere-iso"
    }
}
