use crate::{types::Platform, types::Template, Error, Result};
use std::{collections::HashMap, fs, path::PathBuf};
use toml;

pub struct TemplateManager {
    config: malbox_config::Config,
}

impl TemplateManager {
    pub fn new(config: malbox_config::Config) -> Self {
        Self { config }
    }

    pub async fn list(&self, platform: Option<Platform>) -> Result<Vec<Template>> {
        let mut templates = Vec::new();
        let templates_dir = self.config.paths.config_dir.join("templates");

        for entry in templates_dir.read_dir()? {
            let entry = entry?;
            if entry.file_type()?.is_file()
                && entry.path().extension().map_or(false, |ext| ext == "toml")
            {
                let template: Template = self.load_template(&entry.path())?;
                if let Some(ref p) = platform {
                    if template.platform == *p {
                        templates.push(template);
                    }
                } else {
                    templates.push(template);
                }
            }
        }

        Ok(templates)
    }

    pub async fn create(&self, template: Template) -> Result<()> {
        let path = self
            .config
            .paths
            .config_dir
            .join("templates")
            .join(format!("{}.toml", template.name));

        if path.exists() {
            return Err(Error::Template(format!(
                "Template {} already exists",
                template.name
            )));
        }

        let content =
            toml::to_string_pretty(&template).map_err(|e| Error::Template(e.to_string()))?;

        tokio::fs::write(path, content).await?;
        Ok(())
    }

    pub async fn export(&self, name: &str, output: PathBuf) -> Result<()> {
        let template_path = self
            .config
            .paths
            .config_dir
            .join("templates")
            .join(format!("{}.toml", name));

        if !template_path.exists() {
            return Err(Error::NotFound(format!("Template {} not found", name)));
        }

        tokio::fs::copy(template_path, output).await?;
        Ok(())
    }

    pub async fn import(&self, file: PathBuf, name: String, force: bool) -> Result<()> {
        let dest_path = self
            .config
            .paths
            .config_dir
            .join("templates")
            .join(format!("{}.toml", name));

        if !force && dest_path.exists() {
            return Err(Error::Template(format!("Template {} already exists", name)));
        }

        let template: Template = self.load_template(&file)?;
        self.create(template).await?;
        Ok(())
    }

    fn load_template(&self, path: &PathBuf) -> Result<Template> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| Error::Template(e.to_string()))
    }
}
