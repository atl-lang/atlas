//! Future/Promise stdlib functions for asynchronous operations
//!
//! This module provides Atlas stdlib functions for working with Futures:
//! - futureResolve: Create a resolved future
//! - futureReject: Create a rejected future
//! - futureNew: Create a pending future (with executor)
//! - futureThen: Chain a success handler
//! - futureCatch: Chain an error handler
//! - futureAll: Combine multiple futures
//! - futureRace: Get first completed future
//! - futureIsPending, futureIsResolved, futureIsRejected: Status checks

use super::stdlib_arity_error;
use crate::async_runtime::{future_all, future_race, AtlasFuture};
use crate::span::Span;
use crate::value::{RuntimeError, Value};
use std::sync::Arc;

/// Create a resolved future with a value
///
/// Atlas signature: `futureResolve(value: T) -> Future<T>`
pub fn future_resolve(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("futureResolve", 1, args.len(), span));
    }

    let future = AtlasFuture::resolved(args[0].clone());
    Ok(Value::Future(Arc::new(future)))
}

/// Create a rejected future with an error
///
/// Atlas signature: `futureReject(error: T) -> Future<never>`
pub fn future_reject(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("futureReject", 1, args.len(), span));
    }

    let future = AtlasFuture::rejected(args[0].clone());
    Ok(Value::Future(Arc::new(future)))
}

/// Create a new pending future
///
/// Note: Without an executor callback (which would require closures), this creates
/// a pending future that stays pending. This is mainly useful for testing.
/// In phase-11b/11c, we'll add proper executor support.
///
/// Atlas signature: `futureNew() -> Future<T>`
pub fn future_new(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(stdlib_arity_error("futureNew", 0, args.len(), span));
    }

    let future = AtlasFuture::new_pending();
    Ok(Value::Future(Arc::new(future)))
}

/// Check if a future is pending
///
/// Atlas signature: `futureIsPending(future: Future<T>) -> bool`
pub fn future_is_pending(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("futureIsPending", 1, args.len(), span));
    }

    match &args[0] {
        Value::Future(f) => Ok(Value::Bool(f.is_pending())),
        _ => Err(RuntimeError::TypeError {
            msg: format!("Expected Future, got {}", args[0].type_name()),
            span,
        }),
    }
}

/// Check if a future is resolved
///
/// Atlas signature: `futureIsResolved(future: Future<T>) -> bool`
pub fn future_is_resolved(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("futureIsResolved", 1, args.len(), span));
    }

    match &args[0] {
        Value::Future(f) => Ok(Value::Bool(f.is_resolved())),
        _ => Err(RuntimeError::TypeError {
            msg: format!("Expected Future, got {}", args[0].type_name()),
            span,
        }),
    }
}

/// Check if a future is rejected
///
/// Atlas signature: `futureIsRejected(future: Future<T>) -> bool`
pub fn future_is_rejected(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("futureIsRejected", 1, args.len(), span));
    }

    match &args[0] {
        Value::Future(f) => Ok(Value::Bool(f.is_rejected())),
        _ => Err(RuntimeError::TypeError {
            msg: format!("Expected Future, got {}", args[0].type_name()),
            span,
        }),
    }
}

/// Chain a success handler to a future
///
/// Note: In phase-11a, this only works with immediately resolved/rejected futures.
/// Dynamic chaining with pending futures will be added in phase-11b/11c with executor support.
///
/// Atlas signature: `futureThen(future: Future<T>, handler: fn(T) -> U) -> Future<U>`
pub fn future_then(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("futureThen", 2, args.len(), span));
    }

    let _future = match &args[0] {
        Value::Future(f) => f,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected Future, got {}", args[0].type_name()),
                span,
            })
        }
    };

    // For phase-11a, we can only handle immediately resolved/rejected futures
    // with a simple transformation. Full callback support requires changes
    // to how we pass functions to stdlib (need FunctionRef, not Value)
    // For now, return error indicating this is not yet fully implemented
    Err(RuntimeError::TypeError {
        msg: "futureThen with dynamic handlers requires full executor support (phase-11b/11c)"
            .to_string(),
        span,
    })
}

