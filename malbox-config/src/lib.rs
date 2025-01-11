use anyhow::Context;
use std::path::PathBuf;
use tokio::sync::OnceCell;

pub mod config;
mod error;
pub mod machinery;
mod profiles;
mod shared_types;
mod templates;

pub use config::Config;
pub use error::ConfigError;
pub use machinery::MachineryConfig;
pub use profiles::ProfileConfig;
pub use templates::TemplateConfig;

pub static CONFIG: OnceCell<Config> = OnceCell::const_new();

pub async fn load_config() -> anyhow::Result<&'static Config> {
    CONFIG
        .get_or_try_init(|| async { load_config_internal().await })
        .await
}

async fn load_config_internal() -> anyhow::Result<Config> {
    let config_path = find_config_path()?;
    let config_str = tokio::fs::read_to_string(&config_path)
        .await
        .context("Failed to read config file")?;

    let config_dir = config_path
        .parent()
        .context("Failed to get config directory")?;

    let malbox: config::MalboxConfig =
        toml::from_str(&config_str).context("Failed to parse malbox config")?;

    let machinery = MachineryConfig::load(&config_dir, &malbox.infrastructure.provider)
        .await
        .context("Failed to load machinery config")?;

    let profiles = ProfileConfig::load(&config_dir)
        .await
        .context("Failed to load profiles config")?;

    let templates = TemplateConfig::load(&config_dir)
        .await
        .context("Failed to load templates config")?;

    Ok(Config {
        malbox,
        machinery,
        profiles,
        templates,
    })
}

fn find_config_path() -> anyhow::Result<PathBuf> {
    let paths = vec![
        PathBuf::from("./malbox.toml"),
        PathBuf::from("./configuration/malbox.toml"),
        PathBuf::from("/etc/malbox/malbox.toml"),
    ];

    for path in paths {
        if path.exists() {
            return Ok(path);
        }
    }

    Err(ConfigError::NotFound.into())
}
