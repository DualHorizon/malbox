use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::{Notify, RwLock};

/// A task entry in our priority queue.
/// This holds both the task ID and its priority for sorting/ordering.
#[derive(Clone, Eq, PartialEq)]
struct TaskEntry {
    // Unique identifier for the task.
    task_id: i32,
    // Priority value - higher means more important.
    priority: i64,
}

/// Implements ordering for TaskEntry.
/// This determines how tasks are sorted in the priority queue.
impl Ord for TaskEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by priority (higher values first).
        // This ensures higher priority tasks are processed first.
        self.priority
            .cmp(&other.priority)
            // For tasks with the same priority, sort by ID to ensure FIFO-like ordering.
            // Lower IDs will come first within the same priority level.
            .then_with(|| other.task_id.cmp(&self.task_id))
    }
}

/// Required implementation since Ord depends on PartialOrd.
impl PartialOrd for TaskEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The TaskQueue manages tasks waiting to be executed/processed, ordered by priority.
pub struct TaskQueue {
    // RwLock allows multiple readers or a single writer.
    // BinaryHeap automatically maintains the heap property - highest priority at the top.
    queue: RwLock<BinaryHeap<TaskEntry>>,
    // `tokio::sync::Notify` is used for signaling when the queue has items.
    notify: Arc<Notify>,
}

impl TaskQueue {
    /// Create a new empty task queue.
    pub fn new() -> Self {
        Self {
            queue: RwLock::new(BinaryHeap::new()),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Add a task to the queue with a specified priority.
    /// Tasks with higher priority values will be processed before lower ones.
    pub async fn enqueue(&self, task_id: i32, priority: i64) {
        // Encapsulation to drop the lock before we notify,
        // since we could get deadlocks if we wouldn't.
        {
            // Acquire a write lock on the queue.
            let mut queue = self.queue.write().await;
            // Create a new task entry and add it to the heap.
            // The heap will automatically reorder based on our Ord implementation.
            queue.push(TaskEntry { task_id, priority });
        }
        // Notify that a task is available in the queue.
        self.notify.notify_one();
    }

    /// Get the highest priority task from the queue.
    /// The task will be popped from the queue.
    /// Returns None if queue is empty.
    pub async fn dequeue(&self) -> Option<i32> {
        // Acquire a write lock on the queue.
        let mut queue = self.queue.write().await;
        // BinaryHeap.pop() returns the highest priority item
        // according to our Ord implementation.
        queue.pop().map(|entry| entry.task_id)
    }

    /// Check if the queue is empty.
    pub async fn is_empty(&self) -> bool {
        // We only need a read lock since we're not modifying anythinig.
        let queue = self.queue.read().await;
        queue.is_empty()
    }

    /// Get the current number of tasks in the queue.
    pub async fn len(&self) -> usize {
        // We only need a read lock since we're not modifying anythinig.
        let queue = self.queue.read().await;
        queue.len()
    }

    /// Get all tasks in priority order (highest priority first).
    /// This is useful for debugging or displaying the queue contents.
    pub async fn get_all(&self) -> Vec<i32> {
        let queue = self.queue.read().await;

        // Make a clone that we can drain without affecting the
        // original queue.
        let mut cloned_queue = queue.clone();
        let mut result = Vec::with_capacity(cloned_queue.len());

        // Pop from the heap to get items in priority order.
        while let Some(entry) = cloned_queue.pop() {
            result.push(entry.task_id);
        }

        result
    }

    /// Peek at the highest priority task without removing it.
    pub async fn peek(&self) -> Option<i32> {
        let queue = self.queue.read().await;
        queue.peek().map(|entry| entry.task_id)
    }

    /// Add multiple tasks to the queue at once.
    pub async fn enqueue_batch(&self, tasks: Vec<(i32, i64)>) {
        // Encapsulation to drop the lock before we notify,
        // since we could get deadlocks if we wouldn't.
        {
            let mut queue = self.queue.write().await;
            for (task_id, priority) in tasks {
                queue.push(TaskEntry { task_id, priority });
            }
        }
        self.notify.notify_one();
    }

    /// Get the queue's event notifier.
    pub fn get_notifier(&self) -> Arc<Notify> {
        self.notify.clone()
    }
}
