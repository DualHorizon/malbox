use crate::error::ConfigError;
use bon::Builder;
use directories::ProjectDirs;
use std::path::PathBuf;

#[derive(Debug, Clone, Builder)]
pub struct Paths {
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub data_dir: PathBuf,
    pub state_dir: PathBuf,
    #[builder(default = "self.config_dir.join(\"templates\")")]
    pub templates_dir: PathBuf,
    #[builder(default = "self.config_dir.join(\"terraform\")")]
    pub terraform_dir: PathBuf,
    #[builder(default = "self.config_dir.join(\"packer\")")]
    pub packer_dir: PathBuf,
    #[builder(default = "self.config_dir.join(\"ansible\")")]
    pub ansible_dir: PathBuf,
    #[builder(default = "self.cache_dir.join(\"images\")")]
    pub images_dir: PathBuf,
}

impl Paths {
    pub fn new() -> Result<Self, Error> {
        let proj_dirs = ProjectDirs::from("org", "malbox", "malbox")
            .ok_or_else(|| ConfigError::Xdg("Failed to determine XDG directories".into()))?;

        Ok(Self::builder()
            .config_dir(proj_dirs.config_dir().to_path_buf())
            .cache_dir(proj_dirs.cache_dir().to_path_buf())
            .data_dir(proj_dirs.data_dir().to_path_buf())
            .state_dir(proj_dirs.state_dir().to_path_buf())
            .build())
    }

    pub async fn ensure_dirs_exist(&self) -> Result<(), ConfigError> {
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
