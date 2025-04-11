use crate::communication::common::{CommandMessage, MessagePayload, MessageType, TaskMessage};
use crate::errors::{InternalError, Result};
use iceoryx2::node::Node;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::prelude::*;

/// A plugin-side subscriber for receiving messages from the host.
pub struct PluginSubscriber {
    /// The subscriber for receiving task messages.
    task_subscriber: Option<Subscriber<ipc::Service, MessagePayload, ()>>,

    /// The subscriber for receiving command messages.
    command_subscriber: Option<Subscriber<ipc::Service, MessagePayload, ()>>,
}

impl PluginSubscriber {
    /// Create a new plugin subscriber.
    pub fn new(node: &Node<ipc::Service>) -> Result<Self> {
        // Create the task service
        let task_service = node
            .service_builder(&"malbox.tasks".try_into().unwrap())
            .publish_subscribe::<MessagePayload>()
            .open()
            .map_err(|e| {
                InternalError::CommunicationError(format!("Failed to open task service: {}", e))
            })?;

        // Create the task subscriber
        let task_subscriber = task_service.subscriber_builder().create().map_err(|e| {
            InternalError::CommunicationError(format!("Failed to create task subscriber: {}", e))
        })?;

        // Create the command service
        let command_service = node
            .service_builder(&"malbox.commands".try_into().unwrap())
            .publish_subscribe::<MessagePayload>()
            .open()
            .map_err(|e| {
                InternalError::CommunicationError(format!("Failed to open command service: {}", e))
            })?;

        // Create the command subscriber
        let command_subscriber = command_service.subscriber_builder().create().map_err(|e| {
            InternalError::CommunicationError(format!("Failed to create command subscriber: {}", e))
        })?;

        Ok(Self {
            task_subscriber: Some(task_subscriber),
            command_subscriber: Some(command_subscriber),
        })
    }

    /// Receive a task message.
    pub fn receive_task(&self) -> Result<Option<TaskMessage>> {
        let subscriber = self.task_subscriber.as_ref().ok_or_else(|| {
            InternalError::CommunicationError("Task subscriber not initialized".to_string())
        })?;

        match subscriber.receive() {
            Ok(Some(sample)) => {
                let payload = sample.payload();
                if payload.message_type != MessageType::Task {
                    return Err(InternalError::CommunicationError(format!(
                        "Expected task message, got: {:?}",
                        payload.message_type
                    )));
                }

                let task = payload.to_task().unwrap();

                Ok(Some(task))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(InternalError::CommunicationError(format!(
                "Error receiving task: {}",
                e
            ))),
        }
    }

    /// Receive a command message.
    pub fn receive_command(&self) -> Result<Option<CommandMessage>> {
        let subscriber = self.command_subscriber.as_ref().ok_or_else(|| {
            InternalError::CommunicationError("Command subscriber not initialized".to_string())
        })?;

        match subscriber.receive() {
            Ok(Some(sample)) => {
                let payload = sample.payload();
                if payload.message_type != MessageType::Command {
                    return Err(InternalError::CommunicationError(format!(
                        "Expected command message, got: {:?}",
                        payload.message_type
                    )));
                }

                let command = payload.to_command().unwrap();

                Ok(Some(command))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(InternalError::CommunicationError(format!(
                "Error receiving task: {}",
                e
            ))),
        }
    }
}
