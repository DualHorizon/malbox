use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommunicationError {
    #[error("IPC initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Message send failed: {0}")]
    SendFailed(String),
    #[error("Message receive failed: {0}")]
    ReceiveFailed(String),
    #[error("Channel not initialized")]
    NotInitialized,
    #[error("Service creation failed: {0}")]
    ServiceCreationFailed(String),
    #[error("Invalid message type: expected {expected:?}, got {actual:?}")]
    InvalidMessageType {
        expected: crate::messages::MessageType,
        actual: crate::messages::MessageType,
    },
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, CommunicationError>;
