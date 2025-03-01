use crate::{
    commands::Command,
    error::{CliError, Result},
    utils::progress::Progress,
};
use clap::Parser;
use dialoguer::Confirm;
use malbox_config::Config;
use malbox_downloader::{
    Architecture, Platform, ProcessingStatus, SourceMetadata, SourceRegistry, SourceType,
    SourceVariant, SystemRequirements,
};
use time::OffsetDateTime;

#[derive(Parser)]
pub struct AddSourceArgs {
    #[arg(short = 'f', long)]
    pub family: String,
    #[arg(short = 'e', long)]
    pub edition: String,
    #[arg(short = 'v', long)]
    pub version: String,
    #[arg(short = 'i', long)]
    pub variant_id: String,
    #[arg(short, long)]
    pub description: String,
    #[arg(short, long)]
    pub url: String,
    #[arg(value_enum, short, long, default_value = "iso")]
    pub source_type: SourceType,
    #[arg(value_enum, short = 'p', long)]
    pub platform: Option<Platform>,
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
        let mut registry = SourceRegistry::load(registry_path.clone()).await?;

        let platform = self.platform.unwrap_or_else(|| match self.family.as_str() {
            "windows" => Platform::Windows,
            "linux" => Platform::Linux,
            "macos" => Platform::MacOS,
            _ => Platform::Linux,
        });

        if registry.source_exists(
            Some(&self.family),
            Some(&self.edition),
            Some(&self.version),
            Some(&self.variant_id),
        ) {
            let confirm = Confirm::new()
                .with_prompt(format!(
                    "Source already exists. Do you want to override it?"
                ))
                .default(false)
                .interact()?;

            if !confirm {
                println!("Operation cancelled by user");
                return Ok(());
            }
        }

        Progress::new()
            .run("Adding source", async {
                let now = OffsetDateTime::now_utc();

                let source_variant = SourceVariant {
                    id: self.variant_id,
                    description: self.description,
                    architecture: self.architecture,
                    url: self.url,
                    checksum: self.checksum,
                    checksum_type: self.checksum_type,
                    size: None, // Will be determined during download
                    source_type: self.source_type,
                    compression: None,
                    metadata: SourceMetadata {
                        added_date: now,
                        last_verified: Some(now),
                        last_downloaded: None,
                        downloads_count: 0,
                        verified: false,
                        processing_status: self.processing_status,
                        parent_source: self.parent_source,
                        build_info: None,
                        local_path: None,
                    },
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
                    mirrors: self.mirrors.unwrap_or_default(),
                    license: self.license,
                    documentation_url: self.documentation_url,
                };

                registry.add_source(&self.family, &self.edition, &self.version, source_variant)?;
                registry.save(registry_path).await?;

                println!("Source added successfully");
                Ok(())
            })
            .await
    }
}
