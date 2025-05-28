//! Plugin trait definitions for v1 API.

use super::errors::Result;
use super::{ExecutionContext, ExecutionPolicy, PluginContext};
use crate::sealed::Sealed;
use async_trait::async_trait;
use semver::Version;

/// Core plugin trait that all analysis plugins must implement.
///
/// This trait is sealed to prevent external implementations while maintaining
/// API compatibility. Plugin authors should implement this trait directly.
///
/// # Example
///
/// ```rust
/// use malbox_core::{Plugin, PluginContext, Result, ExecutionContext, ExecutionPolicy};
/// use async_trait::async_trait;
/// use semver::Version;
///
/// struct MyPlugin {
///     version: Version,
/// }
///
/// #[async_trait]
/// impl Plugin for MyPlugin {
///     fn id(&self) -> &str { "com.example.my-plugin" }
///     fn name(&self) -> &str { "My Plugin" }
///     fn author(&self) -> &str { "Example Author" }
///     fn description(&self) -> &str { "An example plugin" }
///     fn version(&self) -> &Version { &self.version }
///     fn execution_context(&self) -> &ExecutionContext { &ExecutionContext::Host }
///     fn execution_policy(&self) -> &ExecutionPolicy { &ExecutionPolicy::Unrestricted }
///
///     async fn initialize(&mut self) -> Result<()> {
///         // Plugin initialization logic
///         Ok(())
///     }
///
///     async fn execute(&self, context: PluginContext) -> Result<()> {
///         // Plugin execution logic
///         println!("Processing file: {:?}", context.input_path);
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait Plugin: Sealed + Send + Sync {
    /// Get the unique identifier for the plugin.
    ///
    /// This should follow reverse domain notation (e.g., "com.example.my-plugin").
    fn id(&self) -> &str;
    /// Get the human-readable name of the plugin.
    fn name(&self) -> &str;
    /// Get the plugin author.
    fn author(&self) -> &str;
    /// Get a description of this plugin.
    fn description(&self) -> &str;
    /// Get the plugin version using semantic versioning.
    fn version(&self) -> &Version;
    /// Get the execution context for the plugin.
    fn execution_context(&self) -> &ExecutionContext;
    /// Get the execution policy for the plugin.
    fn execution_policy(&self) -> &ExecutionPolicy;
    /// Initialize the plugin.
    ///
    /// Called once when the plugin is first loaded. Use this to set up
    /// any resources, load configuration, etc.
    async fn initialize(&mut self) -> Result<()>;
    /// Execute the plugin with the given context.
    ///
    /// This is the main entry point for plugin execution. The context
    /// provides access to the input file, output directory, and configuration.
    async fn execute(&self, context: PluginContext) -> Result<()>;
    /// Shutdown the plugin gracefully.
    ///
    /// Called when the plugin is being unloaded. Use this to clean up
    /// resources, save state, etc. Default implementation does nothing.
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Marker trait for plugin implementations
///
/// This trait exists solely to implement the sealed trait pattern.
/// All plugin structs automatically implement this trait.
pub trait PluginImpl: Send + Sync {}

// Blanket implementation - any type can be a PluginImpl
impl<T> PluginImpl for T where T: Send + Sync {}
