use super::config::WorkerConfig;
use crate::error::Result;
use malbox_database::repositories::tasks::Task;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

/// A batch of tasks ready for processing.
#[derive(Debug)]
pub struct TaskBatch {
    /// Tasks in this batch.
    pub tasks: Vec<Task>,
    /// Resources allocated for the entire batch.
    pub resources: ResourceAllocation,
    /// Channels to send individual results back.
    pub result_channels: Vec<oneshot::Sender<Result<TaskResult>>>,
    /// When this batch was created.
    pub created_at: Instant,
}

/// Collector for building task batches.
pub struct BatchCollector {
    /// Worker configuration.
    config: WorkerConfig,
    /// Current batch being collected.
    current_batch: Option<TaskBatch>,
    /// Queue of pending tasks waiting for batch collection.
    pending_tasks: VecDeque<(
        Task,
        ResourceAllocation,
        oneshot::Sender<Result<TaskResult>>,
    )>,
}

impl BatchCollector {
    /// Create a new batch collector.
    pub fn new(config: WorkerConfig) -> Self {
        Self {
            config,
            current_batch: None,
            pending_tasks: VecDeque::new(),
        }
    }

    /// Add a task to the batch collector.
    pub async fn add_task(
        &mut self,
        task: Task,
        resources: ResourceAllocation,
        result_tx: oneshot::Sender<Result<TaskResult>>,
    ) -> Option<TaskBatch> {
        if !self.config.batch_processing {
            // Batch processing not enabled - return immediate single-task batch
            return Some(TaskBatch {
                tasks: vec![task],
                resources,
                result_channels: vec![result_tx],
                created_at: Instant::now(),
            });
        }

        // Check if we can add to current batch
        if let Some(ref current) = self.current_batch {
            if !current.tasks.is_empty() {
                let first_task = &current.tasks[0];
            }
        }

        // Add to current batch or start new one
        if let Some(ref mut current) = self.current_batch {
            current.tasks.push(task);
            current.result_channels.push(result_tx);

            // Check if batch is full
            if current.tasks.len() >= self.config.max_batch_size {
                return self.current_batch.take();
            }
        } else {
            self.start_new_batch(task, resources, result_tx);
        }

        None
    }

    /// Start a new batch with the given task.
    fn start_new_batch(
        &mut self,
        task: Task,
        resources: ResourceAllocation,
        result_tx: oneshot::Sender<Result<TaskResult>>,
    ) {
        self.current_batch = Some(TaskBatch {
            tasks: vec![task],
            resources,
            result_channels: vec![result_tx],
            created_at: Instant::now(),
        });
    }

    /// Check if the current batch should be finalized due to timeout.
    pub fn should_finalize_batch(&self) -> bool {
        if let Some(ref batch) = self.current_batch {
            let elapsed = batch.created_at.elapsed();
            elapsed >= Duration::from_millis(self.config.batch_timeout_ms)
        } else {
            false
        }
    }

    /// Finalize the current batch and return it.
    pub fn finalize_current_batch(&mut self) -> Option<TaskBatch> {
        self.current_batch.take()
    }
}
