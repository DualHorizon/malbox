use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("Notification service error: {0}")]
    NotificationServiceError(String),
}

pub type Result<T> = std::result::Result<T, SchedulerError>;
