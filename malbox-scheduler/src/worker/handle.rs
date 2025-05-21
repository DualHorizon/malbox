use tokio::sync::{mpsc, oneshot};

/// Handle to a worker instance that allows control over the worker.
///
/// The handle provides a lightweight way to reference and control a worker
/// without owning the worker itself. Multiple components can hold handles
/// to the same worker, allowing for distributed control and management.
#[derive(Clone)]
pub struct WorkerHandle {}
