//! Worker thread infrastructure for Atlas VM concurrency.
//!
//! # Design (D-057, B44-P06)
//!
//! Each worker thread owns:
//! - An isolated [`VM`] (bytecode + globals + execution context)
//! - A Tokio `current_thread` runtime + [`LocalSet`] for cooperative scheduling
//! - A `crossbeam_deque::Worker` local task deque (FIFO, `!Send`)
//! - A wakeup channel receiver (`mpsc::Receiver<()>`)
//!
//! Global task injection and work-stealing are handled by [`Scheduler`].
//!
//! ## Event loop
//!
//! ```text
//! loop {
//!     find_task() → Found  → spawn_local + yield
//!                 → Retry  → yield_now (contention, try again)
//!                 → Empty  → sleep ≤ 500µs (woken by Scheduler::push)
//! }
//! ```
//!
//! The 500µs timeout means idle workers opportunistically steal across all
//! workers even without an explicit wakeup signal, bounding worst-case latency.

use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::OnceLock;
use std::thread;

use tokio::task::LocalSet;

use crate::vm::VM;

pub use super::scheduler::Scheduler;
use super::scheduler::WorkerFindResult;

// ── WorkerTask ───────────────────────────────────────────────────────────────

/// A boxed async task that a worker executes on its `LocalSet`.
///
/// The closure is `Send + 'static` so it can travel from any submitter thread
/// through the global `Injector` and into a worker's local deque.  Once the
/// worker picks it up, it calls the closure with `Rc<WorkerContext>` (thread-
/// local) and the returned `!Send` future runs inside the `LocalSet`.
pub type WorkerTask = Box<
    dyn FnOnce(Rc<WorkerContext>) -> Pin<Box<dyn std::future::Future<Output = ()> + 'static>>
        + Send
        + 'static,
>;

// ── WorkerContext ─────────────────────────────────────────────────────────────

/// Per-worker execution context passed to every task.
///
/// `RefCell<VM>` provides interior mutability without locking: the `LocalSet`
/// schedules tasks cooperatively so only one task body runs at a time.
/// `WorkerContext` must NOT be `Send` — it lives entirely on one thread.
pub struct WorkerContext {
    /// Fully isolated VM instance for this worker.
    pub vm: RefCell<VM>,
    /// Worker index (0-based).
    pub worker_id: usize,
}

// ── Worker ────────────────────────────────────────────────────────────────────

/// A running worker thread.
pub struct Worker {
    /// Worker index (0-based).
    pub id: usize,
    /// Join handle for the OS thread.
    _thread: thread::JoinHandle<()>,
}

impl Worker {
    /// Spawn a worker thread.
    ///
    /// `local_deque` — the crossbeam `Worker` deque created by `Scheduler::new`;
    /// it is `!Send` so it is constructed before spawning and moved in.
    ///
    /// `wake_rx` — the per-worker wakeup receiver from `Scheduler::new`.
    ///
    /// `scheduler` — the shared `Scheduler` used for stealing.
    fn spawn(
        id: usize,
        worker_vm: VM,
        local_deque: crossbeam_deque::Worker<WorkerTask>,
        mut wake_rx: tokio::sync::mpsc::Receiver<()>,
        scheduler: Scheduler,
    ) -> Self {
        let thread_handle = thread::Builder::new()
            .name(format!("atlas-worker-{id}"))
            .spawn(move || {
                // WorkerContext lives on this thread for its entire lifetime.
                let worker_ctx = Rc::new(WorkerContext {
                    vm: RefCell::new(worker_vm),
                    worker_id: id,
                });

                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("atlas-worker: failed to build tokio runtime");

                let local_set = LocalSet::new();

                rt.block_on(local_set.run_until(async move {
                    loop {
                        match scheduler.find_task(id, &local_deque) {
                            WorkerFindResult::Found(task) => {
                                let ctx = Rc::clone(&worker_ctx);
                                tokio::task::spawn_local(task(ctx));
                                // Yield so the spawned task can run before we
                                // search for another one.
                                tokio::task::yield_now().await;
                            }
                            WorkerFindResult::Retry => {
                                // Steal contention — yield and try again.
                                tokio::task::yield_now().await;
                            }
                            WorkerFindResult::Empty => {
                                // No work found.  Sleep up to 500µs — woken
                                // early by Scheduler::push or timed out for
                                // opportunistic steal on the next iteration.
                                // Channel closed (Ok(None)) → pool shutting down.
                                if let Ok(None) = tokio::time::timeout(
                                    std::time::Duration::from_micros(500),
                                    wake_rx.recv(),
                                )
                                .await
                                {
                                    break;
                                }
                            }
                        }
                    }
                }));
            })
            .expect("atlas-worker: failed to spawn OS thread");

        Worker {
            id,
            _thread: thread_handle,
        }
    }
}

// ── WorkerPool ────────────────────────────────────────────────────────────────

/// Pool of worker threads with a shared work-stealing [`Scheduler`].
///
/// Created once at runtime init via [`init_worker_pool`].
pub struct WorkerPool {
    workers: Vec<Worker>,
    /// Shared scheduler used by callers to inject tasks and by workers to steal.
    scheduler: Scheduler,
}

impl WorkerPool {
    /// Create a pool with `n` workers (0 = hardware concurrency).
    ///
    /// Each worker receives an isolated VM cloned from `base_vm` via
    /// [`VM::new_for_worker`].
    pub fn new(n: usize, base_vm: &VM) -> Self {
        let count = if n == 0 {
            thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(4)
        } else {
            n
        };

        let (scheduler, per_worker) = Scheduler::new(count);

        let workers = per_worker
            .into_iter()
            .enumerate()
            .map(|(id, (local_deque, wake_rx))| {
                Worker::spawn(
                    id,
                    base_vm.new_for_worker(),
                    local_deque,
                    wake_rx,
                    scheduler.clone(),
                )
            })
            .collect();

        WorkerPool { workers, scheduler }
    }

    /// Number of workers in the pool.
    pub fn len(&self) -> usize {
        self.workers.len()
    }

    /// `true` if the pool has no workers.
    pub fn is_empty(&self) -> bool {
        self.workers.is_empty()
    }

    /// Inject a task into the pool.
    ///
    /// The task enters the global injector queue and a worker is woken via
    /// the scheduler's round-robin wakeup dispatch.  Work-stealing ensures
    /// the task reaches an idle worker even under skewed load.
    ///
    /// Returns `Err(task)` if the pool has no workers (should not happen).
    pub fn submit(&self, task: WorkerTask) -> Result<(), WorkerTask> {
        if self.workers.is_empty() {
            return Err(task);
        }
        self.scheduler.push(task);
        Ok(())
    }
}

// ── Singleton ─────────────────────────────────────────────────────────────────

/// Global worker pool singleton.
static WORKER_POOL: OnceLock<WorkerPool> = OnceLock::new();

/// Initialise the global worker pool.
///
/// Must be called exactly once, typically after the first `VM::run()` so
/// workers inherit the bytecode and globals snapshot.
/// `n = 0` → hardware concurrency.
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
/// Returns `None` before [`init_worker_pool`] is called.
pub fn worker_pool() -> Option<&'static WorkerPool> {
    WORKER_POOL.get()
}
