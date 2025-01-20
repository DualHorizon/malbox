use crate::{ConfigError, Platform};
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct TemplateConfig {
    #[builder(default)]
    pub windows: HashMap<String, Template>,
    #[builder(default)]
    pub linux: HashMap<String, Template>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub platform: Platform,
    pub packer: PackerConfig,
    pub ansible: Option<AnsibleConfig>,
    pub terraform: Option<TerraformConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct PackerConfig {
    pub template: String,
    #[builder(default)]
    pub vars: HashMap<String, String>,
    #[builder(default = false)]
    pub headless: bool,
    #[builder(default = 4)]
    pub cpu_count: u32,
    #[builder(default = 8192)]
    pub memory_mb: u32,
    #[builder(default = 100)]
    pub disk_size_gb: u32,
    #[builder(default)]
    pub provisioners: Vec<Provisioner>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct AnsibleConfig {
    pub playbook: String,
    #[builder(default)]
    pub vars: HashMap<String, String>,
    #[builder(default)]
    pub roles: Vec<String>,
    #[builder(default = false)]
    pub r#become: bool,
    pub inventory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct TerraformConfig {
    pub template: String,
    #[builder(default)]
    pub vars: HashMap<String, String>,
    pub backend: Option<Backend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backend {
    pub backend_type: String,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Provisioner {
    #[serde(rename = "shell")]
    Shell {
        scripts: Vec<String>,
        #[serde(default)]
        environment_vars: HashMap<String, String>,
    },
    #[serde(rename = "ansible")]
    Ansible {
        playbook: String,
        #[serde(default)]
        extra_vars: HashMap<String, String>,
    },
    #[serde(rename = "powershell")]
    PowerShell {
        scripts: Vec<String>,
        #[serde(default)]
        environment_vars: HashMap<String, String>,
    },
}

impl TemplateConfig {
    pub async fn load(config_root: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let windows =
            Self::load_platform_templates(config_root.as_ref().join("templates").join("windows"))
                .await?;
        let linux =
            Self::load_platform_templates(config_root.as_ref().join("templates").join("linux"))
                .await?;

        Ok(Self::builder().windows(windows).linux(linux).build())
    }

    async fn load_platform_templates(
        path: impl AsRef<Path>,
    ) -> Result<HashMap<String, Template>, ConfigError> {
        let mut templates = HashMap::new();
        let mut entries = fs::read_dir(path.as_ref()).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension() == Some("toml".as_ref()) {
                let content = fs::read_to_string(entry.path()).await?;
                let template: Template =
                    toml::from_str(&content).map_err(|e| ConfigError::Parse {
                        file: entry.path().display().to_string(),
                        error: e.to_string(),
                    })?;
                templates.insert(template.name.clone(), template);
            }
        }

        Ok(templates)
    }

    pub fn get_template(&self, platform: Platform, name: &str) -> Option<&Template> {
        match platform {
            Platform::Windows => self.windows.get(name),
            Platform::Linux => self.linux.get(name),
        }
    }

    pub fn get_templates_for_platform(&self, platform: Platform) -> &HashMap<String, Template> {
        match platform {
            Platform::Windows => &self.windows,
            Platform::Linux => &self.linux,
        }
    }
}
