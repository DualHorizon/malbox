use crate::{
    commands::Command,
    error::{CliError, Result},
};
use clap::Parser;
use malbox_config::Config;
use malbox_downloader::{DownloadRegistry, Downloader};
use std::path::PathBuf;

#[derive(Parser)]
pub struct DownloadArgs {
    #[arg(short, long)]
    pub name: Option<String>,
    #[arg(short, long)]
    pub version: Option<String>,
    #[arg(short, long)]
    pub url: Option<String>,
    #[arg(short, long)]
    pub registry: Option<String>,
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

impl Command for DownloadArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        let registry_path = config.paths.download_dir.join("download_registry.json");
        let registry = DownloadRegistry::load(registry_path.clone()).await?;
        let downloader = Downloader::builder().show_progress(true).build();

        match (self.url.as_ref(), self.name.as_ref()) {
            (Some(url), None) => {
                let output_path = downloader
                    .download(url, None, &config.paths.download_dir, self.output)
                    .await?;
                println!("\nDownload saved to: {}", output_path.display());
            }
            (None, Some(name)) => {
                let source = registry.get_source(name, self.version.as_deref())?;
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
            (None, None) => {
                return Err(CliError::InvalidArgument(
                    "Either --name or --url must be provided".to_string(),
                ));
            }
            (Some(_), Some(_)) => {
                return Err(CliError::InvalidArgument(
                    "Cannot specify both --name and --url".to_string(),
                ));
            }
        }

        Ok(())
    }
}
