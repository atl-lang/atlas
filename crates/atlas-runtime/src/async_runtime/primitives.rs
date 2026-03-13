//! Async primitives: sleep, timers, mutex, timeout
//!
//! Provides essential async primitives for building concurrent applications.
//!
//! ## Async-context safety
//!
//! All functions detect whether they are running inside a Tokio async context
//! (e.g. a worker LocalSet task) and use the appropriate code path:
//!
//! - **Inside Tokio:** `sleep`/`timeout` spawn a background task that resolves
//!   the returned `AtlasFuture` after the delay; the caller is not blocked.
//! - **Outside Tokio:** `sleep`/`timeout` drive the timer synchronously via
//!   `std::thread::sleep`, so the returned future is always already resolved.

use crate::async_runtime::AtlasFuture;
use crate::value::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex as TokioMutex;

// ============================================================================
// Sleep and Timers
// ============================================================================

/// Sleep for a specified duration.
///
/// Returns an `AtlasFuture` that resolves to `null` after `milliseconds` ms.
///
/// - **Inside a Tokio async context:** a background task is spawned to drive
///   the timer; the returned future resolves asynchronously without blocking
///   the calling thread.
/// - **Outside a Tokio context:** the current thread sleeps synchronously and
///   the returned future is already resolved when this function returns.
pub fn sleep(milliseconds: u64) -> AtlasFuture {
    let dur = Duration::from_millis(milliseconds);
    let fut = AtlasFuture::new_pending();
    let resolver = fut.clone();

    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        // Inside Tokio — spawn a background task so we don't block the executor.
        handle.spawn(async move {
            tokio::time::sleep(dur).await;
            resolver.resolve(Value::Null);
        });
    } else {
        // Outside Tokio — simple thread sleep is fine.
        std::thread::sleep(dur);
        resolver.resolve(Value::Null);
    }

    fut
}

/// Create a timer that fires after a delay.
///
/// Equivalent to `sleep` — models an explicit one-shot timer.
pub fn timer(milliseconds: u64) -> AtlasFuture {
    sleep(milliseconds)
}

/// Create a repeating interval tick.
///
/// Waits `milliseconds` and resolves. The caller is expected to call
/// `interval()` again to model subsequent ticks (polling pattern).
pub fn interval(milliseconds: u64) -> AtlasFuture {
    sleep(milliseconds)
}

// ============================================================================
// Async Mutex
// ============================================================================

/// Async-aware mutex for protecting shared data.
///
/// Unlike standard mutexes, this can be held across await points.
#[derive(Clone)]
pub struct AsyncMutex {
    inner: Arc<TokioMutex<Value>>,
}

impl AsyncMutex {
    /// Create a new async mutex with the given initial value.
    pub fn new(value: Value) -> Self {
        Self {
            inner: Arc::new(TokioMutex::new(value)),
        }
    }

    /// Lock the mutex and return the current value.
    ///
    /// - **Inside Tokio:** uses `block_in_place` so the executor thread can
    ///   temporarily block without stalling other async work.
    /// - **Outside Tokio:** uses `blocking_lock` directly.
    pub fn lock(&self) -> AtlasFuture {
        let value = if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::block_in_place(|| {
                let guard = self.inner.blocking_lock();
                guard.clone()
            })
        } else {
            let guard = self.inner.blocking_lock();
            guard.clone()
        };
        AtlasFuture::resolved(value)
    }

    /// Try to lock without blocking.
    ///
    /// Returns `Some(value)` if the lock was acquired immediately,
    /// `None` if the mutex is currently locked.
    pub fn try_lock(&self) -> Option<Value> {
        self.inner.try_lock().ok().map(|guard| guard.clone())
    }

    /// Update the value in the mutex.
    ///
    /// - **Inside Tokio:** uses `block_in_place` so the executor thread can
    ///   temporarily block without stalling other async work.
    /// - **Outside Tokio:** uses `blocking_lock` directly.
    pub fn update(&self, new_value: Value) -> AtlasFuture {
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::block_in_place(|| {
                let mut guard = self.inner.blocking_lock();
                *guard = new_value;
            });
        } else {
            let mut guard = self.inner.blocking_lock();
            *guard = new_value;
        }
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

/// Wrap a Future with a timeout.
///
/// Returns a Future that either resolves to the original value,
/// or rejects with a timeout error if the duration is exceeded.
///
/// - **Inside Tokio:** a background task drives the timeout; the returned
///   future resolves/rejects asynchronously without blocking the caller.
/// - **Outside Tokio:** the current thread blocks until the future settles
///   or the deadline is reached.
pub fn timeout(future: AtlasFuture, milliseconds: u64) -> AtlasFuture {
    let dur = Duration::from_millis(milliseconds);
    let result_fut = AtlasFuture::new_pending();
    let resolver = result_fut.clone();

    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(async move {
            match tokio::time::timeout(dur, future).await {
                Ok(Ok(value)) => resolver.resolve(value),
                Ok(Err(error)) => resolver.reject(error),
                Err(_elapsed) => resolver.reject(Value::string("Operation timed out")),
            }
        });
    } else {
        // Sync path: drive via block_on (safe outside Tokio context).
        let result =
            crate::async_runtime::block_on(async move { tokio::time::timeout(dur, future).await });
        match result {
            Ok(Ok(value)) => resolver.resolve(value),
            Ok(Err(error)) => resolver.reject(error),
            Err(_elapsed) => resolver.reject(Value::string("Operation timed out")),
        }
    }

    result_fut
}

/// Retry an operation with timeout.
///
/// Attempts the operation up to `max_attempts` times, with a timeout per attempt.
/// Returns the first successful result or the last error.
pub fn retry_with_timeout<F>(mut operation: F, max_attempts: usize, timeout_ms: u64) -> AtlasFuture
where
    F: FnMut() -> AtlasFuture,
{
    let mut last_error = Value::string("No attempts made");

    for _attempt in 0..max_attempts {
        let attempt_future = operation();
        let timed = timeout(attempt_future, timeout_ms);

        let start = std::time::Instant::now();
        let max_wait = Duration::from_millis(timeout_ms + 100);

        while start.elapsed() < max_wait {
            match timed.get_state() {
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

    AtlasFuture::rejected(last_error)
}
