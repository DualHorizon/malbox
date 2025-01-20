use crate::{commands::Command, error::Result, utils::progress::Progress};
use clap::Parser;
use malbox_config::Config;

#[derive(Parser)]
pub struct ImportArgs {
    #[arg(short, long)]
    pub environment: String,
    #[arg(short, long)]
    pub address: String,
    #[arg(short, long)]
    pub id: String,
}

impl Command for ImportArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        Progress::new()
            .run("Importing existing infrastructure...", async { Ok(()) })
            .await
    }
}
