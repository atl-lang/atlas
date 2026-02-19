//! FFI Callbacks - Enable C code to call Atlas functions
//!
//! Provides trampoline generation for Atlas→C function pointer conversion.
//! Uses the standard C callback pattern: `extern "C" fn` trampolines with
//! opaque context pointers, enabling sound bidirectional FFI (Atlas↔C).
//!
//! # Sound Implementation
//!
//! C callbacks require TWO components:
//! - A function pointer (`trampoline`) - a real `extern "C" fn` with C ABI
//! - A context pointer (`context`) - opaque pointer passed to the trampoline
//!
//! The trampoline receives context as its first argument, casts it back to
//! the closure, and invokes it. This is the pattern used by libffi, GLFW,
//! Lua C API, and all professional C callback APIs.

use crate::ffi::types::ExternType;
use crate::value::{RuntimeError, Value};
use std::ffi::c_void;
use std::os::raw::{c_double, c_int, c_long};

/// The closure type stored in callback handles
type CallbackClosure = Box<dyn Fn(&[Value]) -> Result<Value, RuntimeError>>;

/// Errors that can occur during callback creation or execution
#[derive(Debug)]
pub enum CallbackError {
    /// Marshaling error during argument/return conversion
    MarshalError(String),
    /// Execution error when calling Atlas function
    ExecutionError(String),
    /// Signature validation error
    InvalidSignature(String),
    /// Unsupported callback signature
    UnsupportedSignature(String),
}

impl From<crate::ffi::marshal::MarshalError> for CallbackError {
    fn from(e: crate::ffi::marshal::MarshalError) -> Self {
        CallbackError::MarshalError(format!("{:?}", e))
    }
}

impl std::fmt::Display for CallbackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallbackError::MarshalError(msg) => write!(f, "Marshal error: {}", msg),
            CallbackError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            CallbackError::InvalidSignature(msg) => write!(f, "Invalid signature: {}", msg),
            CallbackError::UnsupportedSignature(msg) => {
                write!(f, "Unsupported signature: {}", msg)
            }
        }
    }
}

impl std::error::Error for CallbackError {}

// =============================================================================
// EXTERN "C" TRAMPOLINES
// =============================================================================
// These are real function pointers with stable addresses conforming to C ABI.
// Each receives a context pointer as the first argument, casts it to the
// closure, and invokes it with the appropriate arguments.

/// Trampoline: () -> c_int
///
/// # Safety
/// Context must be a valid pointer created by Box::into_raw on a CallbackClosure.
unsafe extern "C" fn trampoline_void_to_int(context: *mut c_void) -> c_int {
    let closure = &*(context as *const CallbackClosure);
    match closure(&[]) {
        Ok(Value::Number(n)) => n as c_int,
        Ok(_) | Err(_) => 0,
    }
}

/// Trampoline: c_double -> c_double
///
/// # Safety
/// Context must be a valid pointer created by Box::into_raw on a CallbackClosure.
unsafe extern "C" fn trampoline_double_to_double(context: *mut c_void, x: c_double) -> c_double {
    let closure = &*(context as *const CallbackClosure);
    match closure(&[Value::Number(x)]) {
        Ok(Value::Number(n)) => n,
        Ok(_) | Err(_) => 0.0,
    }
}

/// Trampoline: (c_double, c_double) -> c_double
///
/// # Safety
/// Context must be a valid pointer created by Box::into_raw on a CallbackClosure.
unsafe extern "C" fn trampoline_double_double_to_double(
    context: *mut c_void,
    x: c_double,
    y: c_double,
) -> c_double {
    let closure = &*(context as *const CallbackClosure);
    match closure(&[Value::Number(x), Value::Number(y)]) {
        Ok(Value::Number(n)) => n,
        Ok(_) | Err(_) => 0.0,
    }
}

/// Trampoline: c_int -> c_int
///
/// # Safety
/// Context must be a valid pointer created by Box::into_raw on a CallbackClosure.
unsafe extern "C" fn trampoline_int_to_int(context: *mut c_void, x: c_int) -> c_int {
    let closure = &*(context as *const CallbackClosure);
    match closure(&[Value::Number(x as f64)]) {
        Ok(Value::Number(n)) => n as c_int,
        Ok(_) | Err(_) => 0,
    }
}

