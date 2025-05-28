//! Plugin metadata definitions.
//!
//! This module defines the metadata/config format for plugins
//! and provides functionality for loading and validating plugin information.
//!
//! NOTE - TODO:
//! Either we will generate the metdata json file when we compile the plugin -
//! or we initilialize it as a "setup" / "init" stub when the plugin is launched
//! for the first time. We should consider performance costs and plugin size, since
//! a plugin would contain extra logic and dependencies for generating such files.

use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Unique identifier for the plugin.
    /// TODO: The format consists of a reverse-domain notation (e.g., "author.malbox.pe-analyzer").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Plugin author.
    pub author: String,
    /// Plugin version (semver).
    pub version: Version,
    /// Execution context.
    pub execution_context: ExecutionContext,
    /// Execution policy.
    pub execution_policy: ExecutionPolicy,
    /// Path to the executable.
    #[serde(skip)]
    pub executable_path: PathBuf,
    // TODO: Other fields...
}

impl PluginManifest {
    /// Load plugin manifest from a JSON file.
    pub async fn from_json_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).await.map_err(|e| {
            PluginRegistryError::IoError(format!("Could not read plugin manifest file: {}", e))
        })?;
        let mut manifest: Self = serde_json::from_str(&content).map_err(|e| {
            PluginRegistryError::SerializationError(format!(
                "Could not deserialize plugin manifest JSON: {}",
                e
            ))
        })?;

        if let Some(parent) = path.parent() {
            let parent_name = parent.file_name().ok_or_else(|| {
                PluginRegistryError::IoError("Could not get plugin directory name".to_string())
            })?;

            let executable = parent.join("bin").join(parent_name);

            manifest.executable_path = executable;
        } else {
            return Err(PluginRegistryError::IoError(format!(
                "Invalid manifest file path: {}",
                std::io::ErrorKind::InvalidInput
            )))?;
        }

        Ok(manifest)
    }

    pub fn validate(&self) -> Result<()> {
        if !self.executable_path.exists() {
            return Err(PluginRegistryError::DiscoveryError(format!(
                "Plugin executable not found at {:?}",
                self.executable_path
            )))?;
        }

        if !self.validate_id() {
            return Err(PluginRegistryError::DiscoveryError(format!(
                "Plugin ID doesn't properly follow convention"
            )))?;
        }

        Ok(())
    }

    fn validate_id(&self) -> bool {
        let mut parts = self.id.split('.');

        match (parts.next(), parts.next(), parts.next()) {
            (Some(author), Some(context), Some(_name)) => {
                author == self.author.to_lowercase()
                    && context == self.execution_context.to_string().to_lowercase()
            }
            _ => false,
        }
    }

    /// Check if the plugin supports a specific platform.
    pub fn supports_platform(&self, platform: &GuestPlatform) -> bool {
        match &self.execution_context {
            ExecutionContext::Host => true,
            ExecutionContext::Guest { platform: p } => p == platform,
        }
    }
}
