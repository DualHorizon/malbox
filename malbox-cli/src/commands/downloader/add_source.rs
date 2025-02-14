use crate::{
    commands::Command,
    error::{CliError, Result},
    utils::progress::Progress,
};
use clap::Parser;
use dialoguer::Confirm;
use malbox_config::Config;
use malbox_downloader::{
    registry::{
        Architecture, Platform, ProcessingStatus, SourceMetadata, SourceType, SystemRequirements,
    },
    DownloadRegistry, DownloadSource,
};
use time::OffsetDateTime;

#[derive(Parser)]
pub struct AddSourceArgs {
    #[arg(short, long)]
    pub name: String,
    #[arg(short, long)]
    pub version: String,
    #[arg(short, long)]
    pub description: String,
    #[arg(short, long)]
    pub url: String,
    #[arg(value_enum, short, long, default_value = "iso")]
    pub source_type: SourceType,
    #[arg(value_enum, short = 'p', long, default_value = "linux")]
    pub platform: Platform,
    #[arg(value_enum, short = 'a', long, default_value = "x86-64")]
    pub architecture: Architecture,
    #[arg(short, long)]
    pub checksum: Option<String>,
    #[arg(long = "checksum-type")]
    pub checksum_type: Option<String>,
    #[arg(long)]
    pub min_cpu_cores: Option<u32>,
    #[arg(long)]
    pub min_memory_mb: Option<u64>,
    #[arg(long)]
    pub min_disk_gb: Option<u64>,
    #[arg(short, long)]
    pub tags: Option<Vec<String>>,
    #[arg(short, long)]
    pub mirrors: Option<Vec<String>>,
    #[arg(long)]
    pub license: Option<String>,
    #[arg(long)]
    pub documentation_url: Option<String>,
    #[arg(long)]
    pub release_notes: Option<String>,
    #[arg(long)]
    pub parent_source: Option<String>,
    #[arg(value_enum, long, default_value = "raw")]
    pub processing_status: ProcessingStatus,
}

impl Command for AddSourceArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        let registry_path = config.paths.download_dir.join("download_registry.json");
        let mut registry = DownloadRegistry::load(registry_path.clone()).await?;

        let source_key = format!("{}-{}", self.name, self.version);
        if registry.custom_sources.contains_key(&source_key) {
            let confirm = Confirm::new()
                .with_prompt(format!(
                    "Source '{}' version '{}' already exists. Do you want to override it?",
                    self.name, self.version
                ))
                .default(false)
                .interact()?;

            if !confirm {
                println!("Operation cancelled by user");
                return Ok(());
            }
        }

        Progress::new()
            .run(&format!("Adding custom source: {}", self.name), async {
                let now = OffsetDateTime::now_utc();
                let source = DownloadSource {
                    name: self.name,
                    version: self.version,
                    description: self.description,
                    url: self.url,
                    source_type: self.source_type,
                    metadata: SourceMetadata {
                        added_date: now,
                        last_verified: Some(now),
                        last_downloaded: None,
                        downloads_count: 0,
                        verified: false,
                        processing_status: self.processing_status,
                        parent_source: self.parent_source,
                        build_info: None, // Could be added as optional parameters if needed
                    },
                    checksum: self.checksum,
                    checksum_type: self.checksum_type,
                    size: None,        // Will be determined during download
                    compression: None, // Could be added as parameter if needed
                    platform: self.platform,
                    architecture: self.architecture,
                    minimum_requirements: if self.min_cpu_cores.is_some()
                        || self.min_memory_mb.is_some()
                        || self.min_disk_gb.is_some()
                    {
                        Some(SystemRequirements {
                            cpu_cores: self.min_cpu_cores.unwrap_or(1),
                            memory_mb: self.min_memory_mb.unwrap_or(1024),
                            disk_gb: self.min_disk_gb.unwrap_or(10),
                            additional_requirements: None,
                        })
                    } else {
                        None
                    },
                    tags: self.tags.unwrap_or_default(),
                    mirrors: self.mirrors.unwrap_or_default(),
                    license: self.license,
                    documentation_url: self.documentation_url,
                    release_notes: self.release_notes,
                };

                registry.add_custom_source(source);
                registry.save(registry_path).await?;

                Ok(())
            })
            .await
    }
}
