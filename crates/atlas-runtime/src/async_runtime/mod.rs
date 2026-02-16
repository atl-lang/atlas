//! Async runtime infrastructure for Atlas
//!
//! This module provides the foundation for asynchronous I/O operations in Atlas:
//! - Future type for representing pending computations
//! - Tokio runtime integration for executing async operations
//! - Task management and spawning
//!
//! The async runtime enables non-blocking I/O operations without requiring
//! language-level async/await syntax (reserved for future versions).

pub mod future;

pub use future::{future_all, future_race, AtlasFuture, FutureState};

use std::sync::OnceLock;
use tokio::runtime::Runtime;

/// Global tokio runtime for async operations
static TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Initialize the global tokio runtime
///
/// This must be called before any async operations. It creates a multi-threaded
/// tokio runtime that will be used for all async operations in Atlas.
///
/// # Panics
/// Panics if the runtime fails to initialize
pub fn init_runtime() {
    TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
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
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to initialize tokio runtime")
    })
}

/// Spawn a task on the tokio runtime
///
/// This is a convenience function for spawning async tasks.
pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    runtime().spawn(future)
}

/// Block on a future until it completes
///
/// This bridges the sync/async boundary by blocking the current thread
/// until the future completes.
pub fn block_on<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    runtime().block_on(future)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_initialization() {
        // Runtime should initialize successfully
        let _ = runtime();
    }

    #[test]
    fn test_block_on() {
        let result = block_on(async { 42 });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_spawn() {
        let handle = spawn(async { "hello" });
        let result = block_on(handle).unwrap();
        assert_eq!(result, "hello");
    }
}
