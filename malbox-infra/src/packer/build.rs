use crate::error::{Error, Result};
use crate::types::Platform;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub platform: Platform,
    pub name: String,
    pub template: String,
    pub iso: Option<String>,
    pub force: bool,
    pub working_dir: Option<PathBuf>,
    pub variables: HashMap<String, String>,
}

pub struct BuildManager {
    config: malbox_config::Paths,
}

impl BuildManager {
    pub fn new(config: malbox_config::Paths) -> Self {
        Self { config }
    }

    pub async fn build(&self, config: BuildConfig) -> Result<()> {
        let mut cmd = Command::new("packer");
        cmd.arg("build").arg("-timestamp-ui");

        if config.force {
            cmd.arg("-force");
        }

        for (key, value) in &config.variables {
            cmd.arg("-var").arg(format!("{}={}", key, value));
        }

        if let Some(iso) = &config.iso {
            cmd.arg("-var").arg(format!("iso_path={}", iso));
        }

        let template_path = self.get_template_path(&config.template, &config.platform);
        cmd.arg(template_path);

        if let Some(dir) = &config.working_dir {
            cmd.current_dir(dir);
        }

        info!("Running packer build command");
        let output = cmd.output().await?;

        if !output.status.success() {
            debug!("Build output: {}", String::from_utf8_lossy(&output.stdout));
            return Err(Error::Packer(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    fn get_template_path(&self, name: &str, platform: &Platform) -> PathBuf {
        let platform_str = match platform {
            Platform::Windows => "windows",
            Platform::Linux => "linux",
        };

        PathBuf::from(&self.config.templates_dir)
            .join("templates")
            .join(platform_str)
            .join(format!("{}.pkr.hcl", name))
    }
}
