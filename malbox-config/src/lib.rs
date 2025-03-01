use std::path::PathBuf;
use tokio::sync::OnceCell;
use tracing::info;

pub mod core;
pub mod error;
pub mod machinery;
pub mod profiles;
pub mod storage;
pub mod templates;
pub mod types;

pub use core::Config;
pub use error::ConfigError;
pub use storage::PathConfig;
pub use types::*;

pub static CONFIG: OnceCell<Config> = OnceCell::const_new();

pub async fn load_config() -> Result<&'static Config, ConfigError> {
    CONFIG
        .get_or_try_init(|| async { load_config_internal().await })
        .await
}

async fn load_config_internal() -> Result<Config, ConfigError> {
    let paths = PathConfig::new()?;

    let config_path = if let Some(path) = find_user_config(&paths) {
        info!("Using user config at {}", path.display());
        path
    } else if let Some(path) = find_system_config() {
        info!("Using system config at {}", path.display());
        path
    } else {
        return Err(ConfigError::NotFound);
    };

    let content =
        tokio::fs::read_to_string(&config_path)
            .await
            .map_err(|e| ConfigError::Parse {
                file: config_path.display().to_string(),
                error: e.to_string(),
            })?;

    let mut config: Config = toml::from_str(&content).map_err(|e| ConfigError::Parse {
        file: config_path.display().to_string(),
        error: e.to_string(),
    })?;

    config.paths = paths;

    config.paths.ensure_dirs_exist().await?;
    tracing::debug!("Using paths: {:#?}", config.paths);

    load_provider_config(&mut config).await?;

    Ok(config)
}

fn find_user_config(paths: &PathConfig) -> Option<PathBuf> {
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

async fn load_provider_config(config: &mut Config) -> Result<(), ConfigError> {
    let provider_type = config.general.provider.to_string();
    let provider_config =
        machinery::MachineryConfig::load(&config.paths.terraform_dir, &provider_type).await?;
    config.machinery = provider_config;
    Ok(())
}
