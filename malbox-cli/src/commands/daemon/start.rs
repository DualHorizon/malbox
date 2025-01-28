use crate::{commands::Command, error::Result};
use clap::Parser;
use malbox_config::Config;
use malbox_daemon::run;

#[derive(Parser)]
pub struct StartArgs {
    #[arg(short, long)]
    pub config_path: Option<String>,
}

// NOTE:
// We should implement indicatif to have a proper loader and show when the service is started properly.
// We might need to split the daemon `run` function into different parts to get more precise loading states.
// It's also worth to consider making a Daemon struct in malbox-daemon, and implement the different methods there, instead of a single `run` function.
impl Command for StartArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        run(config.clone())
            .await
            .map_err(|e| crate::error::CliError::Daemon(e))
    }
}
