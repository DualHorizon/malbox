use crate::commands::Command;
use crate::error::Result;
use clap::{Parser, Subcommand};
use malbox_config::Config;

mod add_source;
mod download;
mod list_sources;

pub use add_source::AddSourceArgs;
pub use download::DownloadArgs;
pub use list_sources::ListSourcesArgs;

#[derive(Parser)]
pub struct DownloaderCommand {
    #[command(subcommand)]
    command: DownloaderCommands,
}

#[derive(Subcommand)]
pub enum DownloaderCommands {
    Download(DownloadArgs),
    AddSource(AddSourceArgs),
    ListSources(ListSourcesArgs),
}

impl Command for DownloaderCommand {
    async fn execute(self, config: &Config) -> Result<()> {
        match self.command {
            DownloaderCommands::Download(args) => args.execute(config).await,
            DownloaderCommands::AddSource(args) => args.execute(config).await,
            DownloaderCommands::ListSources(args) => args.execute(config).await,
        }
    }
}
