//! FFI type system - C-compatible types for FFI boundary
//!
//! Defines:
//! - `ExternType`: Atlas type system representation of C types
//! - `CType`: Runtime representation of C values
//!
//! Type mapping:
//! - ExternType::CInt → CType::Int(i32)
//! - ExternType::CLong → CType::Long(i64)
//! - ExternType::CDouble → CType::Double(f64)
//! - ExternType::CCharPtr → CType::CharPtr(*const i8)
//! - ExternType::CVoid → CType::Void
//! - ExternType::CBool → CType::Bool(u8)

use crate::types::Type;
use serde::{Deserialize, Serialize};
use std::os::raw::c_char;

/// C-compatible extern types for FFI
///
/// These types represent C types in Atlas's type system.
/// Used for type checking extern function declarations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExternType {
    /// C int (platform-specific, typically i32)
    CInt,
    /// C long (platform-specific, i32 on 32-bit, i64 on 64-bit)
    CLong,
    /// C double (f64)
    CDouble,
    /// C char* (null-terminated string pointer)
    CCharPtr,
    /// C void (for void* or void return)
    CVoid,
    /// C bool (u8: 0 or 1)
    CBool,
}

impl ExternType {
    /// Check if an Atlas type can be marshaled to this extern type
    ///
    /// # Examples
    ///
    /// ```
    /// # use atlas_runtime::ffi::ExternType;
    /// # use atlas_runtime::Type;
    /// assert!(ExternType::CInt.accepts_atlas_type(&Type::Number));
    /// assert!(ExternType::CCharPtr.accepts_atlas_type(&Type::String));
    /// assert!(!ExternType::CInt.accepts_atlas_type(&Type::String));
    /// ```
    pub fn accepts_atlas_type(&self, atlas_type: &Type) -> bool {
        matches!(
            (self, atlas_type),
            (ExternType::CInt, Type::Number)
                | (ExternType::CLong, Type::Number)
                | (ExternType::CDouble, Type::Number)
                | (ExternType::CCharPtr, Type::String)
                | (ExternType::CVoid, Type::Void)
                | (ExternType::CBool, Type::Bool)
        )
    }

    /// Get the Atlas type this extern type maps to
    ///
    /// # Examples
    ///
    /// ```
    /// # use atlas_runtime::ffi::ExternType;
    /// # use atlas_runtime::Type;
    /// assert_eq!(ExternType::CInt.to_atlas_type(), Type::Number);
    /// assert_eq!(ExternType::CCharPtr.to_atlas_type(), Type::String);
    /// ```
    pub fn to_atlas_type(&self) -> Type {
        match self {
            ExternType::CInt | ExternType::CLong | ExternType::CDouble => Type::Number,
            ExternType::CCharPtr => Type::String,
            ExternType::CVoid => Type::Void,
            ExternType::CBool => Type::Bool,
        }
    }

    /// Get a display name for this extern type
    pub fn display_name(&self) -> &'static str {
        match self {
            ExternType::CInt => "c_int",
            ExternType::CLong => "c_long",
            ExternType::CDouble => "c_double",
            ExternType::CCharPtr => "c_char_ptr",
            ExternType::CVoid => "c_void",
            ExternType::CBool => "c_bool",
        }
    }
}

/// C type representation for FFI boundary
///
/// Runtime representation of C values during marshaling.
/// These are the actual C-compatible values passed across the FFI boundary.
#[derive(Debug, Clone)]
pub enum CType {
    /// C int value
    Int(i32),
    /// C long value
    Long(i64),
    /// C double value
    Double(f64),
    /// C char* (null-terminated string pointer)
    ///
    /// # Safety
    ///
    /// The pointer must be valid and point to a null-terminated string.
    /// Lifetime managed by MarshalContext.
    CharPtr(*const c_char),
    /// C void (no value)
    Void,
    /// C bool (0 or 1)
    Bool(u8),
}

// Manual PartialEq because we can't derive it for raw pointers
impl PartialEq for CType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CType::Int(a), CType::Int(b)) => a == b,
            (CType::Long(a), CType::Long(b)) => a == b,
            (CType::Double(a), CType::Double(b)) => a == b,
            (CType::CharPtr(a), CType::CharPtr(b)) => a == b,
            (CType::Void, CType::Void) => true,
            (CType::Bool(a), CType::Bool(b)) => a == b,
            _ => false,
        }
    }
}
