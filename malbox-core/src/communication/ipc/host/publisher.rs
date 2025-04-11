use crate::communication::common::{CommandMessage, MessagePayload, MessageType, TaskMessage};
use crate::errors::{InternalError, Result};
use iceoryx2::node::Node;
use iceoryx2::port::publisher::Publisher;
use iceoryx2::prelude::*;
use tracing::{debug, error, info, warn};

/// A host-side publishder for sending messages to plugins.
pub struct HostPublisher {
    /// The publisher for sending task messages.
    task_publisher: Option<Publisher<ipc::Service, MessagePayload, ()>>,

    /// The publisher for sending command messages.
    command_publisher: Option<Publisher<ipc::Service, MessagePayload, ()>>,
}

impl HostPublisher {
    /// Create a new host publisher.
    pub fn new(node: &Node<ipc::Service>) -> Result<Self> {
        // Create the task service
        let task_service = node
            .service_builder(&"malbox.tasks".try_into().unwrap())
            .publish_subscribe::<MessagePayload>()
            .create()
            .map_err(|e| {
                InternalError::CommunicationError(format!("Failed to create task service: {}", e))
            })?;

        // Create the task publisher
        let task_publisher = task_service.publisher_builder().create().map_err(|e| {
            InternalError::CommunicationError(format!("Failed to create task publisher: {}", e))
        })?;

        // Create the command service
        let command_service = node
            .service_builder(&"malbox.commands".try_into().unwrap())
            .publish_subscribe::<MessagePayload>()
            .create()
            .map_err(|e| {
                InternalError::CommunicationError(format!(
                    "Failed to create command service: {}",
                    e
                ))
            })?;

        // Create the command publisher
        let command_publisher = command_service.publisher_builder().create().map_err(|e| {
            InternalError::CommunicationError(format!("Failed to create command publisher: {}", e))
        })?;

        Ok(Self {
            task_publisher: Some(task_publisher),
            command_publisher: Some(command_publisher),
        })
    }

    /// Send a task message to a plugin.
    pub fn send_task(&self, task: TaskMessage, plugin_id: &str) -> Result<()> {
        let publisher = self.task_publisher.as_ref().ok_or_else(|| {
            InternalError::CommunicationError("Task publisher not initialized".to_string())
        })?;

        // Create the message payload
        let payload = MessagePayload::new(MessageType::Task, "TBD", plugin_id)
            .unwrap()
            .with_task_id(&task.task_id)
            .unwrap()
            .with_task(&task)
            .unwrap();

        let sample = publisher.loan_uninit().map_err(|e| {
            InternalError::CommunicationError(format!("Failed to loan sample: {}", e))
        })?;

        let sample = sample.write_payload(payload);

        sample.send().map_err(|e| {
            InternalError::CommunicationError(format!("Failed to send sample: {}", e))
        })?;

        debug!("Sent task message: {:?}", task);

        Ok(())
    }

    /// Send a command message to a plugin.
    pub fn send_command(&self, command: CommandMessage, plugin_id: &str) -> Result<()> {
        let publisher = self.command_publisher.as_ref().ok_or_else(|| {
            InternalError::CommunicationError("Command publisher not initialized".to_string())
        })?;

        // Create the message payload
        let payload = MessagePayload::new(MessageType::Command, "TBD", plugin_id)
            .unwrap()
            .with_command(&command)
            .unwrap();

        let sample = publisher.loan_uninit().map_err(|e| {
            InternalError::CommunicationError(format!("Failed to loan sample: {}", e))
        })?;

        let sample = sample.write_payload(payload);

        sample.send().map_err(|e| {
            InternalError::CommunicationError(format!("Failed to send sample: {}", e))
        })?;

        Ok(())
    }
}