/// Trampoline: (c_int, c_int) -> c_int
///
/// # Safety
/// Context must be a valid pointer created by Box::into_raw on a CallbackClosure.
unsafe extern "C" fn trampoline_int_int_to_int(context: *mut c_void, x: c_int, y: c_int) -> c_int {
    let closure = &*(context as *const CallbackClosure);
    match closure(&[Value::Number(x as f64), Value::Number(y as f64)]) {
        Ok(Value::Number(n)) => n as c_int,
        Ok(_) | Err(_) => 0,
    }
}

/// Trampoline: c_long -> c_long
///
/// # Safety
/// Context must be a valid pointer created by Box::into_raw on a CallbackClosure.
unsafe extern "C" fn trampoline_long_to_long(context: *mut c_void, x: c_long) -> c_long {
    let closure = &*(context as *const CallbackClosure);
    match closure(&[Value::Number(x as f64)]) {
        Ok(Value::Number(n)) => n as c_long,
        Ok(_) | Err(_) => 0,
    }
}

/// Trampoline: () -> void
///
/// # Safety
/// Context must be a valid pointer created by Box::into_raw on a CallbackClosure.
unsafe extern "C" fn trampoline_void_to_void(context: *mut c_void) {
    let closure = &*(context as *const CallbackClosure);
    let _ = closure(&[]);
}

/// Trampoline: c_int -> void
///
/// # Safety
/// Context must be a valid pointer created by Box::into_raw on a CallbackClosure.
unsafe extern "C" fn trampoline_int_to_void(context: *mut c_void, x: c_int) {
    let closure = &*(context as *const CallbackClosure);
    let _ = closure(&[Value::Number(x as f64)]);
}

// =============================================================================
// CALLBACK HANDLE
// =============================================================================

/// Represents an Atlas function wrapped for C calling
///
/// A CallbackHandle provides TWO pointers that C code needs:
/// - `trampoline()` - The real `extern "C" fn` pointer to call
/// - `context()` - An opaque pointer that MUST be passed as the first argument
///
/// C code should call: `trampoline(context, arg1, arg2, ...)`
///
/// The handle owns the closure and frees it on Drop.
pub struct CallbackHandle {
    /// Real extern "C" function pointer - safe to pass to C
    trampoline: *const (),
    /// Opaque context pointer - the boxed closure address
    context: *mut c_void,
    /// Signature for validation
    param_types: Vec<ExternType>,
    return_type: ExternType,
}

impl CallbackHandle {
    /// Get the trampoline function pointer for C code
    ///
    /// This is a real `extern "C" fn` that can be called from C.
    /// C code MUST pass `context()` as the first argument.
    pub fn trampoline(&self) -> *const () {
        self.trampoline
    }

    /// Get the context pointer to pass to the trampoline
    ///
    /// This is an opaque pointer that carries the closure data.
    /// It MUST be passed as the first argument when calling the trampoline.
    pub fn context(&self) -> *mut c_void {
        self.context
    }

    /// Get the callback signature
    pub fn signature(&self) -> (&[ExternType], &ExternType) {
        (&self.param_types, &self.return_type)
    }

    /// Deprecated: Use `trampoline()` instead
    ///
    /// Note: This returns the trampoline for compatibility, but callers
    /// must also use `context()` when invoking the callback.
    #[deprecated(note = "Use trampoline() and context() instead")]
    pub fn fn_ptr(&self) -> *const () {
        self.trampoline
    }
}

impl Drop for CallbackHandle {
    fn drop(&mut self) {
        // SAFETY: context was created by Box::into_raw in create_callback.
        // We must reclaim it to prevent memory leaks.
        if !self.context.is_null() {
            unsafe {
                let _ = Box::from_raw(self.context as *mut CallbackClosure);
            }
        }
    }
}

// Note: CallbackHandle contains raw pointers, so it's not Send/Sync by default.
// The compiler auto-derives !Send and !Sync for types with raw pointers.

