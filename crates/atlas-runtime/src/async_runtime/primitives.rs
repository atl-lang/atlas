//! Async primitives: sleep, timers, mutex, timeout
//!
//! All async primitives return PENDING futures that are resolved by background
//! threads. This enables real concurrency — `spawn(sleep(5000))` does NOT block
//! the interpreter. The `await()` function is the cooperative yield point where
//! the interpreter thread spins until a specific future resolves.
//!
//! Architecture: Thread-per-operation cooperative model.
//! - sleep/interval: timer thread resolves future after delay
//! - mutex: lock thread resolves future when lock acquired
//! - timeout: deadline thread races against inner future

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
/// Returns a PENDING future immediately. A background timer thread resolves
/// the future after the specified milliseconds elapse.
pub fn sleep(milliseconds: u64) -> AtlasFuture {
    let future = AtlasFuture::new_pending();
    let future_clone = future.clone();

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(milliseconds));
        future_clone.resolve(Value::Null);
    });

    future
}

/// Create a timer that fires after a delay
///
/// Identical to sleep — returns a PENDING future resolved by a timer thread.
pub fn timer(milliseconds: u64) -> AtlasFuture {
    sleep(milliseconds)
}

/// Create a repeating interval timer
///
/// Returns a PENDING future that resolves after one interval tick.
/// A background thread handles the timing.
pub fn interval(milliseconds: u64) -> AtlasFuture {
    let future = AtlasFuture::new_pending();
    let future_clone = future.clone();

    std::thread::spawn(move || {
        // First tick is immediate (matches tokio::time::interval behavior)
        std::thread::sleep(Duration::from_millis(milliseconds));
        future_clone.resolve(Value::Null);
    });

    future
}

// ============================================================================
// Async Mutex
// ============================================================================

/// Async-aware mutex for protecting shared data
///
/// Lock operations return PENDING futures resolved by background threads
/// when the lock is acquired.
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
    /// Returns a PENDING future that resolves to the protected value
    /// when the lock is acquired. A background thread handles the blocking.
    pub fn lock(&self) -> AtlasFuture {
        let mutex = Arc::clone(&self.inner);
        let future = AtlasFuture::new_pending();
        let future_clone = future.clone();

        std::thread::spawn(move || {
            let value = super::block_on(async move {
                let guard = mutex.lock().await;
                (*guard).clone()
            });
            future_clone.resolve(value);
        });

        future
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
    /// Returns a PENDING future that resolves when the update completes.
    pub fn update(&self, new_value: Value) -> AtlasFuture {
        let mutex = Arc::clone(&self.inner);
        let future = AtlasFuture::new_pending();
        let future_clone = future.clone();

        std::thread::spawn(move || {
            super::block_on(async move {
                let mut guard = mutex.lock().await;
                *guard = new_value;
            });
            future_clone.resolve(Value::Null);
        });

        future
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
/// Returns a PENDING future that either resolves with the inner future's value
/// or rejects with a timeout error. Two threads race: one watches the inner
/// future, one enforces the deadline.
pub fn timeout(future: AtlasFuture, milliseconds: u64) -> AtlasFuture {
    // If already settled, return immediately
    match future.get_state() {
        crate::async_runtime::FutureState::Resolved(value) => {
            return AtlasFuture::resolved(value);
        }
        crate::async_runtime::FutureState::Rejected(error) => {
            return AtlasFuture::rejected(error);
        }
        crate::async_runtime::FutureState::Pending => {}
    }

    let timeout_future = AtlasFuture::new_pending();
    let result_clone = timeout_future.clone();
    let deadline_clone = timeout_future.clone();
    let inner = future.clone();

    // Thread 1: park on inner future's condvar until it settles
    std::thread::spawn(move || match inner.wait() {
        crate::async_runtime::FutureState::Resolved(value) => {
            result_clone.resolve(value);
        }
        crate::async_runtime::FutureState::Rejected(error) => {
            result_clone.reject(error);
        }
        crate::async_runtime::FutureState::Pending => unreachable!(),
    });

    // Thread 2: enforce deadline
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(milliseconds));
        // Only reject if still pending (inner may have resolved first)
        deadline_clone.reject(Value::string("Operation timed out"));
    });

    timeout_future
}

/// Retry an operation with timeout
///
/// Attempts the operation up to max_attempts times, with a timeout per attempt.
/// Returns the first successful result or the last error.
pub fn retry_with_timeout<F>(mut operation: F, max_attempts: usize, timeout_ms: u64) -> AtlasFuture
where
    F: FnMut() -> AtlasFuture,
{
    let mut last_error = Value::string("No attempts made");

    for _attempt in 0..max_attempts {
        let future = operation();
        let timeout_future = timeout(future, timeout_ms);

        // Park until this attempt settles
        match timeout_future.wait() {
            crate::async_runtime::FutureState::Resolved(value) => {
                return AtlasFuture::resolved(value);
            }
            crate::async_runtime::FutureState::Rejected(error) => {
                last_error = error;
            }
            crate::async_runtime::FutureState::Pending => unreachable!(),
        }
    }

    AtlasFuture::rejected(last_error)
}
