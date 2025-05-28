//! Plugin API Version 1.0
//!
//! This is the current stable plugin API. All plugins should implement
//! the traits defined in this module.

pub mod context;
pub mod errors;
pub mod plugin;
pub mod types;

pub use context::PluginContext;
pub use errors::{PluginError, Result};
pub use plugin::{Plugin, PluginImpl};
pub use types::{
    ExecutionContext, ExecutionPolicy, GuestPlatform, PluginCapability, PluginMetadata,
};

pub const VERSION: &str = "1.0.0";
