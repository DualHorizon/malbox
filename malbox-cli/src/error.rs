use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Configuration error: {0}")]
    Config(#[from] malbox_config::ConfigError),
    #[error("Builder error: {0}")]
    Builder(String),
    #[error("Infrastructure error: {0}")]
    Infrastructure(#[from] malbox_infra::Error),
    #[error("Deamon error: {0}")]
    Daemon(#[from] malbox_daemon::DaemonError),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Command failed: {0}")]
    CommandFailed(String),
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Serde YAML error: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error("Dialoguer error: {0}")]
    Dialoguer(#[from] dialoguer::Error),
}

pub type Result<T> = std::result::Result<T, CliError>;
