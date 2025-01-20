use crate::commands::Command;
use crate::error::Result;
use clap::{Parser, Subcommand};
use malbox_config::Config;

mod build;
mod init;
mod refine;
mod template;

pub use build::BuildArgs;
pub use init::InitArgs;
pub use refine::RefineArgs;
pub use template::{TemplateCommand, TemplateCommands};

#[derive(Parser)]
pub struct BuilderCommand {
    #[command(subcommand)]
    command: BuilderCommands,
}

#[derive(Subcommand)]
pub enum BuilderCommands {
    Build(BuildArgs),
    Refine(RefineArgs),
    Template(TemplateCommand),
    Init(InitArgs),
}

impl Command for BuilderCommand {
    async fn execute(self, config: &Config) -> Result<()> {
        match self.command {
            BuilderCommands::Build(args) => args.execute(config).await,
            BuilderCommands::Refine(args) => args.execute(config).await,
            BuilderCommands::Template(cmd) => cmd.execute(config).await,
            BuilderCommands::Init(args) => args.execute(config).await,
        }
    }
}
