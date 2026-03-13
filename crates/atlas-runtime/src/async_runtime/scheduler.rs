//! Work-stealing task scheduler for Atlas workers (D-057, B44-P06).
//!
//! # Design
//!
//! Each worker owns a local `crossbeam_deque::Worker<WorkerTask>` FIFO deque.
//! The deque is created on the worker thread (it is `!Send`) and a `Stealer`
//! clone — which IS `Send` — is registered in the `Scheduler` so that idle
//! workers can pull tasks from busy ones.
//!
//! Global task injection flows through a single `Injector<WorkerTask>`:
//! - `Scheduler::push(task)` enqueues a task from any thread.
//! - Each worker's find loop: local pop → injector steal → random worker steal.
//! - An MPSC wakeup channel (one per worker) notifies idle workers of new work.
//!
//! ## Theft algorithm
//!
//! ```text
//! loop {
//!     if let Some(task) = local.pop()                { execute(task); continue }
//!     if let Steal::Success(task) = injector.steal_batch_and_pop(&local) { execute(task); continue }
//!     if let Steal::Success(task) = stealers[random].steal() { execute(task); continue }
//!     wait_for_wakeup();
//! }
//! ```
//!
//! ## Memory model
//!
//! Tasks live in the deques.  Deques are lock-free; no `Mutex` needed.
//! The `Injector` is `Send + Sync`; `Stealer` is `Send`; `Worker` is `!Send`.
//!
//! ## References
//! - D-057: Atlas VM concurrency model
//! - <https://docs.rs/crossbeam-deque>
//! - Go scheduler design (M:N model)

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crossbeam_deque::{Injector, Stealer, Worker as CbWorker};

/// Per-worker initialisation bundle: local deque + wakeup receiver.
///
/// Returned by [`Scheduler::new`], one entry per worker.  The `CbWorker` is
/// `!Send` and must be moved directly onto the worker's OS thread.
pub type WorkerInit = (CbWorker<super::WorkerTask>, mpsc::Receiver<()>);
use tokio::sync::mpsc;

use super::WorkerTask;

// ── WorkerFindResult ─────────────────────────────────────────────────────────

/// Outcome of one iteration of a worker's task-finding loop.
pub enum WorkerFindResult {
    /// A task was found (from local deque, injector, or a stolen worker).
    Found(WorkerTask),
    /// No task available right now; caller should wait for a wakeup.
    Empty,
    /// At least one steal attempt returned `Retry` — try again immediately.
    Retry,
}

// ── Scheduler ────────────────────────────────────────────────────────────────

/// Global work-stealing scheduler shared by all workers.
///
/// The `Scheduler` is created by `WorkerPool::new`, then stored behind an
/// `Arc` so each worker thread holds a clone for stealing operations.
pub struct Scheduler {
    /// Global injection queue.  Tasks submitted from outside the pool
    /// (via `WorkerPool::submit`) are pushed here.
    pub injector: Arc<Injector<WorkerTask>>,
    /// One stealer per worker; `stealers[i]` lets other workers steal from
    /// worker `i`'s local deque.
    pub stealers: Arc<Vec<Stealer<WorkerTask>>>,
    /// One wakeup sender per worker.  Sending `()` wakes a sleeping worker.
    pub wakers: Arc<Vec<mpsc::Sender<()>>>,
    /// Round-robin counter for wakeup dispatch.
    next_wakeup: Arc<AtomicUsize>,
}

impl Scheduler {
    /// Create a scheduler for `n` workers.
    ///
    /// Returns the `Scheduler` and a vec of `(Worker<WorkerTask>, Receiver<()>)`,
    /// one per worker.  The `Worker` must be moved to and used exclusively on
    /// the corresponding worker thread.
    pub fn new(n: usize) -> (Self, Vec<WorkerInit>) {
        let injector = Arc::new(Injector::new());

        let mut stealers = Vec::with_capacity(n);
        let mut wakers = Vec::with_capacity(n);
        let mut per_worker = Vec::with_capacity(n);

        for _ in 0..n {
            let local: CbWorker<WorkerTask> = CbWorker::new_fifo();
            stealers.push(local.stealer());

            let (wake_tx, wake_rx) = mpsc::channel(1);
            wakers.push(wake_tx);

            per_worker.push((local, wake_rx));
        }

        let sched = Scheduler {
            injector,
            stealers: Arc::new(stealers),
            wakers: Arc::new(wakers),
            next_wakeup: Arc::new(AtomicUsize::new(0)),
        };

        (sched, per_worker)
    }

    /// Inject a task into the global queue and wake one worker.
    ///
    /// Uses round-robin wake dispatch so tasks spread across workers even
    /// before work-stealing kicks in.
    pub fn push(&self, task: WorkerTask) {
        self.injector.push(task);

        let idx = self.next_wakeup.fetch_add(1, Ordering::Relaxed) % self.wakers.len();
        // Non-blocking send: if the wakeup channel is full the worker is
        // already awake (it will drain the injector on its next iteration).
        let _ = self.wakers[idx].try_send(());
    }

    /// Find the next task for worker `id`.
    ///
    /// Order: own local deque → injector → random other worker.
    /// Returns `Retry` when a steal returns `crossbeam_deque::Steal::Retry`.
    pub fn find_task(&self, id: usize, local: &CbWorker<WorkerTask>) -> WorkerFindResult {
        // 1. Own local deque first (fastest path, no atomic contention).
        if let Some(task) = local.pop() {
            return WorkerFindResult::Found(task);
        }

        // 2. Drain the global injector into our local deque, then pop one.
        match self.injector.steal_batch_and_pop(local) {
            crossbeam_deque::Steal::Success(task) => return WorkerFindResult::Found(task),
            crossbeam_deque::Steal::Retry => return WorkerFindResult::Retry,
            crossbeam_deque::Steal::Empty => {}
        }

        // 3. Steal from a random other worker.
        let n = self.stealers.len();
        if n > 1 {
            // Pick a random starting point to avoid hot-spotting on worker 0.
            let start = {
                // Simple LCG — good enough for load balancing, no dep needed.
                let r = self
                    .next_wakeup
                    .load(Ordering::Relaxed)
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                r % (n - 1)
            };
            for offset in 0..(n - 1) {
                let target = (start + offset) % n;
                if target == id {
                    continue;
                }
                match self.stealers[target].steal() {
                    crossbeam_deque::Steal::Success(task) => return WorkerFindResult::Found(task),
                    crossbeam_deque::Steal::Retry => return WorkerFindResult::Retry,
                    crossbeam_deque::Steal::Empty => {}
                }
            }
        }

        WorkerFindResult::Empty
    }
}

impl Clone for Scheduler {
    fn clone(&self) -> Self {
        Scheduler {
            injector: Arc::clone(&self.injector),
            stealers: Arc::clone(&self.stealers),
            wakers: Arc::clone(&self.wakers),
            next_wakeup: Arc::clone(&self.next_wakeup),
        }
    }
}
