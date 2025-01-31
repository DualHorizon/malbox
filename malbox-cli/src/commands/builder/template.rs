use crate::{
    commands::Command,
    error::Result,
    types::{OutputFormat, PlatformType},
    utils::progress::Progress,
};
use clap::{Parser, Subcommand};
use malbox_config::Config;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
pub struct TemplateCommand {
    #[command(subcommand)]
    command: TemplateCommands,
}

#[derive(Subcommand)]
pub enum TemplateCommands {
    List(ListArgs),
    Create(CreateArgs),
    Export(ExportArgs),
    Import(ImportArgs),
}

#[derive(Parser)]
pub struct ListArgs {
    #[arg(value_enum, short, long)]
    pub platform: Option<PlatformType>,
    #[arg(short, long)]
    pub detailed: bool,
    #[arg(value_enum, short, long, default_value = "text")]
    pub format: OutputFormat,
}

#[derive(Parser)]
pub struct CreateArgs {
    #[arg(short, long)]
    pub name: String,
    #[arg(value_enum)]
    pub platform: PlatformType,
    #[arg(short, long)]
    pub description: String,
    #[arg(short, long)]
    pub base: Option<String>,
}

#[derive(Parser)]
pub struct ExportArgs {
    #[arg(short, long)]
    pub name: String,
    #[arg(short, long)]
    pub output: PathBuf,
}

#[derive(Parser)]
pub struct ImportArgs {
    #[arg(short, long)]
    pub file: PathBuf,
    #[arg(short, long)]
    pub name: String,
    #[arg(short, long)]
    pub force: bool,
}

impl Command for TemplateCommand {
    async fn execute(self, config: &Config) -> Result<()> {
        match self.command {
            TemplateCommands::List(args) => args.execute(config).await,
            TemplateCommands::Create(args) => args.execute(config).await,
            TemplateCommands::Export(args) => args.execute(config).await,
            TemplateCommands::Import(args) => args.execute(config).await,
        }
    }
}

impl Command for ListArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        todo!()
    }
}

impl Command for CreateArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        Progress::new()
            .run(&format!("Creating template '{}'...", self.name), async {
                todo!()
            })
            .await
    }
}

impl Command for ExportArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        todo!()
    }
}

impl Command for ImportArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        todo!()
    }
}
