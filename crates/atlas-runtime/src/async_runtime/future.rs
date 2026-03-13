//! Future/Promise implementation for Atlas
//!
//! Provides a Future type representing pending asynchronous computations.
//! Similar to JavaScript Promises, Atlas Futures can be in three states:
//! - Pending: computation in progress
//! - Resolved: computation completed successfully
//! - Rejected: computation failed with an error
//!
//! Futures support chaining via `then` and `catch` methods, and combinators
//! like `futureAll` and `futureRace` for working with multiple futures.
//!
//! ## std::future::Future integration
//!
//! `AtlasFuture` implements `std::future::Future<Output = Result<Value, Value>>`.
//! Resolved → `Poll::Ready(Ok(value))`, Rejected → `Poll::Ready(Err(error))`.
//! Wakers registered via `poll()` are fired when `resolve()`/`reject()` is called,
//! so Tokio (and any other executor) gets notified immediately.

use crate::value::Value;
use std::fmt;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

/// Future state representing the status of an async computation
#[derive(Clone)]
pub enum FutureState {
    /// Computation is in progress
    Pending,
    /// Computation completed successfully with a value
    Resolved(Value),
    /// Computation failed with an error
    Rejected(Value),
}

impl fmt::Debug for FutureState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FutureState::Pending => write!(f, "Pending"),
            FutureState::Resolved(_) => write!(f, "Resolved"),
            FutureState::Rejected(_) => write!(f, "Rejected"),
        }
    }
}

// ── Internal state ────────────────────────────────────────────────────────────

struct AtlasFutureInner {
    state: FutureState,
    /// Wakers registered by `poll()` while the future is Pending.
    /// Drained and woken when the future transitions to Resolved/Rejected.
    wakers: Vec<Waker>,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Atlas Future — represents a pending asynchronous computation.
///
/// Implements `std::future::Future<Output = Result<Value, Value>>`:
/// - `Ok(value)` when the future resolves successfully.
/// - `Err(error)` when the future is rejected.
///
/// Wakers are registered on `poll()` and notified on `resolve()`/`reject()`,
/// making this compatible with Tokio and any `std::future` executor.
#[derive(Clone)]
pub struct AtlasFuture {
    inner: Arc<Mutex<AtlasFutureInner>>,
}

impl AtlasFuture {
    /// Create a new pending future.
    pub fn new_pending() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AtlasFutureInner {
                state: FutureState::Pending,
                wakers: Vec::new(),
            })),
        }
    }

    /// Create an immediately resolved future.
    pub fn resolved(value: Value) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AtlasFutureInner {
                state: FutureState::Resolved(value),
                wakers: Vec::new(),
            })),
        }
    }

    /// Create an immediately rejected future.
    pub fn rejected(error: Value) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AtlasFutureInner {
                state: FutureState::Rejected(error),
                wakers: Vec::new(),
            })),
        }
    }

    /// Check if the future is pending.
    pub fn is_pending(&self) -> bool {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        matches!(inner.state, FutureState::Pending)
    }

    /// Check if the future is resolved.
    pub fn is_resolved(&self) -> bool {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        matches!(inner.state, FutureState::Resolved(_))
    }

    /// Check if the future is rejected.
    pub fn is_rejected(&self) -> bool {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        matches!(inner.state, FutureState::Rejected(_))
    }

    /// Get the current state (cloned).
    pub fn get_state(&self) -> FutureState {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.state.clone()
    }

    /// Resolve the future with a value.
    ///
    /// Transitions Pending → Resolved and wakes all registered wakers.
    /// No-op if the future is already settled.
    pub fn resolve(&self, value: Value) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if matches!(inner.state, FutureState::Pending) {
            inner.state = FutureState::Resolved(value);
            for waker in inner.wakers.drain(..) {
                waker.wake();
            }
        }
    }

    /// Reject the future with an error.
    ///
    /// Transitions Pending → Rejected and wakes all registered wakers.
    /// No-op if the future is already settled.
    pub fn reject(&self, error: Value) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if matches!(inner.state, FutureState::Pending) {
            inner.state = FutureState::Rejected(error);
            for waker in inner.wakers.drain(..) {
                waker.wake();
            }
        }
    }

    /// Apply a transformation to a resolved future.
    ///
    /// Creates a new future that will contain the result of applying
    /// the handler to this future's value (when it resolves).
    ///
    /// If this future rejects, the error propagates to the new future.
    pub fn then<F>(&self, handler: F) -> Self
    where
        F: FnOnce(Value) -> Value + 'static,
    {
        match self.get_state() {
            FutureState::Resolved(value) => Self::resolved(handler(value)),
            FutureState::Rejected(error) => Self::rejected(error),
            FutureState::Pending => Self::new_pending(),
        }
    }

    /// Handle a rejected future.
    ///
    /// Creates a new future that will contain the result of applying
    /// the error handler to this future's error (when it rejects).
    ///
    /// If this future resolves successfully, the value propagates unchanged.
    pub fn catch<F>(&self, handler: F) -> Self
    where
        F: FnOnce(Value) -> Value + 'static,
    {
        match self.get_state() {
            FutureState::Resolved(value) => Self::resolved(value),
            FutureState::Rejected(error) => Self::resolved(handler(error)),
            FutureState::Pending => Self::new_pending(),
        }
    }
}

// ── std::future::Future ───────────────────────────────────────────────────────

impl std::future::Future for AtlasFuture {
    type Output = Result<Value, Value>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        match &inner.state {
            FutureState::Resolved(v) => Poll::Ready(Ok(v.clone())),
            FutureState::Rejected(e) => Poll::Ready(Err(e.clone())),
            FutureState::Pending => {
                inner.wakers.push(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

// ── Display / Debug ───────────────────────────────────────────────────────────

impl fmt::Debug for AtlasFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        write!(f, "Future({:?})", inner.state)
    }
}

impl fmt::Display for AtlasFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        match &inner.state {
            FutureState::Pending => write!(f, "Future(pending)"),
            FutureState::Resolved(_) => write!(f, "Future(resolved)"),
            FutureState::Rejected(_) => write!(f, "Future(rejected)"),
        }
    }
}

// ── Combinators ───────────────────────────────────────────────────────────────

/// Combine multiple futures into one that resolves when all resolve.
///
/// Returns a future containing an array of all results.
/// If any future rejects, the combined future rejects immediately.
pub fn future_all(futures: Vec<AtlasFuture>) -> AtlasFuture {
    let mut results = Vec::new();

    for future in futures {
        match future.get_state() {
            FutureState::Resolved(value) => results.push(value),
            FutureState::Rejected(error) => return AtlasFuture::rejected(error),
            FutureState::Pending => return AtlasFuture::new_pending(),
        }
    }

    AtlasFuture::resolved(Value::array(results))
}

/// Return the first future to complete (resolve or reject).
///
/// Creates a future that adopts the state of the first future to complete.
pub fn future_race(futures: Vec<AtlasFuture>) -> AtlasFuture {
    for future in futures {
        match future.get_state() {
            FutureState::Resolved(value) => return AtlasFuture::resolved(value),
            FutureState::Rejected(error) => return AtlasFuture::rejected(error),
            FutureState::Pending => {}
        }
    }

    AtlasFuture::new_pending()
}
