use crate::{
    commands::Command,
    error::{CliError, Result},
    types::OutputFormat,
    utils::progress::Progress,
};
use bon::Builder;
use clap::{Parser, Subcommand};
use malbox_config::Config;
use malbox_infra::PlaybookManager;

#[derive(Parser)]
pub struct PlaybookCommand {
    #[command(subcommand)]
    command: PlaybookCommands,
}

#[derive(Subcommand)]
pub enum PlaybookCommands {
    List(ListArgs),
    Create(CreateArgs),
    Edit(EditArgs),
    Apply(ApplyArgs),
    Test(TestArgs),
}

impl Command for PlaybookCommand {
    async fn execute(self, config: &Config) -> Result<()> {
        let playbook_manager = PlaybookManager::new(config.clone());
        match self.command {
            PlaybookCommands::List(args) => args.execute(config).await,
            PlaybookCommands::Create(args) => args.execute(config).await,
            PlaybookCommands::Edit(args) => args.execute(config).await,
            PlaybookCommands::Apply(args) => args.execute(config).await,
            PlaybookCommands::Test(args) => args.execute(config).await,
        }
    }
}

#[derive(Parser, Builder)]
pub struct ListArgs {
    #[arg(short, long)]
    #[builder(default = false)]
    pub detailed: bool,

    #[arg(value_enum, short, long, default_value = "text")]
    #[builder(default = OutputFormat::Text)]
    pub format: OutputFormat,
}

#[derive(Parser, Builder)]
pub struct CreateArgs {
    pub name: String,
    pub description: String,

    #[arg(short, long)]
    #[builder(default)]
    pub roles: Vec<String>,
}

#[derive(Parser, Builder)]
pub struct EditArgs {
    pub name: String,

    #[arg(short, long)]
    pub editor: Option<String>,
}

#[derive(Parser, Builder)]
pub struct ApplyArgs {
    pub name: String,

    #[arg(short, long)]
    #[builder(default)]
    pub targets: Vec<String>,

    #[arg(long)]
    #[builder(default = false)]
    pub check: bool,

    #[arg(short, long = "var", value_parser = crate::utils::validation::parse_key_val)]
    #[builder(default)]
    pub variables: Vec<(String, String)>,
}

#[derive(Parser, Builder)]
pub struct TestArgs {
    pub name: String,

    #[arg(short, long, default_value = "test")]
    #[builder(default = "\"test\"".to_string())]
    pub environment: String,
}

impl Command for ListArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        let playbook_manager = PlaybookManager::new(config.clone());
        let progress = Progress::new();
        progress
            .run("Fetching playbooks...", async {
                let playbooks = playbook_manager.list_playbooks().await?;
                match self.format {
                    // IMPORTANT: tofix
                    OutputFormat::Json => {
                        println!("{:?}", serde_json::to_string_pretty(&playbooks).unwrap());
                    }
                    OutputFormat::Yaml => {
                        println!("{:?}", serde_yaml::to_string(&playbooks).unwrap());
                    }
                    OutputFormat::Text => {
                        println!("Available playbooks:");
                        for playbook in playbooks {
                            if self.detailed {
                                println!("\n{}", playbook.name);
                                println!("  Description: {}", playbook.description);
                                if !playbook.roles.is_empty() {
                                    println!("  Roles:");
                                    for role in playbook.roles {
                                        println!("    - {}", role);
                                    }
                                }
                            } else {
                                println!("- {}", playbook.name);
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
        let playbook_manager = PlaybookManager::new(config.clone());
        let progress = Progress::new();
        progress
            .run(&format!("Creating playbook '{}'...", self.name), async {
                playbook_manager
                    .create_playbook(&self.name, &self.description, &self.roles)
                    .await
                    .map_err(|e| crate::error::CliError::Infrastructure(e))
            })
            .await
    }
}

impl Command for EditArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        let playbook_manager = PlaybookManager::new(config.clone());
        let progress = Progress::new();
        progress
            .run(
                &format!("Opening playbook '{}' for editing...", self.name),
                async {
                    playbook_manager
                        .edit_playbook(&self.name, self.editor.as_deref())
                        .await
                        .map_err(|e| crate::error::CliError::Infrastructure(e))
                },
            )
            .await
    }
}

impl Command for ApplyArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        let playbook_manager = PlaybookManager::new(config.clone());
        let progress = Progress::new();
        progress
            .run(
                &format!(
                    "Applying playbook '{}' to {} target(s)...",
                    self.name,
                    self.targets.len()
                ),
                async {
                    playbook_manager
                        .apply_playbook(&self.name, &self.targets, self.check, &self.variables)
                        .await
                        .map_err(|e| crate::error::CliError::Infrastructure(e))
                },
            )
            .await
    }
}

impl Command for TestArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        let playbook_manager = PlaybookManager::new(config.clone());
        let progress = Progress::new();
        progress
            .run(
                &format!(
                    "Testing playbook '{}' in {} environment...",
                    self.name, self.environment
                ),
                async {
                    playbook_manager
                        .test_playbook(&self.name, &self.environment)
                        .await
                        .map_err(|e| crate::error::CliError::Infrastructure(e))
                },
            )
            .await
    }
}
