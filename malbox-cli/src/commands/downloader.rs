use crate::commands::Command;
use crate::error::Result;
use clap::{Parser, Subcommand};
use malbox_config::Config;

mod download;

pub use download::DownloadArgs;

#[derive(Parser)]
pub struct DownloaderCommand {
    #[command(subcommand)]
    command: DownloaderCommands,
}

#[derive(Subcommand)]
pub enum DownloaderCommands {
    Download(DownloadArgs),
}

impl Command for DownloaderCommand {
    async fn execute(self, config: &Config) -> Result<()> {
        match self.command {
            DownloaderCommands::Download(args) => args.execute(config).await,
        }
    }
}
