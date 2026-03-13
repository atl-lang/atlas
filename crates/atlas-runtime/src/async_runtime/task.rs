//! Task spawning and management for Atlas.
//!
//! Tasks run cooperatively on worker-thread `LocalSet`s (D-057), not as OS
//! threads.  `spawn_task` submits work to the global [`WorkerPool`] via
//! round-robin MPSC dispatch; each worker runs an isolated Tokio
//! `current_thread` runtime so tasks execute concurrently across cores
//! without a global lock on the VM.
//!
//! ## Memory model
//!
//! - Old design: `std::thread::spawn` per task → ~1 MB per task (OS thread stack).
//! - New design: `LocalSet::spawn_local` on a worker → ~100 bytes per task.
//!
//! ## Lifecycle
//!
//! 1. Caller calls `spawn_task(future, name)`.
//! 2. A `WorkerTask` closure is built that will run the future and write the
//!    result into a shared `Arc<TaskState>`.
//! 3. The closure is submitted to the `WorkerPool` (round-robin).
//! 4. The worker receives it, calls `spawn_local`, and the future runs inside
//!    the worker's `LocalSet`.
//! 5. On completion the `TaskState` is updated; anyone holding a `TaskHandle`
//!    can observe the transition.
//!
//! If the `WorkerPool` has not been initialised yet (e.g. in unit tests that
//! don't call `init_worker_pool`) we fall back to marking the task as failed
//! with a clear diagnostic rather than silently spawning an OS thread.

use crate::async_runtime::AtlasFuture;
use crate::value::{ClosureRef, FunctionRef, Value};
use futures_util::FutureExt;
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

/// Global task ID counter.
static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

// ── TaskStatus ───────────────────────────────────────────────────────────────

/// Current lifecycle state of a spawned task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is currently running.
    Running,
    /// Task completed successfully.
    Completed,
    /// Task was cancelled before or during execution.
    Cancelled,
    /// Task failed (panicked or returned an error).
    Failed,
}

// ── TaskState ────────────────────────────────────────────────────────────────

/// Shared state between a `TaskHandle` and the executing task closure.
///
/// Both the submitter (holding a `TaskHandle`) and the worker (running the
/// future) have an `Arc` clone of this so they can communicate completion
/// without a rendezvous.
pub(crate) struct TaskState {
    id: u64,
    name: Option<String>,
    status: StdMutex<TaskStatus>,
    cancelled: AtomicBool,
    result: StdMutex<Option<Result<Value, String>>>,
}

// ── TaskHandle ───────────────────────────────────────────────────────────────

/// Handle to a spawned task.
///
/// Provides status polling, cancellation, and (non-blocking) result retrieval.
/// For a blocking wait use [`spawn_and_await`].
pub struct TaskHandle {
    state: Arc<TaskState>,
    // The underlying JoinHandle is managed by the worker; we never need to
    // poll it directly, so we keep only a marker here.
    _marker: std::marker::PhantomData<JoinHandle<()>>,
}

