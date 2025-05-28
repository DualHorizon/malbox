//! Internal implementation details.
//!
//! This module contains all the internal implementation that is not part
//! of the public plugin API. This includes:
//!
//! - Communication infrastructure
//! - Plugin management systems
//! - Protocol handling
//! - Registry and discovery
//!
//! **Warning**: This module is not part of the stable public API and may
//! change at any time. External crates should not depend on anything here.

pub mod errors;
pub mod plugin;

pub use errors::InternalError;
pub use plugin::{PluginManager, PluginRegistry};
