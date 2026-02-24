//! Hash function infrastructure for Atlas collections
//!
//! Provides deterministic hashing for Atlas values using Rust's DefaultHasher.
//! Only primitive types (number, string, bool, null) are hashable.

use crate::span::Span;
use crate::value::{RuntimeError, Value};
use ordered_float::OrderedFloat;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Wrapper type for hashable Atlas values
///
/// Only Number, String, Bool, Null can be hashed.
/// Arrays, functions, JsonValue, Option, Result are not hashable.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HashKey {
    /// Number value with IEEE 754 canonicalization
    Number(OrderedFloat<f64>),
    /// String value (reference-counted)
    String(Arc<String>),
    /// Boolean value
    Bool(bool),
    /// Null value
    Null,
}

impl HashKey {
    /// Create HashKey from Value, returns error if not hashable
    ///
    /// # Errors
    /// Returns `RuntimeError::UnhashableType` if value cannot be hashed
    pub fn from_value(value: &Value, span: Span) -> Result<Self, RuntimeError> {
        match value {
            Value::Number(n) => {
                // Canonicalize NaN to ensure consistent hashing
                // All NaN values hash to the same value
                let normalized = if n.is_nan() { f64::NAN } else { *n };
                Ok(HashKey::Number(OrderedFloat(normalized)))
            }
            Value::String(s) => Ok(HashKey::String(Arc::clone(s))),
            Value::Bool(b) => Ok(HashKey::Bool(*b)),
            Value::Null => Ok(HashKey::Null),
            _ => Err(RuntimeError::UnhashableType {
                type_name: value.type_name().to_string(),
                span,
            }),
        }
    }

    /// Convert HashKey back to Value
    pub fn to_value(&self) -> Value {
        match self {
            HashKey::Number(n) => Value::Number(n.0),
            HashKey::String(s) => Value::String(Arc::clone(s)),
            HashKey::Bool(b) => Value::Bool(*b),
            HashKey::Null => Value::Null,
        }
    }
}

/// Compute deterministic hash for a HashKey
///
/// Uses Rust's DefaultHasher for reproducible hash values.
/// Same input always produces the same output (AI-friendly testing).
pub fn compute_hash(key: &HashKey) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}
