use crate::{
    commands::Command,
    error::{CliError, Result},
    types::PlatformType,
    utils::{
        interaction::templates::TemplatePrompt, progress::Progress, validation::parse_key_val,
    },
};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Select};
use malbox_config::Config;
use malbox_downloader::{Downloader, SourceRegistry, SourceVariant};
use malbox_infra::packer::{
    build::{BuildConfig, BuildManager},
    templates::{Template, TemplateManager},
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Parser)]
pub struct BuildArgs {
    /// Platform type
    #[arg(short, long)]
    pub platform: Option<PlatformType>,
    /// Template name to use for building
    #[arg(short, long)]
    pub template_name: Option<String>,
    /// Direct path to template file
    #[arg(long)]
    pub template_path: Option<PathBuf>,
    /// Name for the built image
    #[arg(short, long)]
    pub output_name: Option<String>,
    /// OS family (e.g., windows, ubuntu)
    #[arg(long)]
    pub family: Option<String>,
    /// OS edition (e.g., server, desktop)
    #[arg(long)]
    pub edition: Option<String>,
    /// OS version (e.g., 10, 22.04)
    #[arg(long)]
    pub version: Option<String>,
    /// Source variant identifier
    #[arg(long)]
    pub variant: Option<String>,
    /// Direct path to ISO file
    #[arg(long)]
    pub iso: Option<String>,
    /// Force overwrite of existing images
    #[arg(short, long)]
    pub force: bool,
    /// Working directory for build operations
    #[arg(short, long)]
    pub working_dir: Option<PathBuf>,
    /// Template variables in KEY=VALUE format
    #[arg(short, long = "var", value_parser = parse_key_val)]
    pub variables: Vec<(String, String)>,
    /// Force download even if ISO already exists locally
    #[arg(long)]
    pub force_download: bool,
    /// Run without interactive prompts
    #[arg(long, default_value = "false")]
    pub non_interactive: bool,
}

