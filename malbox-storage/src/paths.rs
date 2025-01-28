use crate::error::StorageError;
use bon::Builder;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use std::path::PathBuf;

// NOTE:
// This needs to be rewritten, Paths is a configuration interface,
// found in malbox.toml. This shouldn't be here.
// We should make a Builder that takes Path from malbox-config
// initialization instead.

#[serde_inline_default]
#[derive(Debug, Clone, Builder, Serialize, Deserialize)]
pub struct Paths {
    // This is config_dir is useless, and doesn't make any sense.
    // It would either be ~/.config/malbox/malbox.toml or /etc/malbox/malbox.toml
    // The user can also manually pass its config location via malbox-cli.
    #[serde_inline_default(PathBuf::from("~/.config/malbox/"))]
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub data_dir: PathBuf,
    pub state_dir: PathBuf,
    #[builder(default = "self.config_dir.join(\"templates\")".into())]
    pub templates_dir: PathBuf,
    #[builder(default = "self.config_dir.join(\"terraform\")".into())]
    pub terraform_dir: PathBuf,
    #[builder(default = "self.config_dir.join(\"packer\")".into())]
    pub packer_dir: PathBuf,
    #[builder(default = "self.config_dir.join(\"ansible\")".into())]
    pub ansible_dir: PathBuf,
    #[builder(default = "self.cache_dir.join(\"images\")".into())]
    pub images_dir: PathBuf,
}

impl Paths {
    pub fn new() -> Result<Self, StorageError> {
        let proj_dirs = ProjectDirs::from("org", "malbox", "malbox")
            .ok_or_else(|| StorageError::Xdg("Failed to determine XDG directories".into()))?;

        Ok(Self::builder()
            .config_dir(proj_dirs.config_dir().to_path_buf())
            .cache_dir(proj_dirs.cache_dir().to_path_buf())
            .data_dir(proj_dirs.data_dir().to_path_buf())
            .state_dir(proj_dirs.state_dir().unwrap().to_path_buf())
            .build())
    }

    pub async fn ensure_dirs_exist(&self) -> Result<(), StorageError> {
        for dir in [
            &self.config_dir,
            &self.cache_dir,
            &self.data_dir,
            &self.state_dir,
            &self.templates_dir,
            &self.terraform_dir,
            &self.packer_dir,
            &self.ansible_dir,
            &self.images_dir,
        ] {
            tokio::fs::create_dir_all(dir).await?;
        }
        Ok(())
    }
}
