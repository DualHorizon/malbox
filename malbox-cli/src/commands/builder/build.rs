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
        let template_manager = TemplateManager::new();
        let template = template_manager
            .load(config.paths.templates_dir.clone())
            .await?;

        let mut variables: HashMap<String, String> = self.variables.into_iter().collect();

        let missing_vars = template_manager.get_missing_variables(&template, &variables)?;
        if !missing_vars.is_empty() {
            for var_name in missing_vars {
                // Could use dialoguer crate here for better UX
                println!("Required variable '{}' is missing.", var_name);
                println!("Please enter value for {}: ", var_name);
                let mut value = String::new();
                std::io::stdin().read_line(&mut value)?;
                variables.insert(var_name, value.trim().to_string());
            }
        }

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