impl Command for BuildArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        let BuildArgs {
            platform: platform_opt,
            template_name: template_name_opt,
            template_path: template_path_opt,
            output_name: output_name_opt,
            family: family_opt,
            edition: edition_opt,
            version: version_opt,
            variant: variant_opt,
            iso: iso_opt,
            force,
            working_dir: working_dir_opt,
            variables: vars,
            force_download,
            non_interactive,
        } = self;

        let platform = match platform_opt {
            Some(platform) => platform,
            None => {
                if non_interactive {
                    return Err(CliError::InvalidArgument(
                        "Platform must be specified in non-interactive mode".to_string(),
                    ));
                }

                let platforms = vec!["Windows", "Linux"];
                let platform_idx = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select platform")
                    .default(0)
                    .items(&platforms)
                    .interact()?;

                match platforms[platform_idx] {
                    "Windows" => PlatformType::Windows,
                    "Linux" => PlatformType::Linux,
                    _ => unreachable!(),
                }
            }
        };

        let template_manager = TemplateManager::new();
        let template = if let Some(path) = template_path_opt {
            template_manager.load(path).await?
        } else if let Some(name) = template_name_opt {
            let template_path = find_template_by_name(config, &name, &platform).await?;
            template_manager.load(template_path).await?
        } else {
            if non_interactive {
                return Err(CliError::InvalidArgument(
                    "Either template_name or template_path must be specified in non-interactive mode".to_string(),
                ));
            }

            let template_path = discover_and_select_template(config, &platform).await?;
            template_manager.load(template_path).await?
        };

        let mut variables: HashMap<String, String> = vars.into_iter().collect();

        let output_name = output_name_opt.unwrap_or_else(|| match platform {
            PlatformType::Windows => format!("windows-{}", chrono::Local::now().format("%Y%m%d")),
            PlatformType::Linux => format!("linux-{}", chrono::Local::now().format("%Y%m%d")),
        });

        if let Some(iso_path) = iso_opt.clone() {
            variables.insert("iso_url".to_string(), iso_path.clone());
        } else {
            let has_source_components = family_opt.is_some()
                || edition_opt.is_some()
                || version_opt.is_some()
                || variant_opt.is_some();

            let registry_path = config.paths.download_dir.join("download_registry.json");
            let registry = SourceRegistry::load(registry_path).await?;

            let source = if has_source_components {
                registry.get_source(
                    family_opt.as_deref(),
                    edition_opt.as_deref(),
                    version_opt.as_deref(),
                    variant_opt.as_deref(),
                )?
            } else if !non_interactive {
                select_source_interactively(&registry)?
            } else {
                return Err(CliError::InvalidArgument(
                    "Either source components, --iso option, or interactive mode must be used"
                        .to_string(),
                ));
            };

            if let Some(local_path) = &source.metadata.local_path {
                let path = Path::new(local_path);

                if path.exists() && !self.force_download {
                    variables.insert("iso_url".to_string(), local_path.clone());

                    if let Some(checksum) = &source.checksum {
                        variables
                            .insert("iso_checksum".to_string(), format!("sha256:{}", checksum));
                    }
                } else {
                    download_and_use_source(
                        &source,
                        config,
                        &mut variables,
                        force_download,
                        non_interactive,
                    )
                    .await?;
                }
            } else {
                download_and_use_source(
                    &source,
                    config,
                    &mut variables,
                    force_download,
                    non_interactive,
                )
                .await?;
            }
        }

        let template_prompt = TemplatePrompt::default();
        template_prompt.display_template_info(&template)?;

        if !non_interactive {
            template_prompt
                .prompt_variables(&template, &mut variables)
                .await?;
        } else {
            let missing = template.get_missing_variables(&variables)?;
            if !missing.is_empty() {
                return Err(CliError::InvalidArgument(format!(
                    "Required variables missing in non-interactive mode: {}",
                    missing.join(", ")
                )));
            }
        }

        let build_config = BuildConfig {
            platform: platform.into(),
            name: output_name,
            force,
            working_dir: working_dir_opt,
            iso: iso_opt,
            template: config
                .paths
                .packer_dir
                .to_str()
                .unwrap_or_default()
                .to_string(),
            variables,
        };

        let builder = BuildManager::new(config.paths.clone());
        Progress::new()
            .run("Building image...", async {
                builder
                    .build(build_config)
                    .await
                    .map_err(|e| CliError::Infrastructure(e))
            })
            .await
    }
}

async fn find_template_by_name(
    config: &Config,
    name: &str,
    platform: &PlatformType,
) -> Result<PathBuf> {
    let platform_str = match platform {
        PlatformType::Windows => "windows",
        PlatformType::Linux => "linux",
    };

    let template_dir = config.paths.packer_dir.join("templates").join(platform_str);

    let direct_path = template_dir.join(format!("{}.pkr.hcl", name));
    if direct_path.exists() {
        return Ok(direct_path);
    }

    let mut entries = fs::read_dir(&template_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            let subdir_path = entry.path().join(format!("{}.pkr.hcl", name));
            if subdir_path.exists() {
                return Ok(subdir_path);
            }
        }
    }

    Err(CliError::InvalidArgument(format!(
        "Template '{}' not found for platform '{}'",
        name, platform_str
    )))
}

async fn discover_and_select_template(config: &Config, platform: &PlatformType) -> Result<PathBuf> {
    let platform_str = match platform {
        PlatformType::Windows => "windows",
        PlatformType::Linux => "linux",
    };

    let template_dir = config.paths.packer_dir.join("templates").join(platform_str);
    let mut templates = Vec::new();

    find_templates_in_dir(&template_dir, &mut templates).await?;

    if templates.is_empty() {
        return Err(CliError::InvalidArgument(format!(
            "No templates found for platform '{}'",
            platform_str
        )));
    }

    let template_names: Vec<String> = templates
        .iter()
        .map(|path| {
            path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        })
        .collect();

    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select template")
        .default(0)
        .items(&template_names)
        .interact()?;

    Ok(templates[selection].clone())
}

