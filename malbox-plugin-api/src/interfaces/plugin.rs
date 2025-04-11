//! Plugin trait defition for malbox plugins.
//!
//! This module defines the core trait that all plugins must implement.

use super::errors::Result;
use async_trait::async_trait;
use semver::Version;
use serde::{Deserialize, Serialize};

/// Core plugin trait that all analysis plugins must implement.
///
/// Plugins are the primary extension mechanism in Malbox. Each plugin
/// provides specific analysis capabilties and can execute either on the
/// host system or within analysis VMs.
#[async_trait]
pub trait Plugin {
    /// Get the unique identifier for the plugin.
    ///
    /// This ID must be unique across all plugin types in the system and on the
    /// plugin marketplace.
    ///
    /// The format consists of a reverse-domain notation (e.g., "author.malbox.pe-analyzer").
    fn id(&self) -> &str; // TODO: Implement specific type for notation.

    /// Get the human-readable name of the plugin.
    fn name(&self) -> &str;

    /// Get the plugin author.
    /// NOTE: We might want to store more information about the plugin's author.
    fn author(&self) -> &str;

    /// Get a description of this plugin.
    fn description(&self) -> &str; // TODO: Define max length - MD support?

    /// Get the plugin version.
    ///
    /// Uses semantic versioning for consistency.
    fn version(&self) -> Version;

    /// Get the execution policy for the plugin.
    fn execution_policy(&self) -> ExecutionPolicy;

    /// Initialize the plugin.
    ///
    /// Called once when the plugin is first started.
    /// Use this method to set up resources / specifics needed for analysis,
    /// such as loading models, connecting to databases, etc.
    async fn initialize(&mut self) -> Result<()>;
}

/// Different execution contexts for plugins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionContext {
    /// Plugin executes on the host system.
    Host,

    /// Plugin executes within a guest VM.
    Guest {
        /// The platform the plugin is designed for.
        platform: GuestPlatform,
    },
    // Plugin has components that execute in both host and guest.
    //     Hybrid {
    //         /// The platforms this plugin supports.
    //         platform: GuestPlatform,
    //     },
}

/// Execution policies for plugins.
///
/// The execution policy defines and specifies concurrency rules
/// for how plugins execute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionPolicy {
    /// Plugin must be executed alone, no other plugins can run on the task.
    Exclusive,

    /// Plugin must be executed sequentially, one at a time.
    Sequential,

    /// Plugin can run in parallel with other plugins in the same group.
    ///
    /// A "group" is defined by a specified set (e.g. plugin capability tags, plugin IDs)
    Parallel(String), // TODO: NewType that wraps the different options instead of String.

    /// Plugin has no special execution policy.
    Unrestricted,
}

// Supported guest platforms for plugin execution.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuestPlatform {
    /// Microsoft Windows platform.
    Windows,

    /// Linux platform.
    Linux,
}

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
