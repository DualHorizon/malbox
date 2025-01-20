pub mod communication;
pub mod manager;
pub mod plugin;
pub mod registry;
pub mod types;
pub mod python_plugin;
pub mod node_plugin;

pub use manager::PluginManager;
pub use plugin::{Plugin, PluginRequirements};
pub use registry::PluginRegistry;
pub use types::{ExecutionMode, PluginEvent, PluginType};
pub use python_plugin::*;
pub use node_plugin::*;
