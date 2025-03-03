use crate::error::{Error, Result};
use crate::types::Platform;
use malbox_config::PathConfig;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::{fs, process::Command};
use tracing::{debug, info, warn};

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

/// Manages the building of VM images using Packer
pub struct BuildManager {
    config: PathConfig,
}

impl BuildManager {
    pub fn new(config: PathConfig) -> Self {
        Self { config }
    }

    /// Build a VM image using the provided configuration
    pub async fn build(&self, config: BuildConfig) -> Result<()> {
        // Prepare the build directory
        let build_dir = self.prepare_build_dir(&config).await?;

        // Create the command with proper arguments
        let mut cmd = self.create_packer_command(&config, &build_dir)?;

        // Run the command
        info!("Running packer build command: {:?}", cmd);
        let output = cmd.output().await?;

        if !output.status.success() {
            debug!("Build output: {}", String::from_utf8_lossy(&output.stdout));
            return Err(Error::Packer(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        info!("Successfully built image: {}", config.name);
        Ok(())
    }

    /// Prepare a build directory with all necessary files
    async fn prepare_build_dir(&self, config: &BuildConfig) -> Result<PathBuf> {
        // Use the provided working dir or create a temporary one
        let build_dir = if let Some(dir) = &config.working_dir {
            dir.clone()
        } else {
            let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S");
            let build_dir = self
                .config
                .cache_dir
                .join("builds")
                .join(format!("{}-{}", config.name, timestamp));

            if !build_dir.exists() {
                fs::create_dir_all(&build_dir).await?;
            }

            build_dir
        };

        // Check if the template path exists and is a file or directory
        let template_path = PathBuf::from(&config.template);
        if !template_path.exists() {
            return Err(Error::Template(format!(
                "Template path not found: {:?}",
                template_path
            )));
        }

        // If the template is a directory, we need to copy it entirely
        if template_path.is_dir() {
            copy_directory(&template_path, &build_dir).await?;
        } else {
            // Just copy the template file
            let file_name = template_path
                .file_name()
                .ok_or_else(|| Error::Template("Invalid template path".to_string()))?;

            let target = build_dir.join(file_name);
            fs::copy(&template_path, &target).await?;
        }

        // Check for common directories that might need to be included
        for dir_name in &["scripts", "http", "files", "playbooks"] {
            let source_dir = template_path
                .parent()
                .unwrap_or(&template_path)
                .join(dir_name);

            if source_dir.exists() && source_dir.is_dir() {
                let target_dir = build_dir.join(dir_name);
                if !target_dir.exists() {
                    fs::create_dir_all(&target_dir).await?;
                }

                copy_directory(&source_dir, &target_dir).await?;
            }
        }

        // Create a variables.auto.pkrvars.hcl file with all the variables
        if !config.variables.is_empty() {
            let mut vars_content = String::new();
            for (key, value) in &config.variables {
                // Format the value according to its apparent type
                let formatted_value = if value.starts_with('"') && value.ends_with('"') {
                    // Already a string, leave as is
                    value.clone()
                } else if value == "true" || value == "false" {
                    // Boolean
                    value.clone()
                } else if value.parse::<i64>().is_ok() || value.parse::<f64>().is_ok() {
                    // Number
                    value.clone()
                } else {
                    // String, needs quotes
                    format!("\"{}\"", value.replace('\"', "\\\""))
                };

                vars_content.push_str(&format!("{} = {}\n", key, formatted_value));
            }

            fs::write(build_dir.join("variables.auto.pkrvars.hcl"), vars_content).await?;
        }

        Ok(build_dir)
    }

    /// Create the Packer command with appropriate arguments
    fn create_packer_command(&self, config: &BuildConfig, build_dir: &Path) -> Result<Command> {
        let mut cmd = Command::new("packer");

        // Set the working directory
        cmd.current_dir(build_dir);

        // Basic build command
        cmd.arg("build");

        // Common options
        cmd.arg("-timestamp-ui");
        cmd.arg("-color=true");

        if config.force {
            cmd.arg("-force");
        }

        // Add on-error flag for easier debugging
        cmd.arg("-on-error=ask");

        // Find the template file in the build directory
        let mut template_file = None;
        for entry in std::fs::read_dir(build_dir).map_err(|e| Error::Io(e))? {
            let entry = entry.map_err(|e| Error::Io(e))?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("hcl") {
                // Skip variables files
                if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                    if !file_name.contains("pkrvars") {
                        template_file = Some(path);
                        break;
                    }
                }
            }
        }

        // Add the template file to the command
        match template_file {
            Some(path) => {
                cmd.arg(path.to_string_lossy().to_string());
            }
            None => {
                return Err(Error::Template(
                    "No template file found in build directory".to_string(),
                ));
            }
        }

        Ok(cmd)
    }
}

/// Copy a directory recursively
async fn copy_directory(from: &Path, to: &Path) -> Result<()> {
    if !to.exists() {
        fs::create_dir_all(to).await?;
    }

    let mut entries = fs::read_dir(from).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let file_name = path.file_name().unwrap();
        let target = to.join(file_name);

        if path.is_dir() {
            // Use Box::pin to handle recursion in async function
            Box::pin(copy_directory(&path, &target)).await?;
        } else {
            fs::copy(&path, &target).await?;
        }
    }

    Ok(())
}