/// Chain an error handler to a future
///
/// Note: In phase-11a, this only works with immediately resolved/rejected futures.
/// Dynamic chaining with pending futures will be added in phase-11b/11c with executor support.
///
/// Atlas signature: `futureCatch(future: Future<T>, handler: fn(E) -> T) -> Future<T>`
pub fn future_catch(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("futureCatch", 2, args.len(), span));
    }

    let _future = match &args[0] {
        Value::Future(f) => f,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected Future, got {}", args[0].type_name()),
                span,
            })
        }
    };

    // For phase-11a, we can only handle immediately resolved/rejected futures
    // with a simple transformation. Full callback support requires changes
    // to how we pass functions to stdlib (need FunctionRef, not Value)
    // For now, return error indicating this is not yet fully implemented
    Err(RuntimeError::TypeError {
        msg: "futureCatch with dynamic handlers requires full executor support (phase-11b/11c)"
            .to_string(),
        span,
    })
}

/// Combine multiple futures into one
///
/// Returns a future that resolves when all input futures resolve,
/// with an array of all results. Rejects if any future rejects.
///
/// Atlas signature: `futureAll(futures: Array<Future<T>>) -> Future<Array<T>>`
pub fn future_all_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("futureAll", 1, args.len(), span));
    }

    let futures_array = match &args[0] {
        Value::Array(arr) => arr,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected Array, got {}", args[0].type_name()),
                span,
            })
        }
    };

    // Extract futures from array
    let mut futures = Vec::new();
    for value in futures_array.as_slice().iter() {
        match value {
            Value::Future(f) => futures.push((**f).clone()),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: format!("Expected array of Futures, got {}", value.type_name()),
                    span,
                })
            }
        }
    }

    // Combine futures
    let result = future_all(futures);
    Ok(Value::Future(Arc::new(result)))
}

/// Return the first future to complete
///
/// Creates a future that adopts the state of the first future to complete
/// (either resolved or rejected).
///
/// Atlas signature: `futureRace(futures: Array<Future<T>>) -> Future<T>`
pub fn future_race_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("futureRace", 1, args.len(), span));
    }

    let futures_array = match &args[0] {
        Value::Array(arr) => arr,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected Array, got {}", args[0].type_name()),
                span,
            })
        }
    };

    // Extract futures from array
    let mut futures = Vec::new();
    for value in futures_array.as_slice().iter() {
        match value {
            Value::Future(f) => futures.push((**f).clone()),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: format!("Expected array of Futures, got {}", value.type_name()),
                    span,
                })
            }
        }
    }

    // Race futures
    let result = future_race(futures);
    Ok(Value::Future(Arc::new(result)))
}

// ============================================================================
// B33: future namespace additions
// ============================================================================

/// future.allSettled — waits for all futures, resolves with array of Result-like objects.
/// Each element is either the resolved value or the rejection error.
/// Never rejects itself.
///
/// Atlas signature: `future.allSettled(futures: Future<T>[]) -> Future<T[]>`
pub fn future_all_settled(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("future.allSettled", 1, args.len(), span));
    }

    let futures_array = match &args[0] {
        Value::Array(arr) => arr.clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected Array, got {}", args[0].type_name()),
                span,
            })
        }
    };

    // Collect results (resolved or rejected) without propagating rejection
    let mut results = Vec::new();
    for value in futures_array.as_slice().iter() {
        match value {
            Value::Future(f) => {
                let result = match f.get_state() {
                    crate::async_runtime::FutureState::Resolved(v) => v,
                    crate::async_runtime::FutureState::Rejected(e) => e,
                    crate::async_runtime::FutureState::Pending => Value::Null,
                };
                results.push(result);
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: format!("Expected array of Futures, got {}", value.type_name()),
                    span,
                })
            }
        }
    }

    let result_future = AtlasFuture::resolved(Value::array(results));
    Ok(Value::Future(Arc::new(result_future)))
}

