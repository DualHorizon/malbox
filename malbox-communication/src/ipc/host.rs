//! Host-side IPC channel implementation.

use super::channel::{Channel, ChannelConfig, ChannelRole};
use super::CommunicationChannel;
use crate::error::Result;
use crate::messages::{ChannelMessage, MessagePayload, MessageType};

/// Marker type for host channels.
pub struct HostRole;

/// Host-side communication channel.
pub struct HostChannel {
    inner: Channel<HostRole>,
}

impl HostChannel {
    pub fn new() -> Self {
        let config = ChannelConfig {
            role: ChannelRole::Host,
            node_name: "malbox-host".to_string(),
            service_prefix: "malbox".to_string(),
        };

        Self {
            inner: Channel::new(config),
        }
    }

    pub fn with_config(config: ChannelConfig) -> Self {
        Self {
            inner: Channel::new(config),
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        self.inner.initialize()?;

        // Create host-specific publishers and subscribers
        self.inner.create_publisher("tasks")?;
        self.inner.create_publisher("commands")?;
        self.inner.create_subscriber("results")?;
        self.inner.create_subscriber("events")?;

        Ok(())
    }

    pub fn send_task(&self, task: crate::messages::TaskMessage, plugin_id: &str) -> Result<()> {
        let payload =
            MessagePayload::new(MessageType::Task, "host", plugin_id)?.with_task(&task)?;

        self.inner.send_message(payload)
    }

    pub fn send_command(
        &self,
        command: crate::messages::CommandMessage,
        plugin_id: &str,
    ) -> Result<()> {
        let payload =
            MessagePayload::new(MessageType::Command, "host", plugin_id)?.with_command(&command)?;

        self.inner.send_message(payload)
    }

    pub fn receive_result(&self) -> Result<Option<crate::messages::ResultMessage>> {
        if let Some(payload) = self.inner.receive_message()? {
            if payload.message_type == MessageType::Result {
                return Ok(Some(payload.to_result()?));
            }
        }
        Ok(None)
    }

    pub fn receive_event(&self) -> Result<Option<crate::messages::EventMessage>> {
        if let Some(payload) = self.inner.receive_message()? {
            if payload.message_type == MessageType::Event {
                return Ok(Some(payload.to_event()?));
            }
        }
        Ok(None)
    }
}

impl CommunicationChannel for HostChannel {
    fn send_message(&self, message: ChannelMessage, recipient: Option<&str>) -> Result<()> {
        let recipient = recipient.unwrap_or("broadcast");

        match message {
            ChannelMessage::Task(task) => self.send_task(task, recipient),
            ChannelMessage::Command(command) => self.send_command(command, recipient),
            _ => Err(crate::error::CommunicationError::SendFailed(
                "Unsupported message type for host".to_string(),
            )),
        }
    }

    fn receive_message(&self) -> Result<Option<ChannelMessage>> {
        if let Some(result) = self.receive_result()? {
            return Ok(Some(ChannelMessage::Result(result)));
        }

        if let Some(event) = self.receive_event()? {
            return Ok(Some(ChannelMessage::Event(event)));
        }

        Ok(None)
    }

    fn is_initialized(&self) -> bool {
        self.inner.is_initialized()
    }

    fn id(&self) -> &str {
        self.inner.id()
    }

    fn close(&self) -> Result<()> {
        self.inner.close()
    }
}
