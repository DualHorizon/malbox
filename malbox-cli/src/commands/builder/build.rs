use crate::{
    commands::Command,
    error::Result,
    types::PlatformType,
    utils::{interaction::templates::TemplatePrompt, progress::Progress},
};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Confirm};
use malbox_config::Config;
use malbox_downloader::{DownloadRegistry, Downloader};
use malbox_infra::packer::{
    build::{BuildConfig, BuildManager},
    templates::TemplateManager,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Parser)]
pub struct BuildArgs {
    #[arg(value_enum)]
    pub platform: PlatformType,
    #[arg(short, long)]
    pub name: String,
    /// Specify source to use (format: name-version)
    #[arg(long)]
    pub source: Option<String>,
    /// Manual ISO path (overrides source if both specified)
    #[arg(long)]
    pub iso: Option<String>,
    #[arg(short, long)]
    pub force: bool,
    #[arg(short, long)]
    pub working_dir: Option<PathBuf>,
    #[arg(short, long = "var", value_parser = crate::utils::validation::parse_key_val)]
    pub variables: Vec<(String, String)>,
    /// Force download of source even if a local path exists
    #[arg(long)]
    pub force_download: bool,
}

impl Command for BuildArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        let path = PathBuf::from(
            "/home/shard/.config/malbox/infrastructure/packer/templates/windows/base.pkr.hcl",
        );
        let template_manager = TemplateManager::new();
        let template = template_manager.load(path).await?;

        let mut variables: HashMap<String, String> = self.variables.into_iter().collect();

        if let Some(source_ref) = self.source {
            let (name, version) = source_ref.split_once('-').ok_or_else(|| {
                crate::error::CliError::InvalidArgument(
                    "Source must be in format: name-version".to_string(),
                )
            })?;

            let registry_path = config.paths.download_dir.join("download_registry.json");
            let registry = DownloadRegistry::load(registry_path.clone()).await?;

            let source = registry.get_source(&source_ref, Some(version))?;

            let needs_download = if let Some(local_path) = &source.metadata.local_path {
                let path = Path::new(local_path);
                self.force_download || !path.exists()
            } else {
                true
            };

            if needs_download {
                let theme = ColorfulTheme::default();
                let message = if self.force_download {
                    format!(
                        "Source ISO for {} will be redownloaded. Continue?",
                        source_ref
                    )
                } else {
                    format!(
                        "Source ISO for {} is not available locally. Download it now?",
                        source_ref
                    )
                };

                let confirm = Confirm::with_theme(&theme)
                    .with_prompt(message)
                    .default(true)
                    .interact()?;

                if !confirm {
                    return Err(crate::error::CliError::CommandFailed(
                        "Download cancelled by user".to_string(),
                    ));
                }

                let downloader = Downloader::builder().show_progress(true).build();

                println!("Downloading source: {}", source_ref);
                let iso_path = downloader
                    .download(&source.url, Some(&source), &config.paths.download_dir, None)
                    .await?;

                variables.insert(
                    "iso_url".to_string(),
                    iso_path.to_string_lossy().to_string(),
                );

                let registry = DownloadRegistry::load(registry_path).await?;
                if let Ok(updated_source) = registry.get_source(name, Some(version)) {
                    if let Some(checksum) = updated_source.checksum {
                        variables
                            .insert("iso_checksum".to_string(), format!("sha256:{}", checksum));
                    }

                    if updated_source.url.starts_with("http") {
                        variables.insert("iso_url".to_string(), updated_source.url.clone());
                    }
                }
            } else {
                let local_path = source.metadata.local_path.unwrap();
                variables.insert("iso_url".to_string(), local_path);

                if let Some(checksum) = source.checksum {
                    variables.insert("iso_checksum".to_string(), format!("sha256:{}", checksum));
                }

                if source.url.starts_with("http") {
                    variables.insert("iso_url".to_string(), source.url.clone());
                }
            }
        } else if let Some(iso) = self.iso.clone() {
            variables.insert("iso_url".to_string(), iso);
        }

        let template_prompt = TemplatePrompt::default();
        template_prompt.display_template_info(&template)?;
        template_prompt
            .prompt_variables(&template, &mut variables)
            .await?;

        let build_config = BuildConfig {
            platform: self.platform.into(),
            name: self.name.clone(),
            force: self.force,
            working_dir: self.working_dir,
            iso: self.iso,
            template: config.paths.packer_dir.to_str().unwrap().to_string(),
            variables,
        };

        let builder = BuildManager::new(config.paths.clone());

        Progress::new()
            .run("Building base image...", async {
                builder
                    .build(build_config)
                    .await
                    .map_err(|e| crate::error::CliError::Infrastructure(e))
            })
            .await
    }
}
