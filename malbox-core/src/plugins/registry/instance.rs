//! Plugin instance management.
//!
//! This module handles the lifecycle of individual plugin instances.

use crate::plugins::error::{PluginInstanceError, Result};
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn, Instrument};
use uuid::Uuid;

use super::metadata::PluginManifest;

/// Lifecycle state of a plugin instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceState {
    /// Instance has been created but not started.
    Created,

    /// Instance is starting.
    Starting,

    /// Instance is running.
    Running,

    /// Instance is stopping.
    Stopping,

    /// Instance has stopped.
    Stopped,

    /// Instance has failed.
    Failed,
}

/// A runnning instance of a plugin.
pub struct PluginInstance {
    /// Unique identifier for this instance.
    pub id: Uuid,

    /// Plugin manifest.
    pub manifest: PluginManifest,

    /// Current state of the instance.
    pub state: InstanceState,

    /// Process handle for host plugins.
    process: Option<Arc<RwLock<Child>>>,

    /// Current task ID being processed (if any).
    task_id: Option<Uuid>,
    // TODO:
    // - add comm channels
}

impl PluginInstance {
    /// Create a new plugin instance.
    pub fn new(id: Uuid, manifest: PluginManifest) -> Self {
        Self {
            id,
            manifest,
            state: InstanceState::Created,
            process: None,
            task_id: None,
        }
    }

    /// Assign this instance to a specific task.
    pub fn assign_task(&mut self, task_id: &str) {
        self.task_id = Some(Uuid::from_str(task_id).unwrap())
    }

    /// Get the current task assignment.
    pub fn task_id(&self) -> Option<&Uuid> {
        self.task_id.as_ref()
    }

    /// Start the plugin.
    pub async fn start(&mut self) -> Result<()> {
        // Create process environment
        let mut cmd = Command::new(&self.manifest.executable_path);

        match cmd.spawn() {
            Ok(child) => {
                self.process = Some(Arc::new(RwLock::new(child)));
                self.state = InstanceState::Running;

                info!("Started host plugin {} ({})", self.id, self.manifest.id);
                Ok(())
            }
            Err(e) => {
                self.state = InstanceState::Failed;
                Err(PluginInstanceError::ExecutionError(format!(
                    "Failed to start plugin process: {}",
                    e
                )))?
            }
        }
    }

    /// Stop the plugin.
    pub async fn stop(&mut self) -> Result<()> {
        if self.state != InstanceState::Running {
            // Already stopped or stopping
            if self.state == InstanceState::Stopped || self.state == InstanceState::Stopping {
                return Ok(());
            }

            return Err(PluginInstanceError::ExecutionError(format!(
                "Cannot stop plugin instance {} in state {:?}",
                self.id, self.state
            )))?;
        }

        self.state = InstanceState::Stopping;

        // For host plugins, terminate the process
        if let Some(process) = &self.process {
            let mut process = process.write().await;

            // Try to kill the process
            if let Err(e) = process.kill().await {
                error!("Failed to kill plugin process: {}", e);
                self.state = InstanceState::Failed;
                return Err(PluginInstanceError::ExecutionError(format!(
                    "Failed to kill plugin process: {}",
                    e
                )))?;
            }
        }

        self.state = InstanceState::Stopped;
        info!("Stopped plugin instance {} ({})", self.id, self.manifest.id);
        Ok(())
    }

    /// Check if the instance is running.
    pub async fn is_runnning(&self) -> bool {
        if self.state != InstanceState::Running {
            return false;
        }

        // For host plugins, check the process status
        if let Some(process) = &self.process {
            let mut process = process.write().await;

            match process.try_wait() {
                Ok(None) => true,     // Still running
                Ok(Some(_)) => false, // Exited
                Err(_) => false,
            }
        } else {
            // TODO:
            // - Guest plugins check
            true
        }
    }
}

impl Clone for PluginInstance {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            manifest: self.manifest.clone(),
            state: self.state,
            process: self.process.clone(),
            task_id: self.task_id.clone(),
        }
    }
}
