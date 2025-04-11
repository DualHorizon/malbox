use crate::communication::common::{
    CommandMessage, EventMessage, MessagePayload, MessageType, ResultMessage, TaskMessage,
};
use crate::errors::{InternalError, Result};
use iceoryx2::node::Node;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::prelude::*;
use tracing::debug;

/// A host-side subscriber for receiving messages from plugins.
pub struct HostSubscriber {
    /// The subscriber for receiving result messages.
    result_subscriber: Option<Subscriber<ipc::Service, MessagePayload, ()>>,

    /// The subscriber for receiving event messages.
    event_subscriber: Option<Subscriber<ipc::Service, MessagePayload, ()>>,
}

impl HostSubscriber {
    /// Create a new host subscriber.
    pub fn new(node: &Node<ipc::Service>) -> Result<Self> {
        let result_service = node
            .service_builder(&"malbox.results".try_into().unwrap())
            .publish_subscribe::<MessagePayload>()
            .create()
            .map_err(|e| {
                InternalError::CommunicationError(format!("Failed to create result service: {}", e))
            })?;

        let result_subscriber = result_service.subscriber_builder().create().map_err(|e| {
            InternalError::CommunicationError(format!("Failed to create result subscriber: {}", e))
        })?;

        let event_service = node
            .service_builder(&"malbox.events".try_into().unwrap())
            .publish_subscribe::<MessagePayload>()
            .create()
            .map_err(|e| {
                InternalError::CommunicationError(format!("Failed to create event service: {}", e))
            })?;

        let event_subscriber = event_service.subscriber_builder().create().map_err(|e| {
            InternalError::CommunicationError(format!("Failed to create event subscriber: {}", e))
        })?;

        Ok(Self {
            result_subscriber: Some(result_subscriber),
            event_subscriber: Some(event_subscriber),
        })
    }

    /// Try to receive a result message.
    pub fn receive_result(&self) -> Result<Option<ResultMessage>> {
        let subscriber = self.result_subscriber.as_ref().ok_or_else(|| {
            InternalError::CommunicationError("Result subscriber not initialized".to_string())
        })?;

        match subscriber.receive() {
            Ok(Some(sample)) => {
                let payload = sample.payload();
                if payload.message_type != MessageType::Result {
                    return Err(InternalError::CommunicationError(format!(
                        "Expected result message, got {:?}",
                        payload.message_type
                    )))?;
                }

                let result = payload.to_result().unwrap();

                Ok(Some(result))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(InternalError::CommunicationError(format!(
                "Error receiving result: {}",
                e,
            ))),
        }
    }

    /// Try to receive an event message.
    pub fn receive_event(&self) -> Result<Option<EventMessage>> {
        let subscriber = self.event_subscriber.as_ref().ok_or_else(|| {
            InternalError::CommunicationError("Event subscriber not initialized".to_string())
        })?;
        match subscriber.receive() {
            Ok(Some(sample)) => {
                let payload = sample.payload();
                if payload.message_type != MessageType::Event {
                    return Err(InternalError::CommunicationError(format!(
                        "Expected event message, got {:?}",
                        payload.message_type
                    )))?;
                }

                let event = payload.to_event().unwrap();

                Ok(Some(event))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(InternalError::CommunicationError(format!(
                "Error receiving event: {}",
                e,
            ))),
        }
    }
}
