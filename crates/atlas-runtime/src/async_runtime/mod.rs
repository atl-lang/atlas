//! Async runtime infrastructure for Atlas
//!
//! This module provides the foundation for asynchronous I/O operations in Atlas:
//! - Future type for representing pending computations
//! - Tokio runtime integration for executing async operations
//! - Task management and spawning
//! - Channels for message passing
//! - Async primitives (sleep, timers, mutex, timeout)
//!
//! The async runtime enables non-blocking I/O operations without requiring
//! language-level async/await syntax (reserved for future versions).

pub mod channel;
pub mod future;
pub mod primitives;
pub mod scheduler;
pub mod task;
pub mod worker;

pub use channel::{
    channel_bounded, channel_select, channel_unbounded, ChannelReceiver, ChannelSender,
};
pub use future::{future_all, future_race, AtlasFuture, FutureState};
pub use primitives::{interval, retry_with_timeout, sleep, timeout, timer, AsyncMutex};
pub use task::spawn_blocking_task;
pub use task::{join_all, spawn_and_await, spawn_task, TaskHandle, TaskStatus};
pub use worker::{init_worker_pool, worker_pool, Worker, WorkerPool, WorkerTask};

use std::sync::OnceLock;
use tokio::runtime::Runtime;
use tokio::task::LocalSet;

// D-030: Value must be Send so it can cross thread boundaries in the multi-thread runtime.
// This assertion documents and enforces the threading contract at compile time.
const _: () = {
    fn assert_send<T: Send>() {}
    fn check() {
        assert_send::<crate::value::Value>();
    }
    let _ = check;
};

/// Global tokio runtime for async operations
static TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Base VM snapshot for blocking tasks.
///
/// Wrapped in `Mutex` because `VM` is `Send` but not `Sync` (the JIT trait
/// object is `!Sync`).  Access is guarded; callers only lock briefly to call
/// `new_for_worker()` which produces an independent `VM` clone.
/// Uses `Mutex<Option<VM>>` (not OnceLock) so it can be updated at runtime
/// (e.g. by `http.serve` to snapshot the fully-initialised VM with all globals).
static BLOCKING_BASE_VM: std::sync::OnceLock<std::sync::Mutex<Option<crate::vm::VM>>> =
    std::sync::OnceLock::new();

fn blocking_base_vm_lock() -> &'static std::sync::Mutex<Option<crate::vm::VM>> {
    BLOCKING_BASE_VM.get_or_init(|| std::sync::Mutex::new(None))
}

/// Initialise (or reinitialize) the blocking task pool's base VM snapshot.
///
/// Safe to call multiple times — subsequent calls update the stored snapshot.
/// Call before `blocking_vm()` to guarantee a valid VM is available.
pub fn init_blocking_pool(base_vm: &crate::vm::VM) {
    let mut guard = blocking_base_vm_lock()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    *guard = Some(base_vm.new_for_worker());
}

/// Reinitialise the blocking pool snapshot from the current (live) VM.
///
/// Called by the VM immediately before `http.serve` so that request-handler
/// workers see all globals (impl functions etc.) populated during program init.
pub fn reinit_blocking_pool(current_vm: &crate::vm::VM) {
    init_blocking_pool(current_vm);
}

/// Clone a fresh isolated VM for a blocking task.
///
/// Returns `None` if [`init_blocking_pool`] has not been called.
pub(crate) fn blocking_vm() -> Option<crate::vm::VM> {
    blocking_base_vm_lock()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .map(|vm| vm.new_for_worker())
}

// Thread-local LocalSet for spawning !Send futures
thread_local! {
    static LOCAL_SET: std::cell::RefCell<Option<LocalSet>> = const { std::cell::RefCell::new(None) };
}

/// Initialize the global tokio runtime
///
/// This must be called before any async operations. It creates a multi-threaded
/// tokio runtime that will be used for all async operations in Atlas.
///
/// # Panics
/// Panics if the runtime fails to initialize
pub fn init_runtime() {
    TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to initialize tokio runtime")
    });
}

/// Get a reference to the global tokio runtime
///
/// Initializes the runtime if it hasn't been initialized yet.
pub fn runtime() -> &'static Runtime {
    TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to initialize tokio runtime")
    })
}

/// Spawn a !Send task on the local set
///
/// This allows spawning futures that contain Rc/RefCell (our Value type).
/// Tasks run on the same thread but provide true async concurrency.
pub fn spawn_local<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + 'static,
    F::Output: 'static,
{
    // Initialize LocalSet if needed
    LOCAL_SET.with(|cell| {
        let mut local_set = cell.borrow_mut();
        if local_set.is_none() {
            *local_set = Some(LocalSet::new());
        }
    });

    // Spawn on the LocalSet
    tokio::task::spawn_local(future)
}

/// Block on a future until it completes
///
/// This bridges the sync/async boundary by blocking the current thread
/// until the future completes. Uses LocalSet for !Send futures.
pub fn block_on<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    // Create a new LocalSet for this block_on call
    // This ensures each block_on has its own execution context
    let local_set = LocalSet::new();
    runtime().block_on(local_set.run_until(future))
}
