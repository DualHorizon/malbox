//! Plugin registry system.
//!
//! This module manages the registry of available plugins
//! and their instances.

use crate::internal::communication::ipc::plugin;

use crate::internal::error::{PluginRegistryError, Result};
use discovery::PluginDiscovery;
use instance::PluginInstance;
use metadata::PluginManifest;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as AsyncRwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

mod discovery;
mod instance;
mod metadata;

/// Registry of all available plugins in the system.
///
/// The plugin registry maintains information about all plugins that
/// have been discovered, as well as which ones are currently loaded.
pub struct PluginRegistry {
    /// Path to the plugins directory.
    plugins_dir: PathBuf,
    /// Service for discovering plugins.
    discovery: PluginDiscovery,
    /// All discovered plugins mapped by ID.
    plugins: RwLock<HashMap<String, PluginManifest>>,
    /// All active plugin instances.
    instances: Arc<AsyncRwLock<HashMap<Uuid, PluginInstance>>>,
}

impl PluginRegistry {
    /// Create a new plugin registry.
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self {
            plugins_dir: plugins_dir.clone(),
            plugins: RwLock::new(HashMap::new()),
            discovery: PluginDiscovery::new(plugins_dir),
            instances: Arc::new(AsyncRwLock::new(HashMap::new())),
        }
    }

    /// Initialize the registry by discovering available plugins.
    pub async fn initialize(&self) -> Result<()> {
        let discovered = self.discovery.discover_plugins().await?;

        {
            let mut plugins = self.plugins.write().unwrap();
            for manifest in discovered {
                plugins.insert(manifest.id.clone(), manifest);
            }
        }

        tracing::info!(
            "Initialized plugin registry with {} plugins",
            self.plugins.read().unwrap().len()
        );
        Ok(())
    }

    /// Get all available plugins.
    pub fn get_plugins(&self) -> Vec<PluginManifest> {
        let plugins = self.plugins.read().unwrap();
        plugins.values().cloned().collect()
    }

    /// Find plugins that support a specific platform.
    pub fn find_plugins_for_platform(&self, platform: &GuestPlatform) -> Vec<PluginManifest> {
        let plugins = self.plugins.read().unwrap();
        plugins
            .values()
            .filter(|p| p.supports_platform(platform))
            .cloned()
            .collect()
    }

    /// Create a new plugin instance.
    pub async fn create_instance(&self, plugin_id: &str) -> Result<Uuid> {
        let manifest = {
            let plugins = self.plugins.read().unwrap();
            plugins
                .get(plugin_id)
                .cloned()
                .ok_or_else(|| PluginRegistryError::DiscoveryError(plugin_id.to_string()))?
        };

        let instance_id = Uuid::new_v4();

        let instance = PluginInstance::new(instance_id, manifest);

        {
            let mut instances = self.instances.write().await;
            instances.insert(instance_id, instance);
        }

        debug!(
            "Created plugin instance {} for plugin {}",
            instance_id, plugin_id
        );

        Ok(instance_id)
    }

    /// Get a plugin instance by ID.
    pub async fn get_instance(&self, id: Uuid) -> Option<PluginInstance> {
        let instances = self.instances.read().await;
        instances.get(&id).cloned()
    }

    /// Start a plugin instance.
    pub async fn start_instance(&self, id: Uuid) -> Result<()> {
        let mut instances = self.instances.write().await;

        if let Some(instance) = instances.get_mut(&id) {
            instance.start().await?;
            Ok(())
        } else {
            Err(PluginRegistryError::DiscoveryError(format!(
                "Instance {} not found",
                id
            )))?
        }
    }

    /// Stop a plugin instance.
    pub async fn stop_instance(&self, id: Uuid) -> Result<()> {
        let mut instances = self.instances.write().await;

        if let Some(instance) = instances.get_mut(&id) {
            instance.stop().await?;
            Ok(())
        } else {
            Err(PluginRegistryError::DiscoveryError(format!(
                "Instance {} not found",
                id
            )))?
        }
    }
}
