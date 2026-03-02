//! Task spawning and management for Atlas
//!
//! Provides task spawning, cancellation, and status tracking.
//! Tasks run concurrently on the tokio runtime and can be managed
//! through TaskHandle values.

use crate::async_runtime::AtlasFuture;
use crate::value::Value;
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex as StdMutex};
use tokio::task::JoinHandle;

/// Global task ID counter
static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task was cancelled
    Cancelled,
    /// Task failed with an error
    Failed,
}

/// Inner task state shared between TaskHandle and the actual task
pub(crate) struct TaskState {
    id: u64,
    name: Option<String>,
    status: StdMutex<TaskStatus>,
    cancelled: AtomicBool,
    result: StdMutex<Option<Result<Value, String>>>,
    /// Notified when `result` transitions from None → Some.
    result_ready: Condvar,
}

/// Handle to a spawned task
///
/// Provides control over a running task including status checking,
/// cancellation, and awaiting completion.
pub struct TaskHandle {
    state: Arc<TaskState>,
    // We store the handle but can't use it directly due to Send requirements
    // Instead we track completion via the state
    _marker: std::marker::PhantomData<JoinHandle<()>>,
}

impl TaskHandle {
    /// Create a new task handle
    fn new(name: Option<String>) -> Self {
        let id = TASK_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            state: Arc::new(TaskState {
                id,
                name,
                status: StdMutex::new(TaskStatus::Running),
                cancelled: AtomicBool::new(false),
                result: StdMutex::new(None),
                result_ready: Condvar::new(),
            }),
            _marker: std::marker::PhantomData,
        }
    }

    /// Get task ID
    pub fn id(&self) -> u64 {
        self.state.id
    }

    /// Get task name
    pub fn name(&self) -> Option<&str> {
        self.state.name.as_deref()
    }

    /// Get current task status
    pub fn status(&self) -> TaskStatus {
        *self.state.status.lock().unwrap()
    }

    /// Check if task is pending (running)
    pub fn is_pending(&self) -> bool {
        matches!(self.status(), TaskStatus::Running)
    }

    /// Check if task is completed
    pub fn is_completed(&self) -> bool {
        matches!(self.status(), TaskStatus::Completed)
    }

    /// Check if task was cancelled
    pub fn is_cancelled(&self) -> bool {
        matches!(self.status(), TaskStatus::Cancelled)
    }

    /// Check if task failed
    pub fn is_failed(&self) -> bool {
        matches!(self.status(), TaskStatus::Failed)
    }

    /// Cancel the task
    pub fn cancel(&self) {
        self.state.cancelled.store(true, Ordering::SeqCst);
        let mut status = self.state.status.lock().unwrap();
        if *status == TaskStatus::Running {
            *status = TaskStatus::Cancelled;
        }
    }

    /// Check if cancellation was requested
    pub fn is_cancellation_requested(&self) -> bool {
        self.state.cancelled.load(Ordering::SeqCst)
    }

    /// Wait for task completion and get result
    ///
    /// Returns a Future that resolves to the task's result value.
    /// Parks on condvar — zero CPU while waiting.
    pub fn join(&self) -> AtlasFuture {
        let mut result = self.state.result.lock().unwrap();
        while result.is_none() {
            result = self.state.result_ready.wait(result).unwrap();
        }
        match result.as_ref().unwrap() {
            Ok(value) => AtlasFuture::resolved(value.clone()),
            Err(error) => AtlasFuture::rejected(Value::string(error.clone())),
        }
    }

    /// Mark task as completed with result
    #[allow(dead_code)]
    fn complete(&self, result: Result<Value, String>) {
        let mut status = self.state.status.lock().unwrap();
        if *status == TaskStatus::Running {
            *status = if result.is_ok() {
                TaskStatus::Completed
            } else {
                TaskStatus::Failed
            };
        }
        *self.state.result.lock().unwrap() = Some(result);
    }

    /// Clone the task state reference
    pub(crate) fn state_ref(&self) -> Arc<TaskState> {
        Arc::clone(&self.state)
    }
}

