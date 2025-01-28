use crate::commands::Command;
use crate::error::Result;
use clap::{Parser, Subcommand};
use malbox_config::Config;

mod start;
use start::StartArgs;

#[derive(Parser)]
pub struct DaemonCommand {
    #[command(subcommand)]
    command: DaemonCommands,
}

#[derive(Subcommand)]
pub enum DaemonCommands {
    Start(StartArgs),
}

impl Command for DaemonCommand {
    async fn execute(self, config: &Config) -> Result<()> {
        match self.command {
            DaemonCommands::Start(cmd) => cmd.execute(config).await,
        }
    }
}
