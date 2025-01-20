use crate::commands::Command;
use crate::error::Result;
use clap::{Parser, Subcommand};
use malbox_ansible::PlaybookManager;
use malbox_config::Config;

mod playbook;
mod validate;
mod vars;

pub use playbook::{PlaybookCommand, PlaybookCommands};
pub use validate::ValidateArgs;
pub use vars::{VarsCommand, VarsCommands};

#[derive(Parser)]
pub struct ConfigCommand {
    #[command(subcommand)]
    command: ConfigCommands,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    Playbook(PlaybookCommand),
    Vars(VarsCommand),
    Validate(ValidateArgs),
}

impl Command for ConfigCommand {
    async fn execute(self, config: &Config) -> Result<()> {
        match self.command {
            ConfigCommands::Playbook(cmd) => cmd.execute(config).await,
            ConfigCommands::Vars(cmd) => cmd.execute(config).await,
            ConfigCommands::Validate(args) => args.execute(config).await,
        }
    }
}
