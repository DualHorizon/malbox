use crate::{commands::Command, error::Result, types::PlatformType, utils::Progress};
use clap::Parser;
use malbox_config::Config;
use std::path::PathBuf;

#[derive(Parser)]
pub struct BuildArgs {
    #[arg(value_enum)]
    pub platform: PlatformType,
    #[arg(short, long)]
    pub name: String,
    #[arg(long)]
    pub iso: Option<String>,
    #[arg(short, long)]
    pub force: bool,
    #[arg(short, long)]
    pub working_dir: Option<PathBuf>,
    #[arg(short, long = "var", value_parser = crate::utils::validation::parse_key_val)]
    pub variables: Vec<(String, String)>,
}

impl Command for BuildArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        let builder = malbox_infra::Builder::new(config.clone());

        Progress::new()
            .run("Building base image...", async {
                builder
                    .build(malbox_infrastructure::BuilderConfig {
                        platform: self.platform.into(),
                        name: self.name,
                        iso: self.iso,
                        force: self.force,
                        working_dir: self.working_dir,
                        variables: self.variables.into_iter().collect(),
                    })
                    .await
                    .map_err(|e| crate::error::Error::Infrastructure(e.to_string()))
            })
            .await
    }
}
