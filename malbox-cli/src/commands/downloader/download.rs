use crate::{
    commands::Command,
    error::{CliError, Result},
};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Select};
use malbox_config::Config;
use malbox_downloader::{Downloader, SourceRegistry, SourceVariant};
use std::path::PathBuf;

#[derive(Parser)]
pub struct DownloadArgs {
    #[arg(short, long)]
    /// Name to search for (family, edition, version or variant ID)
    pub name: Option<String>,
    #[arg(short, long)]
    /// Specific version to use
    pub version: Option<String>,
    #[arg(short, long)]
    /// Family to search in (e.g., 'windows', 'linux')
    pub family: Option<String>,
    #[arg(short, long)]
    /// Edition to search for (e.g., 'windows', 'ubuntu')
    pub edition: Option<String>,
    #[arg(short = 'r', long)]
    /// Variant ID to search for
    pub variant: Option<String>,
    #[arg(short, long)]
    /// Direct URL to download from
    pub url: Option<String>,
    #[arg(short, long)]
    /// Output file path
    pub output: Option<PathBuf>,
    #[arg(long, default_value = "false")]
    /// Disable interactive prompts
    pub non_interactive: bool,
}

impl Command for DownloadArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        let registry_path = config.paths.download_dir.join("source_registry.json");
        let downloader = Downloader::builder().show_progress(true).build();
        let registry = SourceRegistry::load(registry_path).await?;

        match (
            &self.url,
            &self.name,
            self.family.as_ref(),
            self.edition.as_ref(),
            self.version.as_ref(),
            self.variant.as_ref(),
        ) {
            (Some(url), None, _, _, _, _) => {
                let output_path = downloader
                    .download(url, None, &config.paths.download_dir, self.output)
                    .await?;
                println!("\nDownload saved to: {}", output_path.display());
            }

            (None, _, Some(family), Some(edition), Some(version), Some(variant)) => {
                let source = registry.get_source(
                    Some(family.as_str()),
                    Some(edition.as_str()),
                    Some(version.as_str()),
                    Some(variant.as_str()),
                )?;

                let output_path = downloader
                    .download(
                        &source.url,
                        Some(&source),
                        &config.paths.download_dir,
                        self.output,
                    )
                    .await?;
                println!("\nDownload saved to: {}", output_path.display());
            }

            (None, Some(name), family, edition, version, variant) => {
                let variant_id = if family.is_some() && edition.is_some() && version.is_some() {
                    name.as_str()
                } else {
                    variant.map_or(name.as_str(), |v| v.as_str())
                };

                let source = registry.get_source(
                    family.map(|f| f.as_str()),
                    edition.map(|e| e.as_str()),
                    version.map(|v| v.as_str()),
                    Some(variant_id),
                )?;

                let output_path = downloader
                    .download(
                        &source.url,
                        Some(&source),
                        &config.paths.download_dir,
                        self.output,
                    )
                    .await?;
                println!("\nDownload saved to: {}", output_path.display());
            }

            _ if !self.non_interactive => {
                let source = select_source_interactively(
                    &registry,
                    self.family.as_deref(),
                    self.edition.as_deref(),
                    self.version.as_deref(),
                    self.variant.as_deref(),
                )?;

                let output_path = downloader
                    .download(
                        &source.url,
                        Some(&source),
                        &config.paths.download_dir,
                        self.output,
                    )
                    .await?;
                println!("\nDownload saved to: {}", output_path.display());
            }

            _ => {
                return Err(CliError::InvalidArgument(
                    "Either --url, --name, or a complete set of source components must be provided in non-interactive mode".to_string(),
                ));
            }
        }

        Ok(())
    }
}

fn select_source_interactively(
    registry: &SourceRegistry,
    cli_family: Option<&str>,
    cli_edition: Option<&str>,
    cli_version: Option<&str>,
    cli_variant: Option<&str>,
) -> Result<SourceVariant> {
    let theme = ColorfulTheme::default();

    let families = registry.list_families();
    if families.is_empty() {
        return Err(CliError::InvalidArgument(
            "No families found in registry".to_string(),
        ));
    }

    let selected_family_id = if let Some(family_id) = cli_family {
        if !families.iter().any(|f| f.id == family_id) {
            return Err(CliError::InvalidArgument(format!(
                "Family '{}' not found",
                family_id
            )));
        }
        family_id.to_string()
    } else {
        let family_items: Vec<String> = families.iter().map(|f| f.name.clone()).collect();
        let family_idx = Select::with_theme(&theme)
            .with_prompt("Select a family")
            .default(0)
            .items(&family_items)
            .interact()?;

        families[family_idx].id.clone()
    };

    let editions = registry.list_editions(&selected_family_id)?;

    let selected_edition_id = if let Some(edition_id) = cli_edition {
        if !editions.iter().any(|e| e.id == edition_id) {
            return Err(CliError::InvalidArgument(format!(
                "Edition '{}' not found in family '{}'",
                edition_id, selected_family_id
            )));
        }
        edition_id.to_string()
    } else {
        let edition_items: Vec<String> = editions.iter().map(|e| e.name.clone()).collect();
        let edition_idx = Select::with_theme(&theme)
            .with_prompt("Select an edition")
            .default(0)
            .items(&edition_items)
            .interact()?;

        editions[edition_idx].id.clone()
    };

    let releases = registry.list_releases(&selected_family_id, &selected_edition_id)?;

    let selected_release_version = if let Some(version) = cli_version {
        if !releases.iter().any(|r| r.version == version) {
            return Err(CliError::InvalidArgument(format!(
                "Version '{}' not found in family '{}', edition '{}'",
                version, selected_family_id, selected_edition_id
            )));
        }
        version.to_string()
    } else {
        let release_items: Vec<String> = releases
            .iter()
            .map(|r| format!("{} ({})", r.version, r.description))
            .collect();
        let release_idx = Select::with_theme(&theme)
            .with_prompt("Select a version")
            .default(0)
            .items(&release_items)
            .interact()?;

        releases[release_idx].version.clone()
    };

    let variants = registry.list_variants(
        &selected_family_id,
        &selected_edition_id,
        &selected_release_version,
    )?;

    let selected_variant = if let Some(variant_id) = cli_variant {
        match variants.iter().find(|v| v.id == variant_id) {
            Some(variant) => variant,
            None => {
                return Err(CliError::InvalidArgument(format!(
                    "Variant '{}' not found in family '{}', edition '{}', version '{}'",
                    variant_id, selected_family_id, selected_edition_id, selected_release_version
                )))
            }
        }
    } else {
        let variant_items: Vec<String> = variants
            .iter()
            .map(|v| format!("{} - {}", v.id, v.description))
            .collect();

        let variant_idx = Select::with_theme(&theme)
            .with_prompt("Select a variant")
            .default(0)
            .items(&variant_items)
            .interact()?;

        variants[variant_idx]
    };

    Ok(selected_variant.clone())
}
