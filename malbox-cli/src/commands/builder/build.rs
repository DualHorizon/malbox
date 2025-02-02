use crate::{commands::Command, error::Result, types::PlatformType, utils::progress::Progress};
use clap::Parser;
use malbox_config::Config;
use malbox_infra::packer::{
    build::{BuildConfig, BuildManager},
    templates::TemplateManager,
};
use std::collections::HashMap;
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
        let path = PathBuf::from("/home/shard/.config/malbox/templates/windows/base.pkr.hcl");

        let template_manager = TemplateManager::new();
        let template = template_manager.load(path).await?;

        template_manager.display_template_info(&template);

        let mut variables: HashMap<String, String> = self.variables.into_iter().collect();
        template_manager
            .prompt_for_variables(&template, &mut variables)
            .await?;

        let build_config = BuildConfig {
            platform: self.platform.into(),
            name: self.name.clone(),
            force: self.force,
            working_dir: self.working_dir,
            iso: self.iso,
            template: config.paths.packer_dir.to_str().unwrap().to_string(),
            variables,
        };

        let builder = BuildManager::new(config.paths.clone());

        Progress::new()
            .run("Building base image...", async {
                builder
                    .build(build_config)
                    .await
                    .map_err(|e| crate::error::CliError::Infrastructure(e))
            })
            .await
    }
}
