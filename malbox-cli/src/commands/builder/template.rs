use crate::{
    commands::Command,
    error::Result,
    types::{OutputFormat, PlatformType},
    utils::Progress,
};
use clap::{Parser, Subcommand};
use malbox_config::Config;
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
        let manager = malbox_infrastructure::TemplateManager::new(config.clone());

        Progress::new()
            .run("Fetching templates...", async {
                let templates = manager
                    .list(self.platform.map(Into::into))
                    .await
                    .map_err(|e| crate::error::Error::Infrastructure(e.to_string()))?;

                match self.format {
                    OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&templates)?);
                    }
                    OutputFormat::Yaml => {
                        println!("{}", serde_yaml::to_string(&templates)?);
                    }
                    OutputFormat::Text => {
                        println!("Available templates:");
                        for template in templates {
                            println!("\n{} ({:?})", template.name, template.platform);
                            if self.detailed {
                                println!("  Description: {}", template.description);
                                if let Some(base) = template.base {
                                    println!("  Base: {}", base);
                                }
                                if !template.playbooks.is_empty() {
                                    println!("  Playbooks:");
                                    for playbook in template.playbooks {
                                        println!("    - {}", playbook);
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(())
            })
            .await
    }
}

impl Command for CreateArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        let manager = malbox_infrastructure::TemplateManager::new(config.clone());

        Progress::new()
            .run(&format!("Creating template '{}'...", self.name), async {
                manager
                    .create(malbox_infrastructure::Template {
                        name: self.name,
                        platform: self.platform.into(),
                        description: self.description,
                        base: self.base,
                        variables: HashMap::new(),
                        playbooks: Vec::new(),
                    })
                    .await
                    .map_err(|e| crate::error::Error::Infrastructure(e.to_string()))
            })
            .await
    }
}

impl Command for ExportArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        let manager = malbox_infrastructure::TemplateManager::new(config.clone());

        Progress::new()
            .run(&format!("Exporting template '{}'...", self.name), async {
                manager
                    .export(&self.name, self.output)
                    .await
                    .map_err(|e| crate::error::Error::Infrastructure(e.to_string()))
            })
            .await
    }
}

impl Command for ImportArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        let manager = malbox_infrastructure::TemplateManager::new(config.clone());

        Progress::new()
            .run("Importing template...", async {
                manager
                    .import(self.file, self.name, self.force)
                    .await
                    .map_err(|e| crate::error::Error::Infrastructure(e.to_string()))
            })
            .await
    }
}