/// Signature string for callback identification
fn signature_string(param_types: &[ExternType], return_type: &ExternType) -> String {
    let params = param_types
        .iter()
        .map(|t| format!("{:?}", t))
        .collect::<Vec<_>>()
        .join(",");
    format!("({:?})->({:?})", params, return_type)
}

/// Create a C-callable callback for an Atlas function
///
/// This generates a trampoline function that:
/// 1. Receives a context pointer (the closure) and C arguments
/// 2. Casts context back to the closure
/// 3. Marshals C args to Atlas values
/// 4. Calls the Atlas function
/// 5. Marshals the result back to C
/// 6. Returns to C code
///
/// # Usage
///
/// ```ignore
/// let handle = create_callback(my_fn, vec![], ExternType::CInt)?;
///
/// // C code calls: trampoline(context)
/// let trampoline: unsafe extern "C" fn(*mut c_void) -> c_int =
///     unsafe { std::mem::transmute(handle.trampoline()) };
/// let result = unsafe { trampoline(handle.context()) };
/// ```
///
/// # Safety
///
/// The callback_fn must correctly handle the provided arguments and return
/// a value compatible with the return_type.
pub fn create_callback<F>(
    callback_fn: F,
    param_types: Vec<ExternType>,
    return_type: ExternType,
) -> Result<CallbackHandle, CallbackError>
where
    F: Fn(&[Value]) -> Result<Value, RuntimeError> + 'static,
{
    // Build signature string for error reporting
    let sig = signature_string(&param_types, &return_type);

    // Box the closure and get its raw pointer (transfers ownership)
    let closure: CallbackClosure = Box::new(callback_fn);
    let context = Box::into_raw(Box::new(closure)) as *mut c_void;

    // Select the appropriate trampoline based on signature
    let trampoline: *const () = match (param_types.as_slice(), &return_type) {
        // () -> CInt
        ([], ExternType::CInt) => trampoline_void_to_int as *const (),

        // CDouble -> CDouble
        ([ExternType::CDouble], ExternType::CDouble) => trampoline_double_to_double as *const (),

        // (CDouble, CDouble) -> CDouble
        ([ExternType::CDouble, ExternType::CDouble], ExternType::CDouble) => {
            trampoline_double_double_to_double as *const ()
        }

        // CInt -> CInt
        ([ExternType::CInt], ExternType::CInt) => trampoline_int_to_int as *const (),

        // (CInt, CInt) -> CInt
        ([ExternType::CInt, ExternType::CInt], ExternType::CInt) => {
            trampoline_int_int_to_int as *const ()
        }

        // CLong -> CLong
        ([ExternType::CLong], ExternType::CLong) => trampoline_long_to_long as *const (),

        // () -> CVoid
        ([], ExternType::CVoid) => trampoline_void_to_void as *const (),

        // CInt -> CVoid
        ([ExternType::CInt], ExternType::CVoid) => trampoline_int_to_void as *const (),

        // Unsupported signature - free the context we allocated
        _ => {
            // SAFETY: We just allocated this above, must free it
            unsafe {
                let _ = Box::from_raw(context as *mut CallbackClosure);
            }
            return Err(CallbackError::UnsupportedSignature(sig));
        }
    };

    Ok(CallbackHandle {
        trampoline,
        context,
        param_types,
        return_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // INVOCATION TESTS - These actually CALL the trampolines
    // =========================================================================
    // These tests would crash under the old UB implementation.

    #[test]
    fn test_callback_void_to_int_callable() {
        let handle =
            create_callback(|_args| Ok(Value::Number(42.0)), vec![], ExternType::CInt).unwrap();

        // Actually invoke the trampoline - this is what C code does
        let trampoline: unsafe extern "C" fn(*mut c_void) -> c_int =
            unsafe { std::mem::transmute(handle.trampoline()) };
        let result = unsafe { trampoline(handle.context()) };

        assert_eq!(result, 42);
    }

    #[test]
    fn test_callback_double_to_double_callable() {
        let handle = create_callback(
            |args| {
                if let Some(Value::Number(n)) = args.first() {
                    Ok(Value::Number(n * 2.0))
                } else {
                    Ok(Value::Number(0.0))
                }
            },
            vec![ExternType::CDouble],
            ExternType::CDouble,
        )
        .unwrap();

        let trampoline: unsafe extern "C" fn(*mut c_void, c_double) -> c_double =
            unsafe { std::mem::transmute(handle.trampoline()) };
        let result = unsafe { trampoline(handle.context(), 21.0) };

        assert!((result - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_callback_double_double_to_double_callable() {
        let handle = create_callback(
            |args| {
                if let (Some(Value::Number(x)), Some(Value::Number(y))) =
                    (args.first(), args.get(1))
                {
                    Ok(Value::Number(x + y))
                } else {
                    Ok(Value::Number(0.0))
                }
            },
            vec![ExternType::CDouble, ExternType::CDouble],
            ExternType::CDouble,
        )
        .unwrap();

        let trampoline: unsafe extern "C" fn(*mut c_void, c_double, c_double) -> c_double =
            unsafe { std::mem::transmute(handle.trampoline()) };
        let result = unsafe { trampoline(handle.context(), 17.0, 25.0) };

        assert!((result - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_callback_int_to_int_callable() {
        let handle = create_callback(
            |args| {
                if let Some(Value::Number(n)) = args.first() {
                    Ok(Value::Number((*n as i32 * 3) as f64))
                } else {
                    Ok(Value::Number(0.0))
                }
            },
            vec![ExternType::CInt],
            ExternType::CInt,
        )
        .unwrap();

        let trampoline: unsafe extern "C" fn(*mut c_void, c_int) -> c_int =
            unsafe { std::mem::transmute(handle.trampoline()) };
        let result = unsafe { trampoline(handle.context(), 14) };

        assert_eq!(result, 42);
    }

    #[test]
    fn test_callback_int_int_to_int_callable() {
        let handle = create_callback(
            |args| {
                if let (Some(Value::Number(x)), Some(Value::Number(y))) =
                    (args.first(), args.get(1))
                {
                    Ok(Value::Number((*x as i32 - *y as i32) as f64))
                } else {
                    Ok(Value::Number(0.0))
                }
            },
            vec![ExternType::CInt, ExternType::CInt],
            ExternType::CInt,
        )
        .unwrap();

        let trampoline: unsafe extern "C" fn(*mut c_void, c_int, c_int) -> c_int =
            unsafe { std::mem::transmute(handle.trampoline()) };
        let result = unsafe { trampoline(handle.context(), 100, 58) };

        assert_eq!(result, 42);
    }

    #[test]
    fn test_callback_long_to_long_callable() {
        let handle = create_callback(
            |args| {
                if let Some(Value::Number(n)) = args.first() {
                    Ok(Value::Number(*n + 1.0))
                } else {
                    Ok(Value::Number(0.0))
                }
            },
            vec![ExternType::CLong],
            ExternType::CLong,
        )
        .unwrap();

        let trampoline: unsafe extern "C" fn(*mut c_void, c_long) -> c_long =
            unsafe { std::mem::transmute(handle.trampoline()) };
        let result = unsafe { trampoline(handle.context(), 41) };

        assert_eq!(result, 42);
    }

    #[test]
    fn test_callback_void_to_void_callable() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        let handle = create_callback(
            move |_args| {
                called_clone.store(true, Ordering::SeqCst);
                Ok(Value::Null)
            },
            vec![],
            ExternType::CVoid,
        )
        .unwrap();

        let trampoline: unsafe extern "C" fn(*mut c_void) =
            unsafe { std::mem::transmute(handle.trampoline()) };
        unsafe { trampoline(handle.context()) };

        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_callback_int_to_void_callable() {
        use std::sync::atomic::{AtomicI32, Ordering};
        use std::sync::Arc;

        let received = Arc::new(AtomicI32::new(0));
        let received_clone = received.clone();

        let handle = create_callback(
            move |args| {
                if let Some(Value::Number(n)) = args.first() {
                    received_clone.store(*n as i32, Ordering::SeqCst);
                }
                Ok(Value::Null)
            },
            vec![ExternType::CInt],
            ExternType::CVoid,
        )
        .unwrap();

        let trampoline: unsafe extern "C" fn(*mut c_void, c_int) =
            unsafe { std::mem::transmute(handle.trampoline()) };
        unsafe { trampoline(handle.context(), 42) };

        assert_eq!(received.load(Ordering::SeqCst), 42);
    }

    // =========================================================================
    // ERROR HANDLING TESTS
    // =========================================================================

    #[test]
    fn test_callback_error_returns_default() {
        let handle = create_callback(
            |_args| {
                Err(RuntimeError::TypeError {
                    msg: "callback test error".into(),
                    span: crate::span::Span { start: 0, end: 0 },
                })
            },
            vec![],
            ExternType::CInt,
        )
        .unwrap();

        let trampoline: unsafe extern "C" fn(*mut c_void) -> c_int =
            unsafe { std::mem::transmute(handle.trampoline()) };
        let result = unsafe { trampoline(handle.context()) };

        // On error, should return 0 (default)
        assert_eq!(result, 0);
    }

    #[test]
    fn test_callback_wrong_return_type_returns_default() {
        use std::sync::Arc;
        // Callback returns String instead of Number
        let handle = create_callback(
            |_args| Ok(Value::String(Arc::new("not a number".to_string()))),
            vec![],
            ExternType::CInt,
        )
        .unwrap();

        let trampoline: unsafe extern "C" fn(*mut c_void) -> c_int =
            unsafe { std::mem::transmute(handle.trampoline()) };
        let result = unsafe { trampoline(handle.context()) };

        // Wrong type should return 0 (default)
        assert_eq!(result, 0);
    }

    // =========================================================================
    // STRUCTURE TESTS
    // =========================================================================

    #[test]
    fn test_callback_handle_has_valid_pointers() {
        let handle =
            create_callback(|_args| Ok(Value::Number(0.0)), vec![], ExternType::CInt).unwrap();

        assert!(!handle.trampoline().is_null());
        assert!(!handle.context().is_null());
    }

    #[test]
    fn test_callback_signature() {
        let handle = create_callback(
            |_args| Ok(Value::Number(0.0)),
            vec![ExternType::CInt, ExternType::CDouble],
            ExternType::CLong,
        );

        // This signature is not supported
        assert!(matches!(
            handle,
            Err(CallbackError::UnsupportedSignature(_))
        ));
    }

    #[test]
    fn test_callback_unsupported_signature() {
        let result = create_callback(
            |_args| Ok(Value::Null),
            vec![ExternType::CCharPtr],
            ExternType::CCharPtr,
        );

        assert!(matches!(
            result,
            Err(CallbackError::UnsupportedSignature(_))
        ));
    }

    #[test]
    fn test_callback_signature_string() {
        let sig = signature_string(&[ExternType::CInt, ExternType::CDouble], &ExternType::CLong);
        assert!(sig.contains("CInt"));
        assert!(sig.contains("CDouble"));
        assert!(sig.contains("CLong"));
    }

    // =========================================================================
    // DROP TEST - Verify closure is freed
    // =========================================================================

    #[test]
    fn test_callback_drop_frees_closure() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        struct DropTracker {
            dropped: Arc<AtomicBool>,
        }

        impl Drop for DropTracker {
            fn drop(&mut self) {
                self.dropped.store(true, Ordering::SeqCst);
            }
        }

        let dropped = Arc::new(AtomicBool::new(false));
        let tracker = DropTracker {
            dropped: dropped.clone(),
        };

        {
            let handle = create_callback(
                move |_args| {
                    // Capture the tracker to ensure it's dropped with the closure
                    let _ = &tracker;
                    Ok(Value::Number(0.0))
                },
                vec![],
                ExternType::CInt,
            )
            .unwrap();

            // Verify tracker hasn't been dropped yet
            assert!(!dropped.load(Ordering::SeqCst));

            // Use handle to prevent optimization
            assert!(!handle.trampoline().is_null());
        }

        // After handle is dropped, the closure (and tracker) should be freed
        assert!(dropped.load(Ordering::SeqCst));
    }
}
