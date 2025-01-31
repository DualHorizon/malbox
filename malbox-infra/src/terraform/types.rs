use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub name: String,
    pub working_dir: PathBuf,
    pub workspace: String,
    pub variables: HashMap<String, String>,
    pub backend_config: HashMap<String, String>,
    pub target: Option<String>,
    pub auto_approve: bool,
}
