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
///    `Rc<WorkerContext>`.
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

    let (result_tx, result_rx) = oneshot::channel::<Result<Value, String>>();

    let fn_task = FunctionTask {
        callable,
        args,
        result_tx,
    };

    // Build the WorkerTask.  The closure is `Send` (travels the MPSC channel)
    // and its returned future is `!Send + 'static` (runs on the LocalSet).
    //
    // Function execution is synchronous within the async block: the VM
    // executes the function body to completion, then sends the result on
    // the oneshot channel.  Because tasks on a LocalSet run cooperatively,
    // no other task can modify `ctx.vm` while this block holds `borrow_mut`.
    let task: crate::async_runtime::WorkerTask = Box::new(move |ctx| {
        Box::pin(async move {
            // Execute the function on the worker's isolated VM.
            // `borrow_mut()` will panic if another task somehow holds the borrow —
            // which cannot happen in cooperative LocalSet scheduling.
            let exec_result = {
                let security = crate::security::SecurityContext::default();
                // Use catch_unwind to surface panics as errors rather than silently
                // closing the result channel (which shows up as "worker shut down").
                let callable = &fn_task.callable;
                let args = fn_task.args.clone();
                let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    ctx.vm
                        .borrow_mut()
                        .execute_task_function(callable, args, &security)
                }));
                match panic_result {
                    Ok(r) => r,
                    Err(payload) => {
                        let msg = payload
                            .downcast_ref::<String>()
                            .cloned()
                            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                            .unwrap_or_else(|| "unknown panic in worker task".to_string());
                        Err(msg)
                    }
                }
            };
            // Send result back to the TaskState bridge below.
            let _ = fn_task.result_tx.send(exec_result);
        })
    });

    // Drive oneshot result_rx → TaskState so the handle settles.
    // We spawn a dedicated OS thread here because `block_on(result_rx.await)`
    // from a sync context is the correct bridge pattern.  This thread exits
    // as soon as the worker sends (or drops) the channel.
    let state_bridge = Arc::clone(&state);
    std::thread::spawn(move || {
        let result = crate::async_runtime::runtime().block_on(async move { result_rx.await.ok() });

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
                    // Channel closed without sending — worker shut down.
                    *status = TaskStatus::Failed;
                    *state_bridge
                        .result
                        .lock()
                        .unwrap_or_else(|e| e.into_inner()) =
                        Some(Err("worker shut down before task completed".to_string()));
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

// ── spawn_blocking_task ───────────────────────────────────────────────────────

/// Spawn a CPU-bound or blocking-I/O Atlas function on Tokio's blocking thread pool.
///
/// Unlike [`spawn_function_task`] (which runs on a cooperative `LocalSet`
/// worker), `spawn_blocking_task` uses `tokio::task::spawn_blocking` — a
/// dedicated OS thread pool designed for long-running or blocking operations.
/// This prevents CPU-heavy Atlas functions from starving cooperative I/O tasks.
///
/// Each blocking task receives its own isolated VM cloned from the snapshot
/// registered via [`crate::async_runtime::init_blocking_pool`].
///
/// ## When to use
///
/// - Pure computations (hashing, encoding, parsing) that run for > ~1 ms
/// - Blocking file or network I/O that hasn't been ported to async
///
/// ## Ordering
///
/// Blocking tasks are fully independent; they run in parallel across the
/// OS thread pool and their completion order is non-deterministic.
pub fn spawn_blocking_task(
    callable: FunctionCallable,
    args: Vec<Value>,
    name: Option<String>,
) -> TaskHandle {
    let handle = TaskHandle::new(name);
    let state = handle.state_ref();

    // Clone a VM for this blocking invocation.
    let vm = match crate::async_runtime::blocking_vm() {
        Some(v) => v,
        None => {
            let mut status = state.status.lock().unwrap_or_else(|e| e.into_inner());
            *status = TaskStatus::Failed;
            *state.result.lock().unwrap_or_else(|e| e.into_inner()) = Some(Err(
                "blocking pool not initialised — call init_blocking_pool() at startup".to_string(),
            ));
            return handle;
        }
    };

    let state_clone = Arc::clone(&state);

    // Spawn on the multi-thread runtime so we can `.spawn_blocking` inside.
    crate::async_runtime::runtime().spawn(async move {
        let task_result = tokio::task::spawn_blocking(move || {
            let security = crate::security::SecurityContext::default();
            let mut vm = vm;
            vm.execute_task_function(&callable, args, &security)
        })
        .await;

        let mut status = state_clone.status.lock().unwrap_or_else(|e| e.into_inner());
        if *status == TaskStatus::Running {
            match task_result {
                Ok(Ok(value)) => {
                    *status = TaskStatus::Completed;
                    *state_clone.result.lock().unwrap_or_else(|e| e.into_inner()) = Some(Ok(value));
                }
                Ok(Err(msg)) => {
                    *status = TaskStatus::Failed;
                    *state_clone.result.lock().unwrap_or_else(|e| e.into_inner()) = Some(Err(msg));
                }
                Err(join_err) => {
                    *status = TaskStatus::Failed;
                    *state_clone.result.lock().unwrap_or_else(|e| e.into_inner()) =
                        Some(Err(format!("blocking task panicked: {join_err}")));
                }
            }
        }
    });

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
