//! Plugin discovery system.
//!
//! This module handles finding and loading plugins from the filesystem.

use crate::plugins::{
    error::{PluginManagerError, PluginRegistryError, Result},
    registry::metadata::PluginManifest,
};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, error, info, warn};

/// Service for discovering plugins in the filesystem.
pub struct PluginDiscovery {
    /// Root directory for plugins.
    plugins_dir: PathBuf,
}

///
/// plugins/
// ├── my_plugin_1/
// │   ├── bin/
// │   │   └── my_plugin_1         # executable
// │   ├── config.toml             # default config metadata (plugin settings)
// │   └── manifest.json           # plugin metadata (version, capabilities, options exposed)
// ├── my_plugin_2/
// │   ├── bin/
// │   │   └── my_plugin_2
// │   ├── config.toml
// │   └── manifest.json

impl PluginDiscovery {
    /// Create a new plugin discovery service.
    pub fn new(plugins_dir: impl Into<PathBuf>) -> Self {
        Self {
            plugins_dir: plugins_dir.into(),
        }
    }

    /// Discover all plugins in the plugins directory.
    pub async fn discover_plugins(&self) -> Result<Vec<PluginManifest>> {
        let mut plugins = Vec::new();

        if !self.plugins_dir.exists() {
            return Err(PluginRegistryError::DiscoveryError(
                "Plugin directory does not exists. Did you run `malbox init`?".to_string(),
            ))?;
        }

        let mut entries = fs::read_dir(&self.plugins_dir).await.map_err(|e| {
            PluginRegistryError::DiscoveryError(format!("Failed to read plugins directory: {}", e))
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            PluginRegistryError::DiscoveryError(format!("Failed to read directory entry: {}", e))
        })? {
            let path = entry.path();

            if path.is_dir() {
                if let Err(e) = self.process_plugin_directory(&path, &mut plugins).await {
                    error!("Error processing plugin directory {:?}: {}", path, e);
                }
            }
        }

        info!("Discovered {} plugins", plugins.len());
        Ok(plugins)
    }

    /// Process a plugin directory.
    async fn process_plugin_directory(
        &self,
        dir: &Path,
        plugins: &mut Vec<PluginManifest>,
    ) -> Result<()> {
        debug!("Processing plugin directory: {:?}", dir);

        let manifest_path = dir.join("manifest.json");
        if !manifest_path.exists() {
            error!("No manifest.json found in {:?}", dir);
            return Ok(());
        }

        match PluginManifest::from_json_file(&manifest_path).await {
            Ok(manifest) => {
                debug!(
                    "Loaded plugin manifest for {}: {}",
                    manifest.id, manifest.name
                );
                if let Err(e) = manifest.validate() {
                    error!("Invalid plugin manifest in {:?}: {}", manifest_path, e);
                    return Ok(());
                }
                plugins.push(manifest);
            }
            Err(e) => {
                warn!(
                    "Failed to laod plugin manifest from {:?}: {}",
                    manifest_path, e
                )
            }
        }

        Ok(())
    }
}
