use crate::utils::validation;
use crate::{commands::Command, error::Result, utils::progress::Progress};
use clap::Parser;
use malbox_config::Config;
use malbox_infra::packer::{
    build::{BuildConfig, BuildManager},
    templates::TemplateManager,
};

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
        let builder = BuildManager::new(config.paths.clone());

        Progress::new()
            .run(
                &format!(
                    "Refining image {} with playbook {}",
                    self.base, self.playbook
                ),
                async { todo!() },
            )
            .await
    }
}