/// future.any — resolves with the first successfully resolved future.
/// Rejects if all futures reject.
///
/// Atlas signature: `future.any(futures: Future<T>[]) -> Future<T>`
pub fn future_any(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("future.any", 1, args.len(), span));
    }

    let futures_array = match &args[0] {
        Value::Array(arr) => arr.clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected Array, got {}", args[0].type_name()),
                span,
            })
        }
    };

    let mut futures = Vec::new();
    for value in futures_array.as_slice().iter() {
        match value {
            Value::Future(f) => futures.push((**f).clone()),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: format!("Expected array of Futures, got {}", value.type_name()),
                    span,
                })
            }
        }
    }

    // Return first resolved future; if all rejected, return rejection
    for f in &futures {
        if let crate::async_runtime::FutureState::Resolved(v) = f.get_state() {
            return Ok(Value::Future(Arc::new(AtlasFuture::resolved(v))));
        }
    }

    // All futures rejected or pending — return first rejection error
    for f in &futures {
        if let crate::async_runtime::FutureState::Rejected(e) = f.get_state() {
            return Ok(Value::Future(Arc::new(AtlasFuture::rejected(e))));
        }
    }

    // All pending — return pending future
    Ok(Value::Future(Arc::new(AtlasFuture::new_pending())))
}

/// future.never — returns a future that never resolves (always pending).
///
/// Atlas signature: `future.never() -> Future<T>`
pub fn future_never(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(stdlib_arity_error("future.never", 0, args.len(), span));
    }
    Ok(Value::Future(Arc::new(AtlasFuture::new_pending())))
}

/// future.delay — returns a future that resolves after a given number of milliseconds.
/// In synchronous context this resolves immediately (no real async delay).
///
/// Atlas signature: `future.delay(ms: number) -> Future<null>`
pub fn future_delay(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("future.delay", 1, args.len(), span));
    }

    match &args[0] {
        Value::Number(n) if *n >= 0.0 => {}
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "future.delay: duration must be a non-negative number".to_string(),
                span,
            })
        }
    }

    // In synchronous evaluation context, resolve immediately.
    // Real delay requires async execution engine (VM with await).
    let future = AtlasFuture::resolved(Value::Null);
    Ok(Value::Future(Arc::new(future)))
}

/// future.finally — chain a handler that runs regardless of resolution state.
/// Returns a future resolving to the original value (or rejection).
///
/// Atlas signature: `future.finally(future: Future<T>, handler: fn() -> void) -> Future<T>`
pub fn future_finally(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("future.finally", 2, args.len(), span));
    }

    let _future = match &args[0] {
        Value::Future(f) => f,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected Future, got {}", args[0].type_name()),
                span,
            })
        }
    };

    // Full callback support requires executor infrastructure (phase-11b/11c)
    Err(RuntimeError::TypeError {
        msg: "future.finally with dynamic handlers requires full executor support".to_string(),
        span,
    })
}

/// future instance .await() — blocking resolve in synchronous context.
/// Only works for already-resolved or already-rejected futures.
///
/// Atlas signature: `.await() -> T`
pub fn future_ns_await(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("future.await", 1, args.len(), span));
    }

    match &args[0] {
        Value::Future(f) => match f.get_state() {
            crate::async_runtime::FutureState::Resolved(v) => Ok(v),
            crate::async_runtime::FutureState::Rejected(e) => Err(RuntimeError::TypeError {
                msg: format!("Awaited future was rejected: {}", e),
                span,
            }),
            crate::async_runtime::FutureState::Pending => Err(RuntimeError::TypeError {
                msg: "Cannot await a pending future in synchronous context".to_string(),
                span,
            }),
        },
        _ => Err(RuntimeError::TypeError {
            msg: format!("Expected Future, got {}", args[0].type_name()),
            span,
        }),
    }
}
