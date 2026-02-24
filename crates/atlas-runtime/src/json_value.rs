//! JSON Value Type
//!
//! Isolated dynamic type for JSON interop. This is the ONLY exception to
//! Atlas's strict typing - designed specifically for API responses and config files.
//!
//! Design follows Rust's serde_json pattern:
//! - Natural indexing: data["user"]["name"]
//! - Explicit extraction: .as_string(), .as_number()
//! - Safe defaults: missing keys return Null, not errors
//!
//! CRITICAL: JsonValue is isolated - cannot be assigned to non-json variables
//! without explicit extraction.

use std::collections::HashMap;
use std::fmt;

/// JSON value type - isolated dynamic type for JSON interop only
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    /// JSON null
    Null,
    /// JSON boolean
    Bool(bool),
    /// JSON number (IEEE 754 double-precision)
    Number(f64),
    /// JSON string
    String(String),
    /// JSON array
    Array(Vec<JsonValue>),
    /// JSON object (key-value map)
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    /// Create a new JSON object
    pub fn object(map: HashMap<String, JsonValue>) -> Self {
        JsonValue::Object(map)
    }

    /// Create a new JSON array
    pub fn array(values: Vec<JsonValue>) -> Self {
        JsonValue::Array(values)
    }

    /// Check if this value is null
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    /// Check if this value is a boolean
    pub fn is_bool(&self) -> bool {
        matches!(self, JsonValue::Bool(_))
    }

    /// Check if this value is a number
    pub fn is_number(&self) -> bool {
        matches!(self, JsonValue::Number(_))
    }

    /// Check if this value is a string
    pub fn is_string(&self) -> bool {
        matches!(self, JsonValue::String(_))
    }

    /// Check if this value is an array
    pub fn is_array(&self) -> bool {
        matches!(self, JsonValue::Array(_))
    }

    /// Check if this value is an object
    pub fn is_object(&self) -> bool {
        matches!(self, JsonValue::Object(_))
    }

    /// Extract as boolean, returns None if not a bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Extract as number, returns None if not a number
    pub fn as_number(&self) -> Option<f64> {
        match self {
            JsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Extract as string reference, returns None if not a string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Extract as array reference, returns None if not an array
    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Extract as object reference, returns None if not an object
    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Index into an object with a string key
    /// Returns JsonValue::Null if key doesn't exist or value is not an object
    pub fn index_str(&self, key: &str) -> JsonValue {
        match self {
            JsonValue::Object(obj) => obj.get(key).cloned().unwrap_or(JsonValue::Null),
            _ => JsonValue::Null,
        }
    }

    /// Index into an array with a numeric index
    /// Returns JsonValue::Null if index out of bounds or value is not an array
    pub fn index_num(&self, index: f64) -> JsonValue {
        // Convert f64 to usize (truncate, must be non-negative integer)
        if index < 0.0 || index.fract() != 0.0 {
            return JsonValue::Null;
        }

        let idx = index as usize;

        match self {
            JsonValue::Array(arr) => arr.get(idx).cloned().unwrap_or(JsonValue::Null),
            _ => JsonValue::Null,
        }
    }

    /// Get the length of an array or object
    /// Returns None if value is neither array nor object
    pub fn len(&self) -> Option<usize> {
        match self {
            JsonValue::Array(arr) => Some(arr.len()),
            JsonValue::Object(obj) => Some(obj.len()),
            _ => None,
        }
    }

    /// Check if array or object is empty
    /// Returns true for null and non-array/object types
    pub fn is_empty(&self) -> bool {
        match self {
            JsonValue::Array(arr) => arr.is_empty(),
            JsonValue::Object(obj) => obj.is_empty(),
            _ => true,
        }
    }
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Bool(b) => write!(f, "{}", b),
            JsonValue::Number(n) => {
                // Format numbers without trailing .0 for integers
                if n.fract() == 0.0 && n.is_finite() {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            JsonValue::String(s) => write!(f, "\"{}\"", s),
            JsonValue::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            JsonValue::Object(obj) => {
                write!(f, "{{")?;
                let mut first = true;
                for (key, val) in obj {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    write!(f, "\"{}\": {}", key, val)?;
                }
                write!(f, "}}")
            }
        }
    }
}
