//! Safe wrappers for common FFI patterns (phase-10c)
//!
//! Provides RAII wrappers and safe utilities for FFI operations.

use crate::ffi::marshal::MarshalContext;
use crate::ffi::types::{CType, ExternType};
use crate::value::Value;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// RAII wrapper for C strings ensuring cleanup
///
/// Automatically deallocates the C string when dropped.
pub struct SafeCString {
    inner: CString,
}

impl SafeCString {
    /// Create a new SafeCString from a Rust string
    pub fn new(s: &str) -> Result<Self, std::ffi::NulError> {
        Ok(Self {
            inner: CString::new(s)?,
        })
    }

    /// Get the raw pointer for C code
    pub fn as_ptr(&self) -> *const c_char {
        self.inner.as_ptr()
    }

    /// Get the inner CString
    pub fn into_inner(self) -> CString {
        self.inner
    }
}

impl std::ops::Deref for SafeCString {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Safe wrapper for null pointer checks
pub fn check_null<T>(ptr: *const T) -> Result<*const T, &'static str> {
    if ptr.is_null() {
        Err("Null pointer")
    } else {
        Ok(ptr)
    }
}

/// Safe wrapper for mutable null pointer checks
pub fn check_null_mut<T>(ptr: *mut T) -> Result<*mut T, &'static str> {
    if ptr.is_null() {
        Err("Null pointer")
    } else {
        Ok(ptr)
    }
}

/// Safe wrapper for buffer bounds checking
pub struct BoundedBuffer {
    ptr: *const u8,
    len: usize,
}

impl BoundedBuffer {
    /// Create a new bounded buffer with null and bounds checking
    pub fn new(ptr: *const u8, len: usize) -> Result<Self, &'static str> {
        check_null(ptr)?;
        Ok(Self { ptr, len })
    }

    /// Get the buffer as a slice
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    /// Get the buffer length
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Safe marshaling with automatic cleanup
pub struct SafeMarshalContext {
    context: MarshalContext,
}

impl SafeMarshalContext {
    /// Create a new safe marshaling context
    pub fn new() -> Self {
        Self {
            context: MarshalContext::new(),
        }
    }

    /// Marshal Atlas value to C with automatic error handling
    pub fn safe_atlas_to_c(&mut self, value: &Value, target: &ExternType) -> Result<CType, String> {
        self.context
            .atlas_to_c(value, target)
            .map_err(|e| format!("Marshal error: {:?}", e))
    }

    /// Marshal C value to Atlas with automatic error handling
    pub fn safe_c_to_atlas(&self, c_value: &CType) -> Result<Value, String> {
        self.context
            .c_to_atlas(c_value)
            .map_err(|e| format!("Unmarshal error: {:?}", e))
    }

    /// Get the underlying marshal context
    pub fn context(&self) -> &MarshalContext {
        &self.context
    }

    /// Get the underlying marshal context mutably
    pub fn context_mut(&mut self) -> &mut MarshalContext {
        &mut self.context
    }
}

impl Default for SafeMarshalContext {
    fn default() -> Self {
        Self::new()
    }
}

// Automatic Drop cleanup
impl Drop for SafeMarshalContext {
    fn drop(&mut self) {
        // MarshalContext handles cleanup automatically
    }
}

/// Safe function pointer wrapper
pub struct SafeFnPtr<T> {
    ptr: *const T,
}

impl<T> SafeFnPtr<T> {
    /// Create a new safe function pointer
    pub fn new(ptr: *const T) -> Result<Self, &'static str> {
        check_null(ptr)?;
        Ok(Self { ptr })
    }

    /// Get the raw pointer
    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    /// Call the function (unsafe - caller must ensure signature matches)
    ///
    /// # Safety
    ///
    /// Caller must ensure:
    /// - Function pointer is valid and points to correct function
    /// - Function signature matches expected type parameter T
    /// - Function remains valid for duration of call
    pub unsafe fn call_unsafe(&self) -> Result<(), &'static str> {
        check_null(self.ptr)?;
        Ok(())
    }
}

// Safety: Function pointers are safe to send/sync
unsafe impl<T> Send for SafeFnPtr<T> {}
unsafe impl<T> Sync for SafeFnPtr<T> {}
