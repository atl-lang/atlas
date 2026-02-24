//! Type conversion between Rust and Atlas values
//!
//! Provides traits and implementations for bidirectional conversion:
//! - `ToAtlas` - Convert Rust types to Atlas `Value`
//! - `FromAtlas` - Convert Atlas `Value` to Rust types
//!
//! # Examples
//!
//! ```
//! use atlas_runtime::api::{ToAtlas, FromAtlas};
//! use atlas_runtime::Value;
//!
//! // Rust to Atlas
//! let atlas_value: Value = 42.0.to_atlas();
//! let atlas_string: Value = "hello".to_string().to_atlas();
//!
//! // Atlas to Rust
//! let rust_number: f64 = FromAtlas::from_atlas(&atlas_value).unwrap();
//! let rust_string: String = FromAtlas::from_atlas(&atlas_string).unwrap();
//! ```

use crate::value::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Error type for value conversion failures
#[derive(Debug, Clone, PartialEq)]
pub enum ConversionError {
    /// Type mismatch during conversion
    TypeMismatch { expected: String, found: String },
    /// Array element type mismatch
    ArrayElementTypeMismatch {
        index: usize,
        expected: String,
        found: String,
    },
    /// Object value type mismatch
    ObjectValueTypeMismatch {
        key: String,
        expected: String,
        found: String,
    },
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::TypeMismatch { expected, found } => {
                write!(f, "Type mismatch: expected {}, found {}", expected, found)
            }
            ConversionError::ArrayElementTypeMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "Array element type mismatch at index {}: expected {}, found {}",
                index, expected, found
            ),
            ConversionError::ObjectValueTypeMismatch {
                key,
                expected,
                found,
            } => write!(
                f,
                "Object value type mismatch for key '{}': expected {}, found {}",
                key, expected, found
            ),
        }
    }
}

impl std::error::Error for ConversionError {}

/// Trait for converting Atlas `Value` to Rust types
pub trait FromAtlas: Sized {
    /// Convert from Atlas `Value` to Rust type
    ///
    /// # Errors
    ///
    /// Returns `ConversionError` if the value cannot be converted to the target type.
    fn from_atlas(value: &Value) -> Result<Self, ConversionError>;
}

/// Trait for converting Rust types to Atlas `Value`
pub trait ToAtlas {
    /// Convert from Rust type to Atlas `Value`
    fn to_atlas(self) -> Value;
}

// Helper function to get type name for error messages
fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Function(_) => "function",
        Value::Builtin(_) => "builtin",
        Value::NativeFunction(_) => "function",
        Value::JsonValue(_) => "json",
        Value::Option(_) => "option",
        Value::Result(_) => "result",
        Value::HashMap(_) => "hashmap",
        Value::HashSet(_) => "hashset",
        Value::Queue(_) => "queue",
        Value::Stack(_) => "stack",
        Value::Regex(_) => "regex",
        Value::DateTime(_) => "datetime",
        Value::HttpRequest(_) => "HttpRequest",
        Value::HttpResponse(_) => "HttpResponse",
        Value::Future(_) => "future",
        Value::TaskHandle(_) => "TaskHandle",
        Value::ChannelSender(_) => "ChannelSender",
        Value::ChannelReceiver(_) => "ChannelReceiver",
        Value::AsyncMutex(_) => "AsyncMutex",
        Value::Closure(_) => "closure",
        Value::SharedValue(_) => "shared",
    }
}

// Implementations for f64 (number)

impl FromAtlas for f64 {
    fn from_atlas(value: &Value) -> Result<Self, ConversionError> {
        match value {
            Value::Number(n) => Ok(*n),
            _ => Err(ConversionError::TypeMismatch {
                expected: "number".to_string(),
                found: type_name(value).to_string(),
            }),
        }
    }
}

impl ToAtlas for f64 {
    fn to_atlas(self) -> Value {
        Value::Number(self)
    }
}

// Implementations for String

impl FromAtlas for String {
    fn from_atlas(value: &Value) -> Result<Self, ConversionError> {
        match value {
            Value::String(s) => Ok(s.to_string()),
            _ => Err(ConversionError::TypeMismatch {
                expected: "string".to_string(),
                found: type_name(value).to_string(),
            }),
        }
    }
}

impl ToAtlas for String {
    fn to_atlas(self) -> Value {
        Value::String(Arc::from(self))
    }
}

// Implementations for bool

impl FromAtlas for bool {
    fn from_atlas(value: &Value) -> Result<Self, ConversionError> {
        match value {
            Value::Bool(b) => Ok(*b),
            _ => Err(ConversionError::TypeMismatch {
                expected: "bool".to_string(),
                found: type_name(value).to_string(),
            }),
        }
    }
}

