use crate::{commands::Command, error::Result, utils::progress::Progress};
use clap::Parser;
use malbox_config::Config;
use malbox_infra::packer::{
    build::{BuildConfig, BuildManager},
    templates::TemplateManager,
};
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
        let builder = BuildManager::new(config.paths.clone());

        Progress::new()
            .run("Initializing builder environment...", async { todo!() })
            .await
    }
}
