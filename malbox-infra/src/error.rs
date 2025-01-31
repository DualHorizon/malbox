use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Packer error: {0}")]
    Packer(String),
    #[error("Template error: {0}")]
    Template(String),
    #[error("Variable error: {0}")]
    Variable(String),
    #[error("Ansible error: {0}")]
    Ansible(String),
    #[error("Terraform error: {0}")]
    Terraform(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HCL parse error: {0}")]
    HclParse(#[from] hcl::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
