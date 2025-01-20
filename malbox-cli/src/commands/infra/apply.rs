use crate::{
    commands::Command,
    error::Result,
    utils::{validation::parse_key_val, Progress},
};
use clap::Parser;
use dialoguer::Confirm;
use malbox_config::Config;

#[derive(Parser)]
pub struct ApplyArgs {
    #[arg(short, long)]
    pub environment: String,
    #[arg(short, long = "var", value_parser = parse_key_val)]
    pub variables: Vec<(String, String)>,
    #[arg(short, long)]
    pub auto_approve: bool,
    #[arg(short, long)]
    pub plan: Option<String>,
}

impl Command for ApplyArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        if !self.auto_approve {
            if !Confirm::new()
                .with_prompt("Do you want to apply these changes?")
                .interact()?
            {
                return Ok(());
            }
        }

        Progress::new()
            .run(
                &format!("Applying changes to environment: {}", self.environment),
                async { Ok(()) },
            )
            .await
    }
}
