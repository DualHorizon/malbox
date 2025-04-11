//! Iceoryx2 communication for host plugins.
//!
//! This module implements zero-copy IPC between host plugins and the core system.

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

mod publisher;
mod subscriber;

use publisher::HostPublisher;
use subscriber::HostSubscriber;

/// Host/Core communication channel using iceoryx2.
pub struct HostIpc {
    /// Iceoryx2 node.
    pub node: Option<Node<ipc::Service>>,

    /// Publisher for sending messages to plugins.
    publisher: Option<HostPublisher>,

    /// Subscriber for receiving messages from plugins.
    subscriber: Option<HostSubscriber>,

    /// Flag indicating if it is properly initialized.
    is_initialized: RwLock<bool>,
}

impl HostIpc {
    /// Create a new host IPC channel.
    pub fn new() -> Result<Self> {
        Ok(Self {
            node: None,
            publisher: None,
            subscriber: None,
            is_initialized: RwLock::new(false),
        })
    }

    /// Initialize the IPC channel.
    pub fn initialize(&mut self) -> Result<()> {
        let node = NodeBuilder::new()
            .name(&"host-node".try_into().unwrap())
            .create()
            .map_err(|e| {
                InternalError::CommunicationError(format!("Failed to create iceoryx2 node: {}", e))
            })?;

        let publisher = HostPublisher::new(&node)?;
        let subscriber = HostSubscriber::new(&node)?;

        self.node = Some(node);
        self.publisher = Some(publisher);
        self.subscriber = Some(subscriber);
        self.is_initialized = RwLock::new(true);

        info!("Initialized iceoryx2 IPC core channel");

        Ok(())
    }
}

impl CommunicationChannel for HostIpc {
    fn send_message(&self, message: ChannelMessage, plugin_id: Option<&str>) -> Result<()> {
        if !self.is_initialized() {
            return Err(InternalError::CommunicationError(
                "Host IPC is not initialized".to_string(),
            ))?;
        }

        let publisher = self.publisher.as_ref().ok_or_else(|| {
            InternalError::CommunicationError("Host publisher is not initialized".to_string())
        })?;

        match message {
            ChannelMessage::Task(task) => {
                publisher.send_task(task, plugin_id.unwrap())?;
            }
            ChannelMessage::Command(command) => {
                publisher.send_command(command, plugin_id.unwrap())?;
            }
            _ => {
                return Err(InternalError::CommunicationError(format!(
                    "Unsupported message type for host IPC: {:?}",
                    message
                )));
            }
        }

        Ok(())
    }

    fn receive_message(&self) -> Result<Option<ChannelMessage>> {
        if !self.is_initialized() {
            return Err(InternalError::CommunicationError(
                "Host IPC is not initialized".to_string(),
            ))?;
        }

        let subscriber = self.subscriber.as_ref().ok_or_else(|| {
            InternalError::CommunicationError("Host subscriber is not initialized".to_string())
        })?;

        if let Some(result) = subscriber.receive_result()? {
            return Ok(Some(ChannelMessage::Result(result)));
        }

        if let Some(event) = subscriber.receive_event()? {
            return Ok(Some(ChannelMessage::Event(event)));
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
