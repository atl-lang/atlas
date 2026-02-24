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

use crate::value::Value;
use std::fmt;
use std::sync::{Arc, Mutex};

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

/// Atlas Future - represents a pending asynchronous computation
///
/// Futures are the foundation of Atlas's async I/O system. They represent
/// values that may not be available yet but will be computed asynchronously.
///
/// # State Machine
/// - Pending → Resolved (success)
/// - Pending → Rejected (error)
/// - Once Resolved or Rejected, state is final
///
/// # Example Usage (Atlas code)
/// ```atlas
/// let f = futureResolve(42);
/// let f2 = then(f, |x| x * 2);
/// // f2 will resolve to 84
/// ```
#[derive(Clone)]
pub struct AtlasFuture {
    state: Arc<Mutex<FutureState>>,
}

impl AtlasFuture {
    /// Create a new pending future
    pub fn new_pending() -> Self {
        Self {
            state: Arc::new(Mutex::new(FutureState::Pending)),
        }
    }

    /// Create an immediately resolved future
    pub fn resolved(value: Value) -> Self {
        Self {
            state: Arc::new(Mutex::new(FutureState::Resolved(value))),
        }
    }

    /// Create an immediately rejected future
    pub fn rejected(error: Value) -> Self {
        Self {
            state: Arc::new(Mutex::new(FutureState::Rejected(error))),
        }
    }

    /// Check if the future is pending
    pub fn is_pending(&self) -> bool {
        matches!(*self.state.lock().unwrap(), FutureState::Pending)
    }

    /// Check if the future is resolved
    pub fn is_resolved(&self) -> bool {
        matches!(*self.state.lock().unwrap(), FutureState::Resolved(_))
    }

    /// Check if the future is rejected
    pub fn is_rejected(&self) -> bool {
        matches!(*self.state.lock().unwrap(), FutureState::Rejected(_))
    }

    /// Get the current state (cloned)
    pub fn get_state(&self) -> FutureState {
        self.state.lock().unwrap().clone()
    }

    /// Resolve the future with a value
    ///
    /// This transitions the future from Pending to Resolved.
    /// If the future is already resolved or rejected, this is a no-op.
    pub fn resolve(&self, value: Value) {
        let mut state = self.state.lock().unwrap();
        if matches!(*state, FutureState::Pending) {
            *state = FutureState::Resolved(value);
        }
    }

    /// Reject the future with an error
    ///
    /// This transitions the future from Pending to Rejected.
    /// If the future is already resolved or rejected, this is a no-op.
    pub fn reject(&self, error: Value) {
        let mut state = self.state.lock().unwrap();
        if matches!(*state, FutureState::Pending) {
            *state = FutureState::Rejected(error);
        }
    }

    /// Apply a transformation to a resolved future
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
            FutureState::Resolved(value) => {
                // Future already resolved, apply handler immediately
                let result = handler(value);
                Self::resolved(result)
            }
            FutureState::Rejected(error) => {
                // Future rejected, propagate error
                Self::rejected(error)
            }
            FutureState::Pending => {
                // For now, pending futures can't be chained dynamically
                // This would require a more complex executor with callback queues
                // For phase-11a, we focus on immediately resolved/rejected futures
                Self::new_pending()
            }
        }
    }

    /// Handle a rejected future
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
            FutureState::Resolved(value) => {
                // Future resolved, propagate value
                Self::resolved(value)
            }
            FutureState::Rejected(error) => {
                // Future rejected, apply error handler
                let result = handler(error);
                Self::resolved(result)
            }
            FutureState::Pending => {
                // Pending future - would need executor support
                Self::new_pending()
            }
        }
    }
}

impl fmt::Debug for AtlasFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Future({:?})", *self.state.lock().unwrap())
    }
}

impl fmt::Display for AtlasFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self.state.lock().unwrap() {
            FutureState::Pending => write!(f, "Future(pending)"),
            FutureState::Resolved(_) => write!(f, "Future(resolved)"),
            FutureState::Rejected(_) => write!(f, "Future(rejected)"),
        }
    }
}

/// Combine multiple futures into one that resolves when all resolve
///
/// Returns a future containing an array of all results.
/// If any future rejects, the combined future rejects immediately.
pub fn future_all(futures: Vec<AtlasFuture>) -> AtlasFuture {
    let mut results = Vec::new();

    for future in futures {
        match future.get_state() {
            FutureState::Resolved(value) => results.push(value),
            FutureState::Rejected(error) => {
                // Any rejection causes immediate rejection
                return AtlasFuture::rejected(error);
            }
            FutureState::Pending => {
                // If any future is pending, result is pending
                return AtlasFuture::new_pending();
            }
        }
    }

    // All futures resolved successfully
    AtlasFuture::resolved(Value::array(results))
}

/// Return the first future to complete (resolve or reject)
///
/// Creates a future that adopts the state of the first future to complete.
pub fn future_race(futures: Vec<AtlasFuture>) -> AtlasFuture {
    for future in futures {
        match future.get_state() {
            FutureState::Resolved(value) => {
                return AtlasFuture::resolved(value);
            }
            FutureState::Rejected(error) => {
                return AtlasFuture::rejected(error);
            }
            FutureState::Pending => {
                // Continue checking other futures
            }
        }
    }

    // All futures still pending
    AtlasFuture::new_pending()
}
