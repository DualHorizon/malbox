use crate::{
    commands::Command,
    error::Result,
    utils::{progress::Progress, validation::parse_key_val},
};
use clap::Parser;
use malbox_config::Config;

#[derive(Parser)]
pub struct PlanArgs {
    #[arg(short, long)]
    pub environment: String,
    #[arg(short, long = "var", value_parser = parse_key_val)]
    pub variables: Vec<(String, String)>,
    #[arg(short, long)]
    pub out: Option<String>,
    #[arg(short, long)]
    pub detailed: bool,
}

impl Command for PlanArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        Progress::new()
            .run(
                &format!("Planning changes for environment: {}", self.environment),
                async { Ok(()) },
            )
            .await
    }
}
