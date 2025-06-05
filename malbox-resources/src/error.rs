use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("Resource not found: {id}")]
    NotFound { id: String },
    #[error("Resource already exists: {id}")]
    AlreadyExists { id: String },
    #[error("Resource not available: {reason}")]
    NotAvailable { reason: String },
    #[error("Resource allocation failed: {reason}")]
    AllocationFailed { reason: String },
    #[error("Resource is locked: {id}")]
    ResourceLocked { id: String },
    #[error("Invalid state transition from {from:?} to {to:?} for resource {id}")]
    InvalidStateTransition {
        id: String,
        from: crate::types::ResourceState,
        to: crate::types::ResourceState,
    },
    #[error("Resource constraints not satisfied: {constraint}")]
    ConstraintsNotMet { constraint: String },
    #[error("Infrastructure provisioning failed: {details}")]
    ProvisioningFailed { details: String },
    #[error("Database operation failed")]
    Database(#[from] malbox_database::error::DatabaseError),
    #[error("Terraform operation failed")]
    Terraform(#[from] malbox_terraform::error::TerraformError),
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    #[error("Internal error: {message}")]
    Internal { message: String },
    #[error("Operation timed out after {seconds} seconds")]
    Timeout { seconds: u64 },
    #[error("Insufficient resources: {resource_type}")]
    InsufficientResources { resource_type: String },
}

pub type Result<T> = std::result::Result<T, ResourceError>;
