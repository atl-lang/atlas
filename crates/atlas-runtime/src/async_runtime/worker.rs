//! Worker thread infrastructure for Atlas VM concurrency.
//!
//! # Design (D-057)
//!
//! Each worker thread owns:
//! - An isolated [`VMContext`] (cloned from main at startup, then reset)
//! - A Tokio [`LocalSet`] for running `!Send` futures cooperatively
//! - A task queue (MPSC channel) for receiving tasks to execute
//!
//! Workers are created once at runtime init via [`WorkerPool`] and live for the
//! duration of the process.  Task dispatching (P04) and work-stealing (P06) are
//! wired in later phases; this phase only establishes the pool and event loops.
//!
//! ## Architecture
//!
//! ```text
//!   Main Thread
//!       │
//!       ▼
//!  WorkerPool::new(N)
//!       │
//!       ├── Worker 0: thread → LocalSet → spin (await tasks)
//!       ├── Worker 1: thread → LocalSet → spin (await tasks)
//!       └── Worker N-1: thread → LocalSet → spin (await tasks)
//! ```
//!
//! Each worker spins on a Tokio LocalSet, blocking its OS thread. Tasks will be
//! sent over the per-worker MPSC channel (wired in P04).

use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;

use tokio::sync::mpsc;
use tokio::task::LocalSet;

use crate::vm::context::VMContext;

/// The default worker channel capacity (backpressure buffer per worker).
const TASK_QUEUE_CAPACITY: usize = 256;

/// A boxed async task that a worker can execute on its `LocalSet`.
///
/// Tasks are `Send + 'static` so they can travel over the MPSC channel from
/// any thread to the worker; however, once received, they execute as
/// `!Send` futures inside the worker's `LocalSet` — the `FnOnce` wrapper
/// is `Send`, its *output* (`Pin<Box<dyn Future>>`) is `!Send`.
pub type WorkerTask = Box<
    dyn FnOnce(Arc<WorkerContext>) -> Pin<Box<dyn std::future::Future<Output = ()> + 'static>>
        + Send
        + 'static,
>;

/// Shared resources handed to every task running inside a worker.
///
/// Wraps the worker's isolated `VMContext` in an `Arc` so task closures can
/// reference it without borrowing the `Worker` directly.  Since tasks are
/// sequenced on a single `LocalSet` (no concurrent mutation), `Arc` alone is
/// sufficient — no `Mutex` required.
pub struct WorkerContext {
    /// Isolated VM execution state for this worker thread.
    pub vm_ctx: VMContext,
    /// Worker index (0-based).
    pub worker_id: usize,
}

/// A running worker thread with its own `LocalSet` and task queue.
pub struct Worker {
    /// Worker index (0-based) for diagnostics / routing.
    pub id: usize,
    /// Sender half of the per-worker task queue.
    pub sender: mpsc::Sender<WorkerTask>,
    /// Join handle for the OS thread backing this worker.
    _thread: thread::JoinHandle<()>,
}

impl Worker {
    /// Spawn a new worker thread.
    ///
    /// The worker clones `base_ctx` and immediately resets it to a fresh state,
    /// then enters a `LocalSet` event loop waiting for tasks.
    fn spawn(id: usize, base_ctx: VMContext) -> Self {
        let (sender, mut receiver) = mpsc::channel::<WorkerTask>(TASK_QUEUE_CAPACITY);

        let thread_handle = thread::Builder::new()
            .name(format!("atlas-worker-{id}"))
            .spawn(move || {
                // Each worker owns an isolated VMContext — cloned from base, then
                // reset to a clean empty state.  No shared mutable state across
                // workers.
                let mut vm_ctx = base_ctx.clone();
                vm_ctx.reset_for_worker();

                let worker_ctx = Arc::new(WorkerContext {
                    vm_ctx,
                    worker_id: id,
                });

                // Build a single-threaded Tokio runtime for this worker.
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("atlas-worker: failed to build tokio runtime");

                let local = LocalSet::new();

                // Event loop: drain the task queue inside the LocalSet.
                rt.block_on(local.run_until(async move {
                    while let Some(task) = receiver.recv().await {
                        let ctx = Arc::clone(&worker_ctx);
                        tokio::task::spawn_local(task(ctx));
                    }
                    // Channel closed → pool is shutting down; exit cleanly.
                }));
            })
            .expect("atlas-worker: failed to spawn OS thread");

        Worker {
            id,
            sender,
            _thread: thread_handle,
        }
    }
}

/// Pool of worker threads that execute Atlas tasks concurrently.
///
/// Created once at runtime init; accessed thereafter via [`worker_pool()`].
pub struct WorkerPool {
    workers: Vec<Worker>,
    /// Round-robin counter for task distribution.
    next: AtomicUsize,
}

impl WorkerPool {
    /// Create a pool with `n` workers, each seeded from `base_ctx`.
    ///
    /// Pass `n = 0` to use the hardware concurrency level
    /// (`std::thread::available_parallelism`).
    pub fn new(n: usize, base_ctx: VMContext) -> Self {
        let count = if n == 0 {
            thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(4)
        } else {
            n
        };

        let workers = (0..count)
            .map(|id| Worker::spawn(id, base_ctx.clone()))
            .collect();

        WorkerPool {
            workers,
            next: AtomicUsize::new(0),
        }
    }

    /// Number of workers in the pool.
    pub fn len(&self) -> usize {
        self.workers.len()
    }

    /// Returns `true` if the pool has no workers (should never happen in practice).
    pub fn is_empty(&self) -> bool {
        self.workers.is_empty()
    }

    /// Submit a task to the next worker (round-robin).
    ///
    /// Returns `Err(task)` if all workers' channels are full (backpressure).
    /// The caller may retry or fall back as appropriate.
    pub fn submit(&self, task: WorkerTask) -> Result<(), WorkerTask> {
        let idx = self.next.fetch_add(1, Ordering::Relaxed) % self.workers.len();
        self.workers[idx]
            .sender
            .try_send(task)
            .map_err(|e| e.into_inner())
    }

    /// Send a task to a specific worker by index.
    ///
    /// Returns `Err` if the worker channel is closed (pool is shutting down).
    pub fn send_to(&self, worker_id: usize, task: WorkerTask) -> Result<(), WorkerTask> {
        let worker = &self.workers[worker_id % self.workers.len()];
        worker.sender.try_send(task).map_err(|e| e.into_inner())
    }

    /// Sender for worker `id` — used by P04/P06 task routing.
    pub fn sender(&self, worker_id: usize) -> &mpsc::Sender<WorkerTask> {
        &self.workers[worker_id % self.workers.len()].sender
    }
}

/// Global worker pool singleton.
static WORKER_POOL: OnceLock<WorkerPool> = OnceLock::new();

/// Initialize the global worker pool.
///
/// Must be called exactly once, after [`crate::async_runtime::init_runtime()`].
/// `n = 0` → use hardware concurrency.
///
/// # Panics
/// Panics if called more than once.
pub fn init_worker_pool(n: usize, base_ctx: VMContext) {
    WORKER_POOL
        .set(WorkerPool::new(n, base_ctx))
        .ok()
        .expect("init_worker_pool called more than once");
}

/// Access the global worker pool.
///
/// Returns `None` if [`init_worker_pool`] has not been called yet.
pub fn worker_pool() -> Option<&'static WorkerPool> {
    WORKER_POOL.get()
}
