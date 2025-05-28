//! Plugin-side IPC channel implementation.

use super::channel::{Channel, ChannelConfig, ChannelRole};
use super::CommunicationChannel;
use crate::error::Result;
use crate::messages::{ChannelMessage, MessagePayload, MessageType};
use uuid::Uuid;

/// Marker type for plugin channels.
pub struct PluginRole;

/// Plugin-side communication channel.
pub struct PluginChannel {
    inner: Channel<PluginRole>,
    plugin_id: String,
}

impl PluginChannel {
    pub fn new() -> Self {
        let plugin_id = format!("plugin-{}", Uuid::new_v4());
        let config = ChannelConfig {
            role: ChannelRole::Plugin,
            node_name: format!("malbox-{}", plugin_id),
            service_prefix: "malbox".to_string(),
        };

        Self {
            inner: Channel::new(config),
            plugin_id,
        }
    }

    pub fn with_plugin_id(plugin_id: String) -> Self {
        let config = ChannelConfig {
            role: ChannelRole::Plugin,
            node_name: format!("malbox-{}", plugin_id),
            service_prefix: "malbox".to_string(),
        };

        Self {
            inner: Channel::new(config),
            plugin_id,
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        self.inner.initialize()?;

        // Create plugin-specific publishers and subscribers
        self.inner.create_publisher("results")?;
        self.inner.create_publisher("events")?;
        self.inner.create_subscriber("tasks")?;
        self.inner.create_subscriber("commands")?;

        Ok(())
    }

    pub fn send_result(&self, result: crate::messages::ResultMessage) -> Result<()> {
        let payload = MessagePayload::new(MessageType::Result, &self.plugin_id, "host")?
            .with_result(&result)?;

        self.inner.send_message(payload)
    }

    pub fn send_event(&self, event: crate::messages::EventMessage) -> Result<()> {
        let payload =
            MessagePayload::new(MessageType::Event, &self.plugin_id, "host")?.with_event(&event)?;

        self.inner.send_message(payload)
    }

    pub fn receive_task(&self) -> Result<Option<crate::messages::TaskMessage>> {
        if let Some(payload) = self.inner.receive_message()? {
            if payload.message_type == MessageType::Task {
                return Ok(Some(payload.to_task()?));
            }
        }
        Ok(None)
    }

    pub fn receive_command(&self) -> Result<Option<crate::messages::CommandMessage>> {
        if let Some(payload) = self.inner.receive_message()? {
            if payload.message_type == MessageType::Command {
                return Ok(Some(payload.to_command()?));
            }
        }
        Ok(None)
    }

    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }
}

impl CommunicationChannel for PluginChannel {
    fn send_message(&self, message: ChannelMessage, _recipient: Option<&str>) -> Result<()> {
        match message {
            ChannelMessage::Result(result) => self.send_result(result),
            ChannelMessage::Event(event) => self.send_event(event),
            _ => Err(crate::error::CommunicationError::SendFailed(
                "Unsupported message type for plugin".to_string(),
            )),
        }
    }

    fn receive_message(&self) -> Result<Option<ChannelMessage>> {
        if let Some(task) = self.receive_task()? {
            return Ok(Some(ChannelMessage::Task(task)));
        }

        if let Some(command) = self.receive_command()? {
            return Ok(Some(ChannelMessage::Command(command)));
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
