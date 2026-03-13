//! Async primitives: sleep, timers, mutex, timeout
//!
//! Provides essential async primitives for building concurrent applications.

use crate::async_runtime::AtlasFuture;
use crate::value::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex as TokioMutex;

// ============================================================================
// Sleep and Timers
// ============================================================================

/// Sleep for a specified duration
///
/// Returns a Future that resolves after the specified number of milliseconds.
/// Currently blocks the current thread (true async will be added in future versions).
pub fn sleep(milliseconds: u64) -> AtlasFuture {
    // Use Tokio's timer so the multi-thread runtime can do other work while
    // the current (calling) thread is suspended.  `block_on` is safe here
    // because `sleep()` is called from the sync VM stdlib dispatch path.
    crate::async_runtime::block_on(async move {
        tokio::time::sleep(Duration::from_millis(milliseconds)).await;
    });
    AtlasFuture::resolved(Value::Null)
}

/// Create a timer that fires after a delay.
///
/// Equivalent to `sleep` — models an explicit one-shot timer.
pub fn timer(milliseconds: u64) -> AtlasFuture {
    sleep(milliseconds)
}

/// Create a repeating interval tick.
///
/// Waits `milliseconds` and resolves.  The caller is expected to call
/// `interval()` again to model subsequent ticks (polling pattern).
pub fn interval(milliseconds: u64) -> AtlasFuture {
    sleep(milliseconds)
}

// ============================================================================
// Async Mutex
// ============================================================================

/// Async-aware mutex for protecting shared data
///
/// Unlike standard mutexes, this can be held across await points.
/// The lock operation returns a Future that resolves when the lock is acquired.
#[derive(Clone)]
pub struct AsyncMutex {
    inner: Arc<TokioMutex<Value>>,
}

impl AsyncMutex {
    /// Create a new async mutex with the given initial value
    pub fn new(value: Value) -> Self {
        Self {
            inner: Arc::new(TokioMutex::new(value)),
        }
    }

    /// Lock the mutex
    ///
    /// Returns a Future that resolves to the protected value.
    /// Currently uses blocking lock (true async will be added in future versions).
    pub fn lock(&self) -> AtlasFuture {
        // Synchronous blocking lock
        let guard = self.inner.blocking_lock();
        let value = (*guard).clone();
        AtlasFuture::resolved(value)
    }

    /// Try to lock without blocking
    ///
    /// Returns Some(value) if the lock was acquired immediately,
    /// None if the mutex is currently locked.
    pub fn try_lock(&self) -> Option<Value> {
        self.inner.try_lock().ok().map(|guard| (*guard).clone())
    }

    /// Update the value in the mutex
    ///
    /// This is a convenience method that locks, updates, and unlocks.
    /// Currently uses blocking lock (true async will be added in future versions).
    pub fn update(&self, new_value: Value) -> AtlasFuture {
        // Synchronous blocking lock
        let mut guard = self.inner.blocking_lock();
        *guard = new_value;
        AtlasFuture::resolved(Value::Null)
    }
}

impl std::fmt::Debug for AsyncMutex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncMutex").finish()
    }
}

// ============================================================================
// Timeout Operations
// ============================================================================

/// Wrap a Future with a timeout
///
/// Returns a Future that either resolves to the original value,
/// or rejects with a timeout error if the duration is exceeded.
/// Currently uses synchronous polling (true async will be added in future versions).
pub fn timeout(future: AtlasFuture, milliseconds: u64) -> AtlasFuture {
    let duration = Duration::from_millis(milliseconds);

    // Drive the AtlasFuture poll loop inside a Tokio timeout so the
    // deadline is enforced by the Tokio timer wheel (no busy-spin).
    // `block_on` is safe here because `timeout()` is called from the sync
    // VM stdlib dispatch path, not from within an async task.
    let result = crate::async_runtime::block_on(async move {
        tokio::time::timeout(duration, async move {
            loop {
                match future.get_state() {
                    crate::async_runtime::FutureState::Resolved(value) => return value,
                    crate::async_runtime::FutureState::Rejected(error) => return error,
                    crate::async_runtime::FutureState::Pending => {
                        // Yield to Tokio for 1 ms between polls so other
                        // tasks can make progress.
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                }
            }
        })
        .await
    });

    match result {
        Ok(value) => AtlasFuture::resolved(value),
        Err(_elapsed) => AtlasFuture::rejected(Value::string("Operation timed out")),
    }
}

/// Retry an operation with timeout
///
/// Attempts the operation up to max_attempts times, with a timeout per attempt.
/// Returns the first successful result or the last error.
///
/// Note: Simplified implementation for current constraints.
pub fn retry_with_timeout<F>(mut operation: F, max_attempts: usize, timeout_ms: u64) -> AtlasFuture
where
    F: FnMut() -> AtlasFuture,
{
    let mut last_error = Value::string("No attempts made");

    for _attempt in 0..max_attempts {
        let future = operation();
        let timeout_future = timeout(future, timeout_ms);

        // Poll for result (simplified)
        let start = std::time::Instant::now();
        let max_wait = Duration::from_millis(timeout_ms + 100);

        while start.elapsed() < max_wait {
            match timeout_future.get_state() {
                crate::async_runtime::FutureState::Resolved(value) => {
                    return AtlasFuture::resolved(value);
                }
                crate::async_runtime::FutureState::Rejected(error) => {
                    last_error = error;
                    break;
                }
                crate::async_runtime::FutureState::Pending => {
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }

    // All attempts failed
    AtlasFuture::rejected(last_error)
}
