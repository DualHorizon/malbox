//! Malbox core - plugin system, communication infrastructure and public API.
//!
//! This crate provides the plugin API for Malbox. This is the **only** crate
//! that plugin authors need to depend on.

// ============================================================================
// PUBLIC API
// ============================================================================

pub mod api;
pub mod sealed;
// pub mod version;

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

// pub use version::{
//     is_api_compatible, supported_api_versions, API_VERSION, CORE_VERSION, PROTOCOL_VERSION,
// };

// ============================================================================
// INTERNAL API - Only for malbox-core internal use and other malbox-* crates
// ============================================================================

#[doc(hidden)]
pub mod internal_api {
    //! Internal implementation details.
    //!
    //! **This module is not part of the public API and may change without notice.**
    //! External crates should not depend on anything in this module.

    pub use crate::internal::*;
}

// Conditional feature-gated exports for advanced users
#[cfg(feature = "plugin-management")]
pub mod management {
    //! Plugin management utilities for malbox runtime.
    //!
    //! This module is only available when the `plugin-management` feature is enabled.
    //! It contains functionality needed by the malbox runtime but not by plugin authors.

    pub use crate::internal_api::communication::{ChannelMessage, CommunicationChannel};
    pub use crate::internal_api::plugin::{PluginManager, PluginRegistry};
}

// All internal implementation - not exported publicly
mod internal {
    pub mod errors;
    pub mod plugin;
}
