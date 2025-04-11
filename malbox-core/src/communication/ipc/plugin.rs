//! Iceoryx2 communication for plugins.
//!
//! This module implements zero-copy IPC between plugins and the host system.

use crate::communication::common::{
    ChannelMessage, CommandMessage, CommunicationChannel, EventMessage, MessagePayload,
    MessageType, ResultMessage, TaskMessage,
};
use crate::errors::{InternalError, Result};
use iceoryx2::node::{Node, NodeBuilder};
use iceoryx2::port::publisher::Publisher;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::prelude::*;
use std::fmt::format;
use std::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

mod publisher;
mod subscriber;

use publisher::PluginPublisher;
use subscriber::PluginSubscriber;

/// Plugin IPC channel using iceoryx2.
pub struct PluginIpc {
    /// Iceoryx2 node.
    pub node: Option<Node<ipc::Service>>,

    /// Publisher for sending messages to the host.
    publisher: Option<PluginPublisher>,

    /// Subscriber for receiving messages from the host.
    subscriber: Option<PluginSubscriber>,

    /// Plugin instance ID.
    instance_id: String,

    /// Flag indicating.
    is_initialized: RwLock<bool>,
}

impl PluginIpc {
    /// Create a new plugin IPC instance.
    pub fn new() -> Result<Self> {
        Ok(Self {
            node: None,
            publisher: None,
            subscriber: None,
            is_initialized: RwLock::new(false),
            instance_id: Uuid::new_v4().to_string(),
        })
    }

    /// Initialized the IPC channel.
    pub fn initialize(&mut self) -> Result<()> {
        let node_name = format!("plugin-node-{}", Uuid::new_v4());

        let node = NodeBuilder::new()
            .name(&node_name.as_str().try_into().unwrap())
            .create()
            .map_err(|e| {
                InternalError::CommunicationError(format!("Failed to create iceoryx2 node: {}", e))
            })?;

        let publisher = PluginPublisher::new(&node)?;
        let subscriber = PluginSubscriber::new(&node)?;

        self.node = Some(node);
        self.publisher = Some(publisher);
        self.subscriber = Some(subscriber);
        self.is_initialized = RwLock::new(true);

        info!("Initialized iceoryx2 IPC plugin channel: {}", node_name);

        Ok(())
    }
}

impl CommunicationChannel for PluginIpc {
    fn send_message(&self, message: ChannelMessage, _plugin_id: Option<&str>) -> Result<()> {
        if !self.is_initialized() {
            return Err(InternalError::CommunicationError(
                "Plugin IPC is not initialized".to_string(),
            ))?;
        }

        let publisher = self.publisher.as_ref().ok_or_else(|| {
            InternalError::CommunicationError("Plugin publisher is not initialized".to_string())
        })?;

        match message {
            ChannelMessage::Result(result) => {
                publisher.send_result(result)?;
            }
            ChannelMessage::Event(event) => {
                publisher.send_event(event)?;
            }
            ChannelMessage::Registration(plugin_id) => {
                // For iceoryx2, registration is implicit when creating the node
                debug!("Registered plugin {} with iceoryx2", plugin_id);
            }
            _ => {
                return Err(InternalError::CommunicationError(format!(
                    "Unsupported message type for plugin IPC: {:?}",
                    message
                )));
            }
        }

        Ok(())
    }

    fn receive_message(&self) -> Result<Option<ChannelMessage>> {
        if !self.is_initialized() {
            return Err(InternalError::CommunicationError(
                "Plugin IPC is not initialized".to_string(),
            ))?;
        }

        let subscriber = self.subscriber.as_ref().ok_or_else(|| {
            InternalError::CommunicationError("Plugin subscriber is not initialized".to_string())
        })?;

        if let Some(task) = subscriber.receive_task()? {
            return Ok(Some(ChannelMessage::Task(task)));
        }

        if let Some(command) = subscriber.receive_command()? {
            return Ok(Some(ChannelMessage::Command(command)));
        }

        Ok(None)
    }

    fn close(&self) -> Result<()> {
        todo!()
    }

    fn is_initialized(&self) -> bool {
        *self.is_initialized.read().unwrap()
    }

    fn id(&self) -> &str {
        "123"
    }
}
