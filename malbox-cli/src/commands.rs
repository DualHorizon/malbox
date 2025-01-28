use crate::error::Result;
use clap::{Parser, Subcommand};
use malbox_config::Config;

pub mod builder;
pub mod completion;
pub mod config;
pub mod daemon;
pub mod infra;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Builder(builder::BuilderCommand),
    Infra(infra::InfraCommand),
    Config(config::ConfigCommand),
    Daemon(daemon::DaemonCommand),
    Completion(completion::CompletionCommand),
}

impl Command for Cli {
    async fn execute(self, config: &Config) -> Result<()> {
        match self.command {
            Commands::Builder(cmd) => cmd.execute(config).await,
            Commands::Infra(cmd) => cmd.execute(config).await,
            Commands::Config(cmd) => cmd.execute(config).await,
            Commands::Daemon(cmd) => cmd.execute(config).await,
            Commands::Completion(cmd) => cmd.execute(config).await,
        }
    }
}

pub trait Command {
    async fn execute(self, config: &Config) -> Result<()>;
}
