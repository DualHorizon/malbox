//! Plugin execution context for API v1.

use std::collections::HashMap;
use std::path::PathBuf;

/// Context provided to plugins during execution.
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// Unique task ID for this execution.
    pub task_id: String,
    /// Input data/file path.
    pub input_path: PathBuf,
    /// Output directory for results.
    pub output_dir: PathBuf,
    /// Plugin-specific configuration.
    pub config: HashMap<String, String>,
    /// Execution timeout in seconds.
    pub timeout_seconds: u64,
    /// Available memory in MB
    pub memory_limit_mb: Option<u64>,
    /// Whether network access is allowed.
    pub network_enabled: bool,
}

impl PluginContext {
    pub fn new(task_id: String, input_path: PathBuf, output_dir: PathBuf) -> Self {
        Self {
            task_id,
            input_path,
            output_dir,
            config: HashMap::new(),
            timeout_seconds: 300, // 5 minutes default
            memory_limit_mb: None,
            network_enabled: false,
        }
    }

    pub fn with_config(mut self, config: HashMap<String, String>) -> Self {
        self.config = config;
        self
    }

    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    pub fn with_memory_limit(mut self, memory_mb: u64) -> Self {
        self.memory_limit_mb = Some(memory_mb);
        self
    }

    pub fn with_network_access(mut self, enabled: bool) -> Self {
        self.network_enabled = enabled;
        self
    }
}
