use crate::shared_types::MachinePlatform;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateConfig {
    pub windows: HashMap<String, Template>,
    pub linux: HashMap<String, Template>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub platform: MachinePlatform,
    pub packer: PackerConfig,
    pub ansible: Option<AnsibleConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackerConfig {
    pub template: String,
    pub vars: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnsibleConfig {
    pub playbook: String,
    pub vars: HashMap<String, String>,
}

impl TemplateConfig {
    pub async fn load(config_root: &Path) -> anyhow::Result<Self> {
        let windows =
            Self::load_platform_templates(config_root.join("templates").join("windows")).await?;
        let linux =
            Self::load_platform_templates(config_root.join("templates").join("linux")).await?;

        Ok(TemplateConfig { windows, linux })
    }

    async fn load_platform_templates(path: PathBuf) -> anyhow::Result<HashMap<String, Template>> {
        let mut templates = HashMap::new();
        let mut entries = tokio::fs::read_dir(path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension() == Some("toml".as_ref()) {
                let content = tokio::fs::read_to_string(entry.path()).await?;
                let template: Template = toml::from_str(&content)?;
                templates.insert(template.name.clone(), template);
            }
        }

        Ok(templates)
    }
}
