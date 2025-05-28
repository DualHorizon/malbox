//! IPC communication implementations using iceoryx2.

pub mod channel;
pub mod host;
pub mod plugin;

pub use channel::{Channel, ChannelConfig, ChannelRole};
pub use host::HostChannel;
pub use plugin::PluginChannel;

use crate::error::Result;
use crate::messages::ChannelMessage;

/// Generic communication channel trait
pub trait CommunicationChannel {
    /// Send a message through the channel
    fn send_message(&self, message: ChannelMessage, recipient: Option<&str>) -> Result<()>;

    /// Try to receive a message from the channel
    fn receive_message(&self) -> Result<Option<ChannelMessage>>;

    /// Check if the channel is initialized
    fn is_initialized(&self) -> bool;

    /// Get the channel identifier
    fn id(&self) -> &str;

    /// Close the channel
    fn close(&self) -> Result<()>;
}
