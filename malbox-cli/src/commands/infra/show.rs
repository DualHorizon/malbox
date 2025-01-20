use crate::{commands::Command, error::Result, types::OutputFormat};
use clap::Parser;
use malbox_config::Config;

#[derive(Parser)]
pub struct ShowArgs {
    #[arg(short, long)]
    pub environment: String,
    #[arg(value_enum, short, long, default_value = "text")]
    pub format: OutputFormat,
}

impl Command for ShowArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        Ok(())
    }
}
