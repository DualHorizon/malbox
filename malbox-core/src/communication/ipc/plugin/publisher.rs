use crate::communication::common::{EventMessage, MessagePayload, MessageType, ResultMessage};
use crate::errors::{InternalError, Result};
use iceoryx2::node::Node;
use iceoryx2::port::publisher::Publisher;
use iceoryx2::prelude::*;
use tracing::{debug, error, info, warn};

/// A plugin-side publisher for sending messages to the host.
pub struct PluginPublisher {
    /// The publisher for sending result messages.
    result_publisher: Option<Publisher<ipc::Service, MessagePayload, ()>>,

    /// The publisher for sending event messages.
    event_publisher: Option<Publisher<ipc::Service, MessagePayload, ()>>,
}

impl PluginPublisher {
    /// Create a new plugin publisher.
    pub fn new(node: &Node<ipc::Service>) -> Result<Self> {
        // Create the result service
        let result_service = node
            .service_builder(&"malbox.results".try_into().unwrap())
            .publish_subscribe::<MessagePayload>()
            .open()
            .map_err(|e| {
                InternalError::CommunicationError(format!("Failed to open result service: {}", e))
            })?;

        // Create the result publisher
        let result_publisher = result_service.publisher_builder().create().map_err(|e| {
            InternalError::CommunicationError(format!("Failed to create result publisher: {}", e))
        })?;

        // Create the event service
        let event_service = node
            .service_builder(&"malbox.events".try_into().unwrap())
            .publish_subscribe::<MessagePayload>()
            .open()
            .map_err(|e| {
                InternalError::CommunicationError(format!("Failed to open event service: {}", e))
            })?;

        // Create the event publisher
        let event_publisher = event_service.publisher_builder().create().map_err(|e| {
            InternalError::CommunicationError(format!("Failed to create event publisher: {}", e))
        })?;

        Ok(Self {
            result_publisher: Some(result_publisher),
            event_publisher: Some(event_publisher),
        })
    }

    /// Send a result message to the host.
    pub fn send_result(&self, result: ResultMessage) -> Result<()> {
        let publisher = self.result_publisher.as_ref().ok_or_else(|| {
            InternalError::CommunicationError("Result publisher is not initialized".to_string())
        })?;

        // Create the message payload
        let payload = MessagePayload::new(MessageType::Result, "TBD", "")
            .unwrap()
            .with_result(&result)
            .unwrap()
            .with_task_id(&result.task_id)
            .unwrap();

        // Get a sample to publish
        let sample = publisher.loan_uninit().map_err(|e| {
            InternalError::CommunicationError(format!("Failed to loan sample: {}", e))
        })?;

        let sample = sample.write_payload(payload);

        sample.send().map_err(|e| {
            InternalError::CommunicationError(format!("Failed to publish result: {}", e))
        })?;

        Ok(())
    }

    /// Send an event message to the host.
    pub fn send_event(&self, event: EventMessage) -> Result<()> {
        let publisher = self.event_publisher.as_ref().ok_or_else(|| {
            InternalError::CommunicationError("Event publisher is not initialized".to_string())
        })?;

        // Create the message payload
        let payload = MessagePayload::new(MessageType::Event, "TBD", "")
            .unwrap()
            .with_event(&event)
            .unwrap();

        // Get a sample to publish
        let sample = publisher.loan_uninit().map_err(|e| {
            InternalError::CommunicationError(format!("Failed to loan sample: {}", e))
        })?;

        let sample = sample.write_payload(payload);

        sample.send().map_err(|e| {
            InternalError::CommunicationError(format!("Failed to publish event: {}", e))
        })?;

        println!("event is sent");

        Ok(())
    }
}
