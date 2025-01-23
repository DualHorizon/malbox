use crate::{commands::Command, error::Result, utils::progress::Progress};
use clap::Parser;
use malbox_config::Config;
use std::path::PathBuf;

#[derive(Parser)]
pub struct InitArgs {
    #[arg(short, long)]
    pub working_dir: Option<PathBuf>,
    #[arg(short, long)]
    pub force: bool,
}

impl Command for InitArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        let builder = malbox_infra::Builder::new(config.clone());

        Progress::new()
            .run("Initializing builder environment...", async {
                builder
                    .init(self.working_dir, self.force)
                    .await
                    .map_err(|e| crate::error::CliError::Infrastructure(e))
            })
            .await
    }
}