impl Clone for TaskHandle {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            _marker: std::marker::PhantomData,
        }
    }
}

impl fmt::Debug for TaskHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaskHandle")
            .field("id", &self.id())
            .field("name", &self.name())
            .field("status", &self.status())
            .finish()
    }
}

impl fmt::Display for TaskHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = self.name() {
            write!(f, "Task({}, \"{}\")", self.id(), name)
        } else {
            write!(f, "Task({})", self.id())
        }
    }
}

/// Spawn a new async task
///
/// The task executes concurrently on the tokio runtime.
/// Returns a TaskHandle that can be used to check status or await completion.
///
/// # Arguments
/// * `future` - The async computation to run
/// * `name` - Optional task name for debugging
///
/// # Example
/// ```ignore
/// let handle = spawn_task(async { 42 }, Some("my-task"));
/// let result = handle.join(); // Returns Future<number>
/// ```
pub fn spawn_task<F>(future: F, name: Option<String>) -> TaskHandle
where
    F: std::future::Future<Output = Value> + Send + 'static,
{
    let handle = TaskHandle::new(name);
    let state = handle.state_ref();

    // Run the future on a dedicated OS thread.
    // No tokio needed — the future itself handles all async coordination.
    let state_clone = Arc::clone(&state);
    std::thread::spawn(move || {
        // Check for cancellation before starting
        if state_clone.cancelled.load(Ordering::SeqCst) {
            let mut status = state_clone.status.lock().unwrap();
            *status = TaskStatus::Cancelled;
            return;
        }

        // Execute the future using block_on (drives it to completion)
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::async_runtime::block_on(future)
        }));

        match result {
            Ok(value) => {
                let mut status = state_clone.status.lock().unwrap();
                if *status == TaskStatus::Running {
                    *status = TaskStatus::Completed;
                    *state_clone.result.lock().unwrap() = Some(Ok(value));
                    state_clone.result_ready.notify_all();
                }
            }
            Err(panic_err) => {
                let error_msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_err.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Task panicked".to_string()
                };

                let mut status = state_clone.status.lock().unwrap();
                if *status == TaskStatus::Running {
                    *status = TaskStatus::Failed;
                    *state_clone.result.lock().unwrap() = Some(Err(error_msg));
                    state_clone.result_ready.notify_all();
                }
            }
        }
    });

    handle
}

/// Spawn a task and immediately await its result
///
/// This is a convenience function that spawns a task and blocks until completion.
/// Useful for simple async operations that don't need concurrent management.
pub fn spawn_and_await<F>(future: F) -> Result<Value, String>
where
    F: std::future::Future<Output = Value> + Send + 'static,
{
    let handle = spawn_task(future, None);

    // Park on condvar until result is ready — zero CPU while waiting.
    let mut result = handle.state.result.lock().unwrap();
    let timeout = std::time::Duration::from_secs(30);
    while result.is_none() {
        let (guard, wait_result) = handle
            .state
            .result_ready
            .wait_timeout(result, timeout)
            .unwrap();
        result = guard;
        if result.is_none() && wait_result.timed_out() {
            return Err("Task timeout".to_string());
        }
    }
    result.clone().unwrap_or(Ok(Value::Null))
}

/// Join multiple tasks
///
/// Returns a Future that resolves when all tasks complete.
/// Results are returned in the same order as the input handles.
pub fn join_all(handles: Vec<TaskHandle>) -> AtlasFuture {
    let mut results = Vec::new();

    for handle in &handles {
        // Block until each task completes
        let future = handle.join();
        match future.get_state() {
            crate::async_runtime::FutureState::Resolved(value) => results.push(value),
            crate::async_runtime::FutureState::Rejected(error) => {
                return AtlasFuture::rejected(error);
            }
            crate::async_runtime::FutureState::Pending => {
                // Should not happen since join() now blocks
                return AtlasFuture::new_pending();
            }
        }
    }

    AtlasFuture::resolved(Value::array(results))
}
