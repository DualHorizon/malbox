//! Task notification service.
//!
//! This service is used to send submitted tasks to the scheduler.

use crate::error::{Result, SchedulerError};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct TaskNotificationService {
    sender: mpsc::Sender<i32>,
}

impl TaskNotificationService {
    pub fn new() -> (Self, mpsc::Receiver<i32>) {
        let (sender, receiver) = mpsc::channel(100);
        (Self { sender }, receiver)
    }

    pub async fn notify_new_task(&self, task_id: i32) -> Result<()> {
        self.sender.send(task_id).await.map_err(|_| {
            SchedulerError::NotificationServiceError("Failed to notify task schgeduler".to_string())
        })
    }
}
