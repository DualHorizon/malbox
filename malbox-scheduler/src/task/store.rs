use super::Result;
use super::TaskError;
use malbox_database::repositories::machinery::update_machine;
use malbox_database::repositories::tasks::{
    fetch_pending_tasks, fetch_task, insert_task, update_task_status, Task, TaskState,
};
use malbox_database::PgPool;
use std::collections::HashMap;
use time::OffsetDateTime;
use time::PrimitiveDateTime;
use tokio::sync::RwLock;

/// The TaskStore is responsible for storing tasks and synchronizing
/// with the database.
pub struct TaskStore {
    // Persistent database connection pool.
    db: PgPool,
    // In-memory cache of tasks for quick access.
    // Using RwLock for concurrent read/write access.
    tasks: RwLock<HashMap<i32, Task>>,
}

impl TaskStore {
    /// Creates a new TaskStore.
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            tasks: RwLock::new(HashMap::new()),
        }
    }
    /// Load a task by ID, first checking the in-memory cache,
    /// then falling back to the database if needed.
    pub async fn load_task(&self, task_id: i32) -> Result<Task> {
        // First check the in-memory cache with a read lock.
        // This allows multiple readers but blocks writers.
        {
            let tasks = self.tasks.read().await;
            if let Some(task) = tasks.get(&task_id) {
                // Returning a clone to avoid holding the lock longer than needed.
                return Ok(task.clone());
            }
        }

        // Not found in cache, fetch from the database.
        let task = fetch_task(&self.db, task_id)
            .await?
            .ok_or_else(|| TaskError::NotFound(task_id.to_string()))?;

        // Update the cache with a write lock.
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task.id.unwrap(), task.clone());
        }

        Ok(task)
    }

    /// Update the state of a task both in memory and database.
    pub async fn update_task_state(&self, task_id: i32, state: TaskState) -> Result<()> {
        // Update the in-memory cache.
        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                // Update the task's state.
                task.status = state.clone();

                // Update timestamps based on the new state.
                match &state {
                    TaskState::Running => {
                        let now_odt = OffsetDateTime::now_utc();
                        task.started_on =
                            Some(PrimitiveDateTime::new(now_odt.date(), now_odt.time()));
                    }
                    // NOTE: Should we actually consider a failed task as completed in our cache?
                    TaskState::Completed | TaskState::Failed | TaskState::Canceled => {
                        let now_odt = OffsetDateTime::now_utc();
                        task.completed_on =
                            Some(PrimitiveDateTime::new(now_odt.date(), now_odt.time()));
                    }
                    _ => {}
                }
            }
        }

        // Update task state in the database.
        update_task_status(&self.db, task_id, state).await.unwrap();

        Ok(())
    }

    /// Update the result of a task both in-memory and database.
    pub async fn update_task_result(&self, task_id: i32, result: String) -> Result<()> {
        // Update the in-memory cache.
        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                // TODO: actually update the result in struct.
                // Field is not present as of now.
                todo!();
            }
        }

        // DB function: update_task_result

        Ok(())
    }

    /// Load all pending tasks from the database.
    /// This is used during startup to initialize the task queue.
    pub async fn load_pending_tasks(&self) -> Result<Vec<Task>> {
        // Fetch the pending tasks from database.
        let pending_tasks = fetch_pending_tasks(&self.db).await?;
        // Update in-memory cache with pending tasks fetched from database.
        {
            let mut tasks_map = self.tasks.write().await;
            for task in &pending_tasks {
                tasks_map.insert(task.id.unwrap(), task.clone());
            }
        }

        Ok(pending_tasks)
    }

    /// Store a new task, both in-memory and database.
    pub async fn store_task(&self, task: Task) -> Result<()> {
        // First insert the task in the database.
        // We need the ID that postgres generates.
        let task = insert_task(&self.db, task).await?;

        // Add the task to in-memory storage.
        {
            let mut tasks_map = self.tasks.write().await;
            tasks_map.insert(task.id.unwrap(), task.clone());
        }

        Ok(())
    }
}
