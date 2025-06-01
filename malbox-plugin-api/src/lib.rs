//! Malbox plugin API - public plugin API.
//!
//! This crate provides the plugin API for Malbox. This is the **only** crate
//! that plugin authors need to depend on.

pub mod api;
pub mod error;
pub mod sealed;

pub use api::v1::{
    // Types
    ExecutionContext,
    ExecutionPolicy,
    GuestPlatform,
    // Core traits
    Plugin,
    PluginCapability,
    // Context and results
    PluginContext,
    // Errors
    PluginError,
    PluginMetadata,
    Result,
};
