use iceoryx2::node::{Node, NodeBuilder};
use iceoryx2::port::listener::Listener;
use iceoryx2::port::publisher::Publisher;
use iceoryx2::prelude::*;
use iceoryx2::service::header::publish_subscribe::Header;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IpcError {
    #[error("ipc error: {0}")]
    Ipc(String),
}

type Result<T> = std::result::Result<T, IpcError>;

/// IPC channels for the host system.
#[derive(Debug)]
pub struct GuestIpc {
    /// The iceoryx2 node represents the local runtime environment
    /// for our malbox process.
    /// The node is responsible for setting up or connecting to the
    /// shared memory segement(s), handling process-local state, and
    /// coordinating cleanup.
    pub node: Option<Node<ipc::Service>>,
    pub listener: Option<Listener<ipc::Service>>,
    pub publisher: Option<Publisher<ipc::Service, u64, ()>>,
}

impl GuestIpc {
    pub fn new() -> Result<Self> {
        Ok(Self {
            node: None,
            listener: None,
            publisher: None,
        })
    }

    pub async fn initialize(&mut self) -> Result<()> {
        let node = NodeBuilder::new()
            .create::<ipc::Service>()
            .map_err(|e| IpcError::Ipc(e.to_string()))?;

        let listener = node
            .service_builder(&"malbox.core.events".try_into().unwrap())
            .event()
            .create()
            .map_err(|e| IpcError::Ipc(e.to_string()))?
            .listener_builder()
            .create()
            .map_err(|e| IpcError::Ipc(e.to_string()))?;

        let publisher = node
            .service_builder(&"malbox.core.results".try_into().unwrap())
            .publish_subscribe::<u64>()
            .create()
            .map_err(|e| IpcError::Ipc(e.to_string()))?
            .publisher_builder()
            .create()
            .map_err(|e| IpcError::Ipc(e.to_string()))?;

        self.node = Some(node);
        self.listener = Some(listener);
        self.publisher = Some(publisher);

        Ok(())
    }

    pub async fn send_result(&self) -> Result<()> {
        let publisher = self
            .publisher
            .as_ref()
            .ok_or_else(|| IpcError::Ipc("Publisher not initialized".to_string()))?;

        let sample_uninit = publisher
            .loan_uninit()
            .map_err(|e| IpcError::Ipc(e.to_string()))?;
        let sample = sample_uninit.write_payload(1234);
        sample.send().unwrap();

        Ok(())
    }
}
