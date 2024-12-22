mod master;
mod plugin;

use super::types::PluginType;
pub use master::MasterCommunication;
pub use plugin::PluginCommunication;

// TODO: TBD
#[derive(Debug, Clone)]
pub struct PluginMessage {
    pub plugin_id: String,
    pub plugin_type: PluginType,
    pub data: Vec<u8>,
}
