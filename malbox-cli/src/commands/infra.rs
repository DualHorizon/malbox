use crate::commands::Command;
use crate::error::Result;
use clap::{Parser, Subcommand};
use malbox_config::Config;

mod apply;
mod destroy;
mod import;
mod init;
mod plan;
mod show;

pub use apply::ApplyArgs;
pub use destroy::DestroyArgs;
pub use import::ImportArgs;
pub use init::InitArgs;
pub use plan::PlanArgs;
pub use show::ShowArgs;

#[derive(Parser)]
pub struct InfraCommand {
    #[command(subcommand)]
    command: InfraCommands,
}

#[derive(Subcommand)]
pub enum InfraCommands {
    Init(InitArgs),
    Plan(PlanArgs),
    Apply(ApplyArgs),
    Destroy(DestroyArgs),
    Show(ShowArgs),
    Import(ImportArgs),
}

impl Command for InfraCommand {
    async fn execute(self, config: &Config) -> Result<()> {
        match self.command {
            InfraCommands::Init(args) => args.execute(config).await,
            InfraCommands::Plan(args) => args.execute(config).await,
            InfraCommands::Apply(args) => args.execute(config).await,
            InfraCommands::Destroy(args) => args.execute(config).await,
            InfraCommands::Show(args) => args.execute(config).await,
            InfraCommands::Import(args) => args.execute(config).await,
        }
    }
}
