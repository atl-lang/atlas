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
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::thread;

use tokio::sync::mpsc;
use tokio::task::LocalSet;

use crate::vm::VM;

/// The default worker channel capacity (backpressure buffer per worker).
const TASK_QUEUE_CAPACITY: usize = 256;

/// A boxed async task that a worker can execute on its `LocalSet`.
///
/// Tasks are `Send + 'static` so they can travel over the MPSC channel from
/// any thread to the worker; however, once received, they execute as
/// `!Send` futures inside the worker's `LocalSet` — the `FnOnce` wrapper
/// is `Send`, its *output* (`Pin<Box<dyn Future>>`) is `!Send`.
pub type WorkerTask = Box<
    dyn FnOnce(Rc<WorkerContext>) -> Pin<Box<dyn std::future::Future<Output = ()> + 'static>>
        + Send
        + 'static,
>;

/// Shared resources handed to every task running inside a worker.
///
/// Each worker owns a full isolated [`VM`] (bytecode + globals + context),
/// wrapped in a `RefCell` for interior mutability.  Tasks running on the
/// worker's `LocalSet` execute cooperatively — only one task is active at a
/// time — so `RefCell` is sufficient and no locking is needed.
///
/// `WorkerContext` is NOT `Send` (due to `RefCell`), which is correct: it
/// lives entirely on the worker thread and must never cross thread boundaries.
pub struct WorkerContext {
    /// Fully isolated VM instance for this worker thread.
    ///
    /// `RefCell` enables interior mutability for cooperative-single-threaded
    /// task execution without locking overhead.
    pub vm: std::cell::RefCell<VM>,
    /// Worker index (0-based) for diagnostics and routing.
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
    /// `worker_vm` is a pre-constructed isolated VM for this worker (see
    /// [`VM::new_for_worker`]).  The worker enters a `LocalSet` event loop and
    /// waits for tasks, executing each one on its isolated VM.
    fn spawn(id: usize, worker_vm: VM) -> Self {
        let (sender, mut receiver) = mpsc::channel::<WorkerTask>(TASK_QUEUE_CAPACITY);

        let thread_handle = thread::Builder::new()
            .name(format!("atlas-worker-{id}"))
            .spawn(move || {
                // Rc (not Arc) — WorkerContext stays on this thread forever.
                // Rc<RefCell<VM>> gives interior mutability without locking.
                let worker_ctx = Rc::new(WorkerContext {
                    vm: std::cell::RefCell::new(worker_vm),
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
                        let ctx = Rc::clone(&worker_ctx);
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
    /// Create a pool with `n` workers, each with an isolated VM cloned from `base_vm`.
    ///
    /// Pass `n = 0` to use the hardware concurrency level
    /// (`std::thread::available_parallelism`).
    pub fn new(n: usize, base_vm: &VM) -> Self {
        let count = if n == 0 {
            thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(4)
        } else {
            n
        };

        let workers = (0..count)
            .map(|id| Worker::spawn(id, base_vm.new_for_worker()))
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
/// Must be called exactly once, typically right after the first `VM::run()`
/// so workers inherit the correct bytecode and globals snapshot.
/// `n = 0` → use hardware concurrency.
///
/// Each worker receives an isolated `VM` cloned from `base_vm` via
/// [`VM::new_for_worker`] — same bytecode and globals, independent execution
/// context.
///
/// # Panics
/// Panics if called more than once.
pub fn init_worker_pool(n: usize, base_vm: &VM) {
    WORKER_POOL
        .set(WorkerPool::new(n, base_vm))
        .ok()
        .expect("init_worker_pool called more than once");
}

/// Access the global worker pool.
///
/// Returns `None` if [`init_worker_pool`] has not been called yet.
pub fn worker_pool() -> Option<&'static WorkerPool> {
    WORKER_POOL.get()
}
