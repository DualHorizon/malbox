use serde::Deserialize;
use std::fs;
use std::process::exit;
use tokio::sync::OnceCell;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub http: Http,
    pub postgres: Postgres,
    pub debug: Debug,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Http {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Postgres {
    pub database_url: String,
    pub port: u16,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Debug {
    pub rust_log: String,
}

pub static CONFIG: OnceCell<Config> = OnceCell::const_new();

async fn init_config() -> Config {
    let file_name = "../malbox.toml";

    let contents = match fs::read_to_string(file_name) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Could not read file `{}`", file_name);
            exit(1);
        }
    };

    let data: Config = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(_) => {
            eprintln!("Unable to load data from file");
            exit(1);
        }
    };

    data
}

pub async fn load_config() -> &'static Config {
    CONFIG.get_or_init(init_config).await
}