impl TaskHandle {
    fn new(name: Option<String>) -> Self {
        let id = TASK_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            state: Arc::new(TaskState {
                id,
                name,
                status: StdMutex::new(TaskStatus::Running),
                cancelled: AtomicBool::new(false),
                result: StdMutex::new(None),
            }),
            _marker: std::marker::PhantomData,
        }
    }

    /// Unique task ID.
    pub fn id(&self) -> u64 {
        self.state.id
    }

    /// Optional human-readable task name.
    pub fn name(&self) -> Option<&str> {
        self.state.name.as_deref()
    }

    /// Current lifecycle status.
    pub fn status(&self) -> TaskStatus {
        *self.state.status.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// `true` while the task is still executing.
    pub fn is_pending(&self) -> bool {
        matches!(self.status(), TaskStatus::Running)
    }

    /// `true` once the task has finished successfully.
    pub fn is_completed(&self) -> bool {
        matches!(self.status(), TaskStatus::Completed)
    }

    /// `true` if the task was cancelled.
    pub fn is_cancelled(&self) -> bool {
        matches!(self.status(), TaskStatus::Cancelled)
    }

    /// `true` if the task failed.
    pub fn is_failed(&self) -> bool {
        matches!(self.status(), TaskStatus::Failed)
    }

    /// Request cancellation.  The task checks this flag before starting its
    /// body; cooperative mid-execution cancellation is not yet implemented.
    pub fn cancel(&self) {
        self.state.cancelled.store(true, Ordering::SeqCst);
        let mut status = self.state.status.lock().unwrap_or_else(|e| e.into_inner());
        if *status == TaskStatus::Running {
            *status = TaskStatus::Cancelled;
        }
    }

    /// `true` if `cancel()` has been called.
    pub fn is_cancellation_requested(&self) -> bool {
        self.state.cancelled.load(Ordering::SeqCst)
    }

    /// Non-blocking result check.
    ///
    /// Returns a resolved `AtlasFuture` if done, or a pending future if the
    /// task is still running.  For a blocking wait, use [`spawn_and_await`].
    pub fn join(&self) -> AtlasFuture {
        let result = self
            .state
            .result
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        match result {
            Some(Ok(value)) => AtlasFuture::resolved(value),
            Some(Err(error)) => AtlasFuture::rejected(Value::string(error)),
            None => AtlasFuture::new_pending(),
        }
    }

    /// Access to the inner state for the task executor.
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
        match self.name() {
            Some(name) => write!(f, "Task({}, \"{}\")", self.id(), name),
            None => write!(f, "Task({})", self.id()),
        }
    }
}

// ── FunctionTask ─────────────────────────────────────────────────────────────

/// A pending Atlas function call destined to run on a worker's `VMContext`.
///
/// Created by `spawn()` when the caller passes a `Function` or `Closure`
/// value instead of a `Future`.  The worker receives this through the task
/// queue and (in P05) drives it through its isolated `VMContext`.
///
/// ## Lifecycle
///
/// 1. `spawn_function_task` wraps this in a `WorkerTask` and submits it.
/// 2. The worker's event loop receives the `WorkerTask` and calls it with
///    `Arc<WorkerContext>`.
/// 3. (P05) The worker extracts the `FunctionTask` from the closure and
///    executes `func` with `args` on `ctx.vm_ctx`, sending the result on
///    `result_tx`.
/// 4. The `TaskHandle` holder receives the result via the shared `TaskState`.
pub struct FunctionTask {
    /// The function to call.  Either a bare function or a closure with
    /// captured upvalues — both resolved by the VM in P05.
    pub callable: FunctionCallable,
    /// Positional arguments to pass to the function.
    pub args: Vec<Value>,
    /// One-shot channel to deliver the result back to the task's `TaskState`.
    /// Dropped without sending if the worker shuts down before executing.
    pub result_tx: oneshot::Sender<Result<Value, String>>,
}

/// Which kind of callable is held in a `FunctionTask`.
pub enum FunctionCallable {
    /// A compiled Atlas function (no captures).
    Function(FunctionRef),
    /// A compiled Atlas closure (with upvalue environment).
    Closure(ClosureRef),
}

// ── spawn_task ───────────────────────────────────────────────────────────────

