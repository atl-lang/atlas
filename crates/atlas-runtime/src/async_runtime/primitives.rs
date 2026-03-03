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
    // Synchronous sleep - blocks the current thread
    // True non-blocking async will require language-level async/await support
    std::thread::sleep(Duration::from_millis(milliseconds));
    AtlasFuture::resolved(Value::Null)
}

/// Create a timer that fires after a delay
///
/// Similar to sleep but explicitly models a timer that completes.
pub fn timer(milliseconds: u64) -> AtlasFuture {
    sleep(milliseconds)
}

/// Create a repeating interval timer
///
/// Returns a Future that can be polled repeatedly.
/// Each poll waits for the interval duration.
///
/// Note: This is a simplified synchronous implementation.
/// True non-blocking async will require language-level async/await support.
pub fn interval(milliseconds: u64) -> AtlasFuture {
    // Synchronous implementation - wait for the interval then resolve
    std::thread::sleep(Duration::from_millis(milliseconds));
    AtlasFuture::resolved(Value::Null)
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
    let start = std::time::Instant::now();
    let poll_interval = Duration::from_millis(10);

    // Poll the future until resolved, rejected, or timeout
    loop {
        match future.get_state() {
            crate::async_runtime::FutureState::Resolved(value) => {
                return AtlasFuture::resolved(value);
            }
            crate::async_runtime::FutureState::Rejected(error) => {
                return AtlasFuture::rejected(error);
            }
            crate::async_runtime::FutureState::Pending => {
                if start.elapsed() > duration {
                    return AtlasFuture::rejected(Value::string("Operation timed out"));
                }
                std::thread::sleep(poll_interval);
            }
        }
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
