use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Build error: {0}")]
    Build(String),
    #[error("Infrastructure error: {0}")]
    Infrastructure(String),
    #[error("Template error: {0}")]
    Template(String),
    #[error("Playbook error: {0}")]
    Playbook(String),
    #[error("Playbook YAML error: {0}")]
    PlaybookYaml(#[from] serde_yaml::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Invalid configuration: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, Error>;