// NOTE: Should we move this into the template manager directly?
// Probably.
async fn find_templates_in_dir(dir: &PathBuf, templates: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    let mut entries = fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            Box::pin(find_templates_in_dir(&path, templates)).await?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("hcl") {
            if let Ok(content) = fs::read_to_string(&path).await {
                if content.contains("source")
                    && (content.contains("build {") || content.contains("build{"))
                {
                    templates.push(path);
                }
            }
        }
    }

    Ok(())
}

// NOTE: we probably can make a shared function/method for this, since we use
// similar logic in other commands, such as `malbox downloader download`
async fn download_and_use_source(
    source: &SourceVariant,
    config: &Config,
    variables: &mut HashMap<String, String>,
    force_download: bool,
    non_interactive: bool,
) -> Result<()> {
    let theme = ColorfulTheme::default();
    let message = if force_download {
        "Source will be redownloaded. Continue?"
    } else {
        "Source needs to be downloaded. Continue?"
    };

    let should_download = if non_interactive {
        true
    } else {
        Confirm::with_theme(&theme)
            .with_prompt(message)
            .default(true)
            .interact()?
    };

    if !should_download {
        return Err(CliError::CommandFailed(
            "Download cancelled by user".to_string(),
        ));
    }

    let downloader = Downloader::builder().show_progress(true).build();
    println!("Downloading source...");

    let iso_path = downloader
        .download(&source.url, Some(source), &config.paths.download_dir, None)
        .await?;

    variables.insert(
        "iso_url".to_string(),
        iso_path.to_string_lossy().to_string(),
    );

    if let Some(checksum) = &source.checksum {
        variables.insert("iso_checksum".to_string(), format!("sha256:{}", checksum));
    }

    Ok(())
}

// NOTE: should be moved somewhere else and imported since we use similar logic in other commands.
fn select_source_interactively(registry: &SourceRegistry) -> Result<SourceVariant> {
    let theme = ColorfulTheme::default();

    let families = registry.list_families();
    if families.is_empty() {
        return Err(CliError::InvalidArgument(
            "No families found in registry".to_string(),
        ));
    }

    let family_items: Vec<String> = families
        .iter()
        .map(|f| format!("{} - {}", f.id, f.description))
        .collect();

    let family_idx = Select::with_theme(&theme)
        .with_prompt("Select a family")
        .default(0)
        .items(&family_items)
        .interact()?;

    let selected_family_id = &families[family_idx].id;

    let editions = registry.list_editions(selected_family_id)?;
    let edition_items: Vec<String> = editions
        .iter()
        .map(|e| format!("{} - {}", e.id, e.description))
        .collect();

    let edition_idx = Select::with_theme(&theme)
        .with_prompt("Select an edition")
        .default(0)
        .items(&edition_items)
        .interact()?;

    let selected_edition_id = &editions[edition_idx].id;

    let releases = registry.list_releases(selected_family_id, selected_edition_id)?;
    let release_items: Vec<String> = releases
        .iter()
        .map(|r| format!("{} - {}", r.version, r.description))
        .collect();

    let release_idx = Select::with_theme(&theme)
        .with_prompt("Select a version")
        .default(0)
        .items(&release_items)
        .interact()?;

    let selected_release_version = &releases[release_idx].version;

    let variants = registry.list_variants(
        selected_family_id,
        selected_edition_id,
        selected_release_version,
    )?;

    let variant_items: Vec<String> = variants
        .iter()
        .map(|v| {
            let arch_str = match v.architecture {
                malbox_downloader::Architecture::X86 => "x86",
                malbox_downloader::Architecture::X86_64 => "x86-64",
                malbox_downloader::Architecture::Arm64 => "arm64",
            };
            format!("{} - {} ({})", v.id, v.description, arch_str)
        })
        .collect();

    let variant_idx = Select::with_theme(&theme)
        .with_prompt("Select a variant")
        .default(0)
        .items(&variant_items)
        .interact()?;

    Ok(variants[variant_idx].clone())
}
