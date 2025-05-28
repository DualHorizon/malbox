//! Type definitions for Plugin API v1.

// Work-in-progress
// TODO: Extensive capability list

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Different execution contexts for plugins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ExecutionContext {
    /// Plugin executes on the host system.
    Host,

    /// Plugin executes within a guest VM.
    Guest {
        /// The platform the plugin is designed for.
        platform: GuestPlatform,
    },
}

/// Execution policies for plugins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ExecutionPolicy {
    /// Plugin must be executed alone, no other plugins can run on the task.
    Exclusive,

    /// Plugin must be executed sequentially, one at a time.
    Sequential,

    /// Plugin can run in parallel with other plugins in the same group.
    Parallel(String),

    /// Plugin has no special execution policy.
    Unrestricted,
}

/// Supported guest platforms for plugin execution.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GuestPlatform {
    /// Microsoft Windows platform.
    Windows,

    /// Linux platform.
    Linux,
}

/// Plugin metadata for registration and discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin API version this plugin was built for.
    pub api_version: String,

    /// Plugin capabilities/features.
    pub capabilities: HashSet<PluginCapability>,

    /// Plugin tags for categorization.
    pub tags: HashSet<String>,

    /// Whether this plugin is considered stable.
    pub stable: bool,
}

/// Plugin capabilities that can be declared.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PluginCapability {
    /// Plugin can analyze files.
    FileAnalysis,

    /// Plugin can perform network analysis.
    NetworkAnalysis,

    /// Plugin can generate reports.
    Reporting,

    /// Plugin provides visualization.
    Visualization,

    /// Plugin can unpack/decode files.
    Unpacking,
}

// Display implementations
impl std::fmt::Display for ExecutionContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionContext::Host => write!(f, "host"),
            ExecutionContext::Guest { platform } => write!(f, "guest-{}", platform),
        }
    }
}

impl std::fmt::Display for GuestPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuestPlatform::Windows => write!(f, "windows"),
            GuestPlatform::Linux => write!(f, "linux"),
        }
    }
}
