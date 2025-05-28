//! Generic IPC channel implementation using iceoryx2.

use crate::error::{CommunicationError, Result};
use crate::messages::{ChannelMessage, MessagePayload, MessageType};
use iceoryx2::node::{Node, NodeBuilder};
use iceoryx2::port::publisher::Publisher;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::prelude::*;
use std::marker::PhantomData;
use std::sync::RwLock;
use tracing::{debug, error, info};
use uuid::Uuid;

/// Channel role determines the communication pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelRole {
    Host,
    Plugin,
}

/// Configuration for IPC channels.
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub role: ChannelRole,
    pub node_name: String,
    pub service_prefix: String,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            role: ChannelRole::Host,
            node_name: format!("malbox-node-{}", Uuid::new_v4()),
            service_prefix: "malbox".to_string(),
        }
    }
}

/// Generic IPC channel using iceoryx2.
pub struct Channel<R> {
    node: Option<Node<ipc::Service>>,
    config: ChannelConfig,
    publishers: RwLock<Vec<Publisher<ipc::Service, MessagePayload, ()>>>,
    subscribers: RwLock<Vec<Subscriber<ipc::Service, MessagePayload, ()>>>,
    is_initialized: RwLock<bool>,
    _role: PhantomData<R>,
}

impl<R> Channel<R> {
    pub fn new(config: ChannelConfig) -> Self {
        Self {
            node: None,
            config,
            publishers: RwLock::new(Vec::new()),
            subscribers: RwLock::new(Vec::new()),
            is_initialized: RwLock::new(false),
            _role: PhantomData,
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        let node = NodeBuilder::new()
            .name(&self.config.node_name.as_str().try_into().unwrap())
            .create()
            .map_err(|e| {
                CommunicationError::InitializationFailed(format!("Node creation: {}", e))
            })?;

        self.node = Some(node);
        *self.is_initialized.write().unwrap() = true;

        info!(
            "Initialized IPC channel: {} ({:?})",
            self.config.node_name, self.config.role
        );
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        *self.is_initialized.read().unwrap()
    }

    pub fn id(&self) -> &str {
        &self.config.node_name
    }

    /// Create a publisher for the specified service.
    pub fn create_publisher(&self, service_name: &str) -> Result<()> {
        if !self.is_initialized() {
            return Err(CommunicationError::NotInitialized);
        }

        let node = self.node.as_ref().unwrap();
        let service = node
            .service_builder(
                &format!("{}.{}", self.config.service_prefix, service_name)
                    .try_into()
                    .unwrap(),
            )
            .publish_subscribe::<MessagePayload>()
            .open()
            .map_err(|e| {
                CommunicationError::ServiceCreationFailed(format!("Publisher service: {}", e))
            })?;

        let publisher = service
            .publisher_builder()
            .create()
            .map_err(|e| CommunicationError::ServiceCreationFailed(format!("Publisher: {}", e)))?;

        self.publishers.write().unwrap().push(publisher);
        debug!("Created publisher for service: {}", service_name);
        Ok(())
    }

    /// Create a subscriber for the specified service.
    pub fn create_subscriber(&self, service_name: &str) -> Result<()> {
        if !self.is_initialized() {
            return Err(CommunicationError::NotInitialized);
        }

        let node = self.node.as_ref().unwrap();
        let service = node
            .service_builder(
                &format!("{}.{}", self.config.service_prefix, service_name)
                    .try_into()
                    .unwrap(),
            )
            .publish_subscribe::<MessagePayload>()
            .open()
            .map_err(|e| {
                CommunicationError::ServiceCreationFailed(format!("Subscriber service: {}", e))
            })?;

        let subscriber = service
            .subscriber_builder()
            .create()
            .map_err(|e| CommunicationError::ServiceCreationFailed(format!("Subscriber: {}", e)))?;

        self.subscribers.write().unwrap().push(subscriber);
        debug!("Created subscriber for service: {}", service_name);
        Ok(())
    }

    /// Send a message using the first available publisher.
    pub fn send_message(&self, payload: MessagePayload) -> Result<()> {
        let publishers = self.publishers.read().unwrap();
        let publisher = publishers
            .first()
            .ok_or_else(|| CommunicationError::SendFailed("No publishers available".to_string()))?;

        let sample = publisher
            .loan_uninit()
            .map_err(|e| CommunicationError::SendFailed(format!("Loan sample: {}", e)))?;

        let sample = sample.write_payload(payload);
        sample
            .send()
            .map_err(|e| CommunicationError::SendFailed(format!("Send sample: {}", e)))?;

        Ok(())
    }

    /// Try to receive a message from any subscriber.
    pub fn receive_message(&self) -> Result<Option<MessagePayload>> {
        let subscribers = self.subscribers.read().unwrap();

        for subscriber in subscribers.iter() {
            match subscriber.receive() {
                Ok(Some(sample)) => {
                    let payload = sample.payload().clone();
                    return Ok(Some(payload));
                }
                Ok(None) => continue,
                Err(e) => {
                    error!("Error receiving message: {}", e);
                    continue;
                }
            }
        }

        Ok(None)
    }

    pub fn close(&self) -> Result<()> {
        *self.is_initialized.write().unwrap() = false;
        info!("Closed IPC channel: {}", self.config.node_name);
        Ok(())
    }
}
