use crate::{commands::Command, error::Result, utils::progress::Progress};
use clap::Parser;
use malbox_config::Config;

#[derive(Parser)]
pub struct InitArgs {
    #[arg(short, long)]
    pub environment: String,
    #[arg(short, long)]
    pub force: bool,
    #[arg(short, long = "backend-config")]
    pub backend_config: Vec<String>,
}

impl Command for InitArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        Progress::new()
            .run(
                &format!(
                    "Initializing infrastructure for environment: {}",
                    self.environment
                ),
                async { Ok(()) },
            )
            .await
    }
}
