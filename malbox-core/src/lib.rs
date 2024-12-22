pub mod communication;
pub mod manager;
pub mod plugin;
pub mod registry;
pub mod types;

pub use manager::PluginManager;
pub use plugin::{Plugin, PluginRequirements};
pub use registry::PluginRegistry;
pub use types::{ExecutionMode, PluginEvent, PluginType};