impl ToAtlas for bool {
    fn to_atlas(self) -> Value {
        Value::Bool(self)
    }
}

// Implementations for () (null)

impl FromAtlas for () {
    fn from_atlas(value: &Value) -> Result<Self, ConversionError> {
        match value {
            Value::Null => Ok(()),
            _ => Err(ConversionError::TypeMismatch {
                expected: "null".to_string(),
                found: type_name(value).to_string(),
            }),
        }
    }
}

impl ToAtlas for () {
    fn to_atlas(self) -> Value {
        Value::Null
    }
}

// Implementations for Option<T>

impl<T: FromAtlas> FromAtlas for Option<T> {
    fn from_atlas(value: &Value) -> Result<Self, ConversionError> {
        match value {
            Value::Null => Ok(None),
            _ => Ok(Some(T::from_atlas(value)?)),
        }
    }
}

impl<T: ToAtlas> ToAtlas for Option<T> {
    fn to_atlas(self) -> Value {
        match self {
            None => Value::Null,
            Some(v) => v.to_atlas(),
        }
    }
}

// Implementations for Vec<T> (array)

impl<T: FromAtlas> FromAtlas for Vec<T> {
    fn from_atlas(value: &Value) -> Result<Self, ConversionError> {
        match value {
            Value::Array(arr) => {
                let arr_slice = arr.as_slice();
                let mut result = Vec::with_capacity(arr_slice.len());
                for (index, elem) in arr_slice.iter().enumerate() {
                    match T::from_atlas(elem) {
                        Ok(converted) => result.push(converted),
                        Err(ConversionError::TypeMismatch { expected, found }) => {
                            return Err(ConversionError::ArrayElementTypeMismatch {
                                index,
                                expected,
                                found,
                            });
                        }
                        Err(e) => return Err(e),
                    }
                }
                Ok(result)
            }
            _ => Err(ConversionError::TypeMismatch {
                expected: "array".to_string(),
                found: type_name(value).to_string(),
            }),
        }
    }
}

impl<T: ToAtlas> ToAtlas for Vec<T> {
    fn to_atlas(self) -> Value {
        let values: Vec<Value> = self.into_iter().map(|v| v.to_atlas()).collect();
        Value::array(values)
    }
}

// Implementations for HashMap<String, T> (object)

impl<T: FromAtlas> FromAtlas for HashMap<String, T> {
    fn from_atlas(value: &Value) -> Result<Self, ConversionError> {
        // Atlas doesn't have a native object type (other than JsonValue)
        // This is primarily for JsonValue object conversion
        // For now, return type mismatch error
        Err(ConversionError::TypeMismatch {
            expected: "object".to_string(),
            found: type_name(value).to_string(),
        })
    }
}

impl<T: ToAtlas> ToAtlas for HashMap<String, T> {
    fn to_atlas(self) -> Value {
        // Atlas doesn't have a native object/map type
        // We can't convert HashMap to Value directly
        // This would require creating an array of key-value pairs or using JsonValue
        // For v0.2, we'll create a JsonValue object
        use crate::json_value::JsonValue as JV;

        let mut obj = HashMap::new();
        for (key, value) in self {
            // Convert value to Atlas Value first, then try to convert to JsonValue
            let atlas_value = value.to_atlas();
            let json_value = match atlas_value {
                Value::Null => JV::Null,
                Value::Bool(b) => JV::Bool(b),
                Value::Number(n) => JV::Number(n),
                Value::String(s) => JV::String(s.to_string()),
                Value::Array(arr) => {
                    // Convert array to JSON array
                    let json_arr: Vec<JV> = arr
                        .as_slice()
                        .iter()
                        .map(|v| match v {
                            Value::Null => JV::Null,
                            Value::Bool(b) => JV::Bool(*b),
                            Value::Number(n) => JV::Number(*n),
                            Value::String(s) => JV::String(s.to_string()),
                            _ => JV::Null, // Can't convert functions, nested arrays, etc.
                        })
                        .collect();
                    JV::Array(json_arr)
                }
                _ => JV::Null, // Can't convert functions, JsonValue, etc.
            };
            obj.insert(key, json_value);
        }
        Value::JsonValue(Arc::new(JV::Object(obj)))
    }
}

// Convenience implementations for reference types

impl ToAtlas for &str {
    fn to_atlas(self) -> Value {
        Value::String(Arc::new(self.to_string()))
    }
}

impl ToAtlas for &String {
    fn to_atlas(self) -> Value {
        Value::String(Arc::new(self.clone()))
    }
}
