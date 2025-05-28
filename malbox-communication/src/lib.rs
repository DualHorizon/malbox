//! Malbox communication library.
//!
//! This crate provides zero-copy IPC communication primitives for Malbox
//! using iceoryx2. It supports both host-side and plugin-side communication
//! with a generic, reusable architecture.

pub mod error;
pub mod ipc;
pub mod messages;

pub use error::{CommunicationError, Result};
pub use ipc::{host::HostChannel, plugin::PluginChannel, Channel, ChannelConfig, ChannelRole};
pub use messages::{
    ChannelMessage, CommandMessage, EventMessage, MessagePayload, MessageType, ResultMessage,
    TaskMessage,
};
