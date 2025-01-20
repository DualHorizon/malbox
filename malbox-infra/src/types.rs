use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Platform {
    Windows,
    Linux,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderConfig {
    pub platform: Platform,
    pub name: String,
    pub iso: Option<String>,
    pub force: bool,
    pub working_dir: Option<PathBuf>,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefineConfig {
    pub base: String,
    pub name: String,
    pub playbook: String,
    pub force: bool,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub platform: Platform,
    pub description: String,
    pub base: Option<String>,
    pub variables: HashMap<String, String>,
    pub playbooks: Vec<String>,
}