/// Spawn a new async task on the worker pool.
///
/// The future is submitted to the global `WorkerPool` (round-robin) and runs
/// cooperatively inside the worker's `LocalSet`, consuming ~100 bytes of
/// overhead rather than spawning an OS thread.
///
/// Returns an `Err`-status `TaskHandle` immediately if the pool is full or
/// has not been initialised.
///
/// # Arguments
/// * `future` — async computation producing a `Value`
/// * `name`   — optional label shown in diagnostics / `Display`
pub fn spawn_task<F>(future: F, name: Option<String>) -> TaskHandle
where
    F: std::future::Future<Output = Value> + Send + 'static,
{
    let handle = TaskHandle::new(name);
    let state = handle.state_ref();

    // Build the WorkerTask closure.  It is `Send` (travels over the MPSC
    // channel) but returns a `!Send` future (runs on the worker's LocalSet).
    let state_for_task = Arc::clone(&state);
    let task: crate::async_runtime::WorkerTask = Box::new(move |_ctx| {
        Box::pin(async move {
            // Honour pre-start cancellation.
            if state_for_task.cancelled.load(Ordering::SeqCst) {
                *state_for_task
                    .status
                    .lock()
                    .unwrap_or_else(|e| e.into_inner()) = TaskStatus::Cancelled;
                return;
            }

            let result = std::panic::AssertUnwindSafe(future).catch_unwind().await;

            match result {
                Ok(value) => {
                    let mut status = state_for_task
                        .status
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    if *status == TaskStatus::Running {
                        *status = TaskStatus::Completed;
                        *state_for_task
                            .result
                            .lock()
                            .unwrap_or_else(|e| e.into_inner()) = Some(Ok(value));
                    }
                }
                Err(panic_payload) => {
                    let msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "task panicked".to_string()
                    };
                    let mut status = state_for_task
                        .status
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    if *status == TaskStatus::Running {
                        *status = TaskStatus::Failed;
                        *state_for_task
                            .result
                            .lock()
                            .unwrap_or_else(|e| e.into_inner()) = Some(Err(msg));
                    }
                }
            }
        })
    });

    match crate::async_runtime::worker_pool() {
        Some(pool) => {
            if pool.submit(task).is_err() {
                // Backpressure: all worker channels full.  Rare in practice.
                let mut status = state.status.lock().unwrap_or_else(|e| e.into_inner());
                if *status == TaskStatus::Running {
                    *status = TaskStatus::Failed;
                    *state.result.lock().unwrap_or_else(|e| e.into_inner()) =
                        Some(Err("worker pool channel full — task dropped".to_string()));
                }
            }
        }
        None => {
            // Pool not yet initialised.  This is a programming error —
            // init_worker_pool() must be called before spawn_task().
            let mut status = state.status.lock().unwrap_or_else(|e| e.into_inner());
            if *status == TaskStatus::Running {
                *status = TaskStatus::Failed;
                *state.result.lock().unwrap_or_else(|e| e.into_inner()) = Some(Err(
                    "worker pool not initialised — call init_worker_pool() at startup".to_string(),
                ));
            }
        }
    }

    handle
}

// ── spawn_function_task ───────────────────────────────────────────────────────

