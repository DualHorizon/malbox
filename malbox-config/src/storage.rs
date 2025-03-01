use crate::error::ConfigError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    #[serde(default = "default_config_dir")]
    pub config_dir: PathBuf,
    #[serde(default = "default_cache_dir")]
    pub cache_dir: PathBuf,
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
    #[serde(default = "default_state_dir")]
    pub state_dir: PathBuf,
    #[serde(default = "default_terraform_dir")]
    pub terraform_dir: PathBuf,
    #[serde(default = "default_packer_dir")]
    pub packer_dir: PathBuf,
    #[serde(default = "default_ansible_dir")]
    pub ansible_dir: PathBuf,
    #[serde(default = "default_download_dir")]
    pub download_dir: PathBuf,
}

// NOTE: Should probably be handled somewhere else, not malbox-config
impl PathConfig {
    pub fn new() -> Result<Self, ConfigError> {
        if let Some(proj_dirs) = directories::ProjectDirs::from("org", "malbox", "malbox") {
            Ok(Self {
                config_dir: proj_dirs.config_dir().to_path_buf(),
                cache_dir: proj_dirs.cache_dir().to_path_buf(),
                data_dir: proj_dirs.data_dir().to_path_buf(),
                state_dir: proj_dirs.state_dir().unwrap().to_path_buf(),
                terraform_dir: default_terraform_dir(),
                packer_dir: default_packer_dir(),
                ansible_dir: default_ansible_dir(),
                download_dir: default_download_dir(),
            })
        } else {
            Err(ConfigError::PathError {
                message: "Failed to determine XDG directories".into(),
                path: PathBuf::new(),
            })
        }
    }

    pub async fn ensure_dirs_exist(&self) -> Result<(), ConfigError> {
        for dir in [
            &self.config_dir,
            &self.cache_dir,
            &self.data_dir,
            &self.state_dir,
            &self.terraform_dir,
            &self.packer_dir,
            &self.ansible_dir,
            &self.download_dir,
        ] {
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(|e| ConfigError::PathError {
                    message: e.to_string(),
                    path: dir.clone(),
                })?;
        }
        Ok(())
    }
}

fn default_config_dir() -> PathBuf {
    if let Some(proj_dirs) = directories::ProjectDirs::from("org", "malbox", "malbox") {
        proj_dirs.config_dir().to_path_buf()
    } else {
        PathBuf::from("/etc/malbox")
    }
}

fn default_cache_dir() -> PathBuf {
    if let Some(proj_dirs) = directories::ProjectDirs::from("org", "malbox", "malbox") {
        proj_dirs.cache_dir().to_path_buf()
    } else {
        PathBuf::from("/var/cache/malbox")
    }
}

fn default_data_dir() -> PathBuf {
    if let Some(proj_dirs) = directories::ProjectDirs::from("org", "malbox", "malbox") {
        proj_dirs.data_dir().to_path_buf()
    } else {
        PathBuf::from("/var/lib/malbox")
    }
}

fn default_state_dir() -> PathBuf {
    if let Some(proj_dirs) = directories::ProjectDirs::from("org", "malbox", "malbox") {
        proj_dirs.state_dir().unwrap().to_path_buf()
    } else {
        PathBuf::from("/var/lib/malbox/state")
    }
}

fn default_terraform_dir() -> PathBuf {
    default_config_dir().join("infrastructure/terraform")
}

fn default_packer_dir() -> PathBuf {
    default_config_dir().join("infrastructure/packer")
}

fn default_ansible_dir() -> PathBuf {
    default_config_dir().join("infrastructure/ansible")
}

fn default_download_dir() -> PathBuf {
    default_config_dir().join("downloads")
}
