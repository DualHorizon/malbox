use crate::commands::Command;
use crate::error::Result;
use clap::Subcommand;
use malbox_config::Config;

mod add;
mod list;

pub use add::AddSourceArgs;
pub use list::ListSourcesArgs;

#[derive(Subcommand)]
pub enum SourceCommand {
    /// Add a new source
    Add(AddSourceArgs),
    /// List available sources
    List(ListSourcesArgs),
    // Remove all existing sources
}

impl Command for SourceCommand {
    async fn execute(self, config: &Config) -> Result<()> {
        match self {
            Self::Add(args) => args.execute(config).await,
            Self::List(args) => args.execute(config).await,
        }
    }
}
