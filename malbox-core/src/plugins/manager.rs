//! Plugin manager for the Malbox worker runtime.
//!
//! This module provides a higher-level API for working with plugins,
//! and profiles.

use crate::communication::ipc::host::HostIpc;

use super::error::{PluginManagerError, Result};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info, warn};

use super::registry::PluginRegistry;

/// High-level manager for plugin operations.
pub struct PluginManager {
    /// Plugin registry.
    registry: Arc<PluginRegistry>,
    host_ipc: Arc<RwLock<HostIpc>>,
}

impl PluginManager {
    /// Create a new plugin manager.
    pub fn new(plugins_dir: PathBuf) -> Self {
        let registry = Arc::new(PluginRegistry::new(plugins_dir));
        let host_ipc = Arc::new(RwLock::new(HostIpc::new().unwrap()));

        Self { registry, host_ipc }
    }

    /// Initialize the plugin system.
    pub async fn initialize(&mut self) -> Result<()> {
        self.registry.initialize().await?;
        self.host_ipc.write().unwrap().initialize()?;

        Ok(())
    }

    /// Get the plugin registry.
    pub fn registry(&self) -> &PluginRegistry {
        &self.registry
    }
}
