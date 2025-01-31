use crate::commands::Command;
use crate::error::Result;
use clap::{Parser, Subcommand};
use malbox_config::Config;

mod playbook;
mod validate;
mod vars;

pub use validate::ValidateArgs;
pub use vars::VarsCommand;

#[derive(Parser)]
pub struct ConfigCommand {
    #[command(subcommand)]
    command: ConfigCommands,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    Vars(VarsCommand),
    Validate(ValidateArgs),
}

impl Command for ConfigCommand {
    async fn execute(self, config: &Config) -> Result<()> {
        match self.command {
            ConfigCommands::Vars(cmd) => cmd.execute(config).await,
            ConfigCommands::Validate(args) => args.execute(config).await,
        }
    }
}
