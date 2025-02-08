use crate::{
    commands::Command,
    error::{CliError, Result},
    utils::{interaction::templates::TemplatePrompt, progress::Progress},
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

        if let Some(url) = self.url {
            // Should be done in downloader, based on the actualy file name.
            // Since it may not be in the URL.
            let output = self.output.unwrap_or_else(|| {
                config
                    .paths
                    .download_dir
                    .join(url.split("/").last().unwrap_or("download.iso"))
            });
            downloader.download(&url, output).await?;
            return Ok(());
        }

        let name = self.name.as_deref().ok_or_else(|| {
            CliError::InvalidArgument("Either --name or --url most be provided".to_string())
        })?;

        let source = registry.get_source(name, self.version.as_deref())?;
        let filename = format!("{}-{}.iso", source.name, source.version);
        let output = self
            .output
            .unwrap_or_else(|| config.paths.download_dir.join(filename));

        downloader.download(&source.url, output).await?;

        Ok(())
    }
}
