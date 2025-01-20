use crate::{commands::Command as CliCommand, error::Result, Config};
use clap::{Command, CommandFactory, Parser, ValueEnum};
use clap_complete::Shell;

#[derive(Parser)]
pub struct CompletionCommand {
    #[arg(value_enum)]
    shell: Shell,
}

impl CliCommand for CompletionCommand {
    async fn execute(self, _config: &Config) -> Result<()> {
        let mut cmd = crate::Cli::command();
        clap_complete::generate(self.shell, &mut cmd, "malbox", &mut std::io::stdout());
        Ok(())
    }
}

#[derive(Subcommand)]
pub enum Commands {
    Builder(builder::BuilderCommand),
    Infra(infra::InfraCommand),
    Config(config::ConfigCommand),
    Completion(completion::CompletionCommand),
}
