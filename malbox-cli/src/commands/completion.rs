use crate::{commands::Command as CliCommand, error::Result};
use clap::{CommandFactory, Parser};
use clap_complete::Shell;
use malbox_config::Config;

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
