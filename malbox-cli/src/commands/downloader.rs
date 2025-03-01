use crate::commands::Command;
use crate::error::Result;
use clap::{Parser, Subcommand};
use malbox_config::Config;

mod download;
mod sources;

pub use download::DownloadArgs;
pub use sources::SourceCommand;

#[derive(Parser)]
pub struct DownloaderCommand {
    #[command(subcommand)]
    command: DownloaderCommands,
}

#[derive(Subcommand)]
pub enum DownloaderCommands {
    /// Download a file directly or via its source
    Download(DownloadArgs),
    #[command(subcommand)]
    /// Manage download sources and their configuration
    Source(SourceCommand),
}

impl Command for DownloaderCommand {
    async fn execute(self, config: &Config) -> Result<()> {
        match self.command {
            DownloaderCommands::Download(args) => args.execute(config).await,
            DownloaderCommands::Source(cmd) => cmd.execute(config).await,
        }
    }
}
