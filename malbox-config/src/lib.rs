use malbox_storage::paths::Paths;
use std::path::{Path, PathBuf};
use tokio::sync::OnceCell;
use tracing::info;

pub mod core;
pub mod error;
pub mod machinery;
pub mod profiles;
pub mod templates;
pub mod types;

pub use crate::core::Config;
pub use crate::error::ConfigError;
pub use crate::types::*;

pub static CONFIG: OnceCell<Config> = OnceCell::const_new();

pub async fn load_config() -> Result<&'static Config, ConfigError> {
    CONFIG
        .get_or_try_init(|| async { load_config_internal().await })
        .await
}

async fn load_config_internal() -> Result<Config, ConfigError> {
    let xdg_config = Paths::new()?;

    let config_path = if let Some(path) = find_user_config(&xdg_config) {
        info!("Using user config at {}", path.display());
        path
    } else if let Some(path) = find_system_config() {
        info!("Using system config at {}", path.display());
        path
    } else {
        return Err(ConfigError::NotFound);
    };

    let content = tokio::fs::read_to_string(&config_path).await?;
    let mut config: Config = toml::from_str(&content).map_err(|e| ConfigError::Parse {
        file: config_path.display().to_string(),
        error: e.to_string(),
    })?;

    config.paths = xdg_config;
    config.paths.ensure_dirs_exist().await?;
    tracing::debug!("Paths: {:#?}, {:#?}", config.paths, config_path);
    load_provider_config(config_path.as_path(), &mut config).await?;

    Ok(config)
}

fn find_user_config(paths: &Paths) -> Option<PathBuf> {
    let user_config = paths.config_dir.join("malbox.toml");
    if user_config.exists() {
        Some(user_config)
    } else {
        None
    }
}

fn find_system_config() -> Option<PathBuf> {
    let system_config = PathBuf::from("/etc/malbox/malbox.toml");
    if system_config.exists() {
        Some(system_config)
    } else {
        None
    }
}

async fn load_provider_config(config_path: &Path, config: &mut Config) -> Result<(), ConfigError> {
    let provider_type = config.general.provider.to_string();
    let provider_config =
        machinery::MachineryConfig::load(&config.paths.config_dir, &provider_type).await?;

    config.machinery = provider_config;
    Ok(())
}
