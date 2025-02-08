use crate::{
    commands::Command,
    error::{CliError, Result},
    utils::progress::Progress,
};
use clap::Parser;
use malbox_config::Config;
use malbox_downloader::{DownloadRegistry, DownloadSource};
use std::path::PathBuf;

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
    #[arg(short, long)]
    pub checksum: Option<String>,
    #[arg(short = 't', long = "checksum-type")]
    pub checksum_type: Option<String>,
}

impl Command for AddSourceArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        let registry_path = config.paths.download_dir.join("download_registry.json");

        Progress::new()
            .run(&format!("Adding custom source: {}", self.name), async {
                let mut registry = DownloadRegistry::load(registry_path.clone()).await?;

                let source = DownloadSource {
                    name: self.name,
                    version: self.version,
                    description: self.description,
                    url: self.url,
                    checksum: self.checksum,
                    checksum_type: self.checksum_type,
                    size: None,
                    tags: vec![],
                    mirrors: vec![],
                };

                registry.add_custom_source(source);
                registry.save(registry_path).await?;
                Ok(())
            })
            .await
    }
}
