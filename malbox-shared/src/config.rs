use serde::Deserialize;
use std::process::exit;
use tokio::sync::OnceCell;

pub mod machinery;
pub mod malbox;

use machinery::MachineryConfig;
use malbox::MalboxConfig;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub malbox: MalboxConfig,
    pub machinery: MachineryConfig,
}

pub static CONFIG: OnceCell<Config> = OnceCell::const_new();

pub async fn load_config() -> &'static Config {
    let malbox_config = match malbox::load_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load malbox config: {}", e);
            exit(1);
        }
    };

    let machinery_config = match machinery::load_config(&malbox_config.machinery.hypervisor) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load machinery config: {}", e);
            exit(1);
        }
    };

    let config = Config {
        malbox: malbox_config,
        machinery: machinery_config,
    };

    CONFIG.get_or_init(|| async { config }).await
}