/// Spawn an Atlas `Function` or `Closure` as a concurrent task.
///
/// Builds a [`FunctionTask`] and submits it to the worker pool.  The worker
/// receives the task and (in P05) executes the function on its isolated
/// `VMContext`.  Until P05 lands the result channel is sent an error so the
/// `TaskHandle` transitions to `Failed` rather than staying `Running` forever.
///
/// Returns a `TaskHandle` that reflects the outcome when the function finishes.
pub fn spawn_function_task(
    callable: FunctionCallable,
    args: Vec<Value>,
    name: Option<String>,
) -> TaskHandle {
    let handle = TaskHandle::new(name);
    let state = handle.state_ref();

    let (result_tx, mut result_rx) = oneshot::channel::<Result<Value, String>>();

    let fn_task = FunctionTask {
        callable,
        args,
        result_tx,
    };

    // Wrap FunctionTask in a WorkerTask.  The worker calls this closure with
    // its WorkerContext; P05 will use ctx.vm_ctx to execute the function.
    // For now we send a pending-implementation marker so the handle settles.
    let task: crate::async_runtime::WorkerTask = Box::new(move |_ctx| {
        // Move fn_task into the async block so the result_tx is dropped on
        // the worker thread (not the submitting thread).
        let _fn_task = fn_task; // P05 will use this; for now it's a no-op placeholder.
        Box::pin(async move {
            // B44-P05 will replace this with actual VM function execution via ctx.
            // Dropping _fn_task here closes result_tx, which is caught below.
        })
    });

    // Drive the result_rx → TaskState bridge on a background task so the
    // handle settles once the worker either executes (P05) or drops the tx.
    let state_bridge = Arc::clone(&state);
    std::thread::spawn(move || {
        // Block until the worker sends a result or drops the sender.
        let result =
            crate::async_runtime::runtime().block_on(async move { result_rx.try_recv().ok() });

        let mut status = state_bridge
            .status
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if *status == TaskStatus::Running {
            match result {
                Some(Ok(value)) => {
                    *status = TaskStatus::Completed;
                    *state_bridge
                        .result
                        .lock()
                        .unwrap_or_else(|e| e.into_inner()) = Some(Ok(value));
                }
                Some(Err(msg)) => {
                    *status = TaskStatus::Failed;
                    *state_bridge
                        .result
                        .lock()
                        .unwrap_or_else(|e| e.into_inner()) = Some(Err(msg));
                }
                None => {
                    // Sender was dropped without sending (P05 not yet implemented).
                    *status = TaskStatus::Failed;
                    *state_bridge
                        .result
                        .lock()
                        .unwrap_or_else(|e| e.into_inner()) = Some(Err(
                        "function task execution not yet implemented (B44-P05)".to_string(),
                    ));
                }
            }
        }
    });

    // Submit to pool or fail immediately if pool unavailable.
    match crate::async_runtime::worker_pool() {
        Some(pool) => {
            if pool.submit(task).is_err() {
                let mut status = state.status.lock().unwrap_or_else(|e| e.into_inner());
                if *status == TaskStatus::Running {
                    *status = TaskStatus::Failed;
                    *state.result.lock().unwrap_or_else(|e| e.into_inner()) = Some(Err(
                        "worker pool channel full — function task dropped".to_string(),
                    ));
                }
            }
        }
        None => {
            let mut status = state.status.lock().unwrap_or_else(|e| e.into_inner());
            if *status == TaskStatus::Running {
                *status = TaskStatus::Failed;
                *state.result.lock().unwrap_or_else(|e| e.into_inner()) = Some(Err(
                    "worker pool not initialised — call init_worker_pool() at startup".to_string(),
                ));
            }
        }
    }

    handle
}

// ── spawn_and_await ──────────────────────────────────────────────────────────

/// Run a future to completion, blocking the current thread until done.
///
/// This bridges the sync/async boundary for Atlas stdlib operations that need
/// to perform async work (I/O, timers, …) without language-level `await`.
///
/// Implemented via `block_on` — no busy-wait, no `thread::sleep`, no timeout.
/// The calling thread yields properly to the Tokio executor while the future
/// is pending.
///
/// # Panics
/// Panics if called from within an async context (same restriction as
/// `Runtime::block_on`).
pub fn spawn_and_await<F>(future: F) -> Result<Value, String>
where
    F: std::future::Future<Output = Value> + 'static,
{
    Ok(crate::async_runtime::block_on(future))
}

// ── join_all ─────────────────────────────────────────────────────────────────

/// Check all handles for completion, returning their results if all are done.
///
/// Returns a pending `AtlasFuture` if any task is still running, or an
/// immediately-rejected future if any task failed.
pub fn join_all(handles: Vec<TaskHandle>) -> AtlasFuture {
    let mut results = Vec::with_capacity(handles.len());

    for handle in handles {
        let result = handle
            .state
            .result
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        match result {
            Some(Ok(value)) => results.push(value),
            Some(Err(error)) => return AtlasFuture::rejected(Value::string(error)),
            None => return AtlasFuture::new_pending(),
        }
    }

    AtlasFuture::resolved(Value::array(results))
}
