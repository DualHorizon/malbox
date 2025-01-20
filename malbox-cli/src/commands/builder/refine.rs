use crate::utils::validation;
use crate::{commands::Command, error::Result, utils::Progress};
use clap::Parser;
use malbox_config::Config;

#[derive(Parser)]
pub struct RefineArgs {
    #[arg(short, long)]
    pub base: String,
    #[arg(short, long)]
    pub name: String,
    #[arg(short, long)]
    pub playbook: String,
    #[arg(short, long)]
    pub force: bool,
    #[arg(short, long = "var", value_parser = validation::parse_key_val)]
    pub variables: Vec<(String, String)>,
}

impl Command for RefineArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        let builder = malbox_infrastructure::Builder::new(config.clone());

        Progress::new()
            .run(
                &format!(
                    "Refining image {} with playbook {}",
                    self.base, self.playbook
                ),
                async {
                    builder
                        .refine(malbox_infrastructure::RefineConfig {
                            base: self.base,
                            name: self.name,
                            playbook: self.playbook,
                            force: self.force,
                            variables: self.variables.into_iter().collect(),
                        })
                        .await
                        .map_err(|e| crate::error::Error::Infrastructure(e.to_string()))
                },
            )
            .await
    }
}
