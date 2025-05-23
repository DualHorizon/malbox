use malbox_database::repositories::machinery::MachinePlatform;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Configuration for worker instances.
///
/// Defines all aspects of worker behavior including task compatibility,
/// resource constraints, execution environment, and batch processing capabilities.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerConfig {
    /// Unique name for this worker configuration.
    pub name: String,
    /// Task types this worker is compatible with.
    /// None means it can handle all task types.
    #[serde(default)]
    pub compatible_tasks: Option<HashSet<String>>,
    /// Execution mode for this worker.
    pub execution_mode: ExecutionMode,
    /// Whether this worker supports batch processing.
    pub batch_processing: bool,
    /// Maximum of tasks in a single batch.
    #[serde(default = "default_max_batch_size")]
    pub max_batch_size: usize,
    /// Maximum time to wait for batch collection (milliseconds).
    #[serde(default = "default_batch_timeout")]
    pub batch_timeout_ms: u64,
    /// Maximum idle time before timeout (milliseconds).
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_ms: u64,
    /// Maximum concurrent tasks this worker can handle.
    #[serde(default = "default_max_concurrent_tasks")]
    pub max_concurrent_tasks: usize,
    /// Resource limits for this worker.
    #[serde(default)]
    pub resource_limits: ResourceLimits,
    /// Plugin restrictions for this worker.
    #[serde(default)]
    pub plugin_restrictions: PluginRestrictions,
    /// Platforms this worker can handle.
    #[serde(default)]
    pub compatible_platforms: HashSet<MachinePlatform>,
    /// Worker priority (higher numbers = higher priority for task assignment).
    #[serde(default = "default_priority")]
    pub priority: u8,
}

/// Resource limits for workers.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ResourceLimits {
    /// Memory limit in megabytes.
    pub memory_mb: Option<usize>,
    /// CPU core limit.
    pub cpu_cores: Option<usize>,
    /// Disk space limit in megabytes.
    pub disk_mb: Option<usize>,
    /// Network bandwidth limit in KB/s.
    pub network_kbps: Option<usize>,
}

/// Plugin access restrictions for workers.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PluginRestrictions {
    /// Plugins this worker is allowed to use.
    #[serde(default)]
    pub allow: HashSet<String>,
    /// Plugins this worker is not allowed to use.
    #[serde(default)]
    pub deny: HashSet<String>,
}

// Default value functions for serde
fn default_max_batch_size() -> usize {
    1
}
fn default_batch_timeout() -> u64 {
    500
}
fn default_idle_timeout() -> u64 {
    500
}
fn default_max_concurrent_tasks() -> usize {
    1
}
fn default_priority() -> u8 {
    5
}
