//! Runtime value representation

use std::cell::RefCell;
use std::rc::Rc;
use thiserror::Error;

/// Runtime value type
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Null value
    Null,
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// String value (reference-counted)
    String(Rc<String>),
    /// Boolean value
    Bool(bool),
    /// Array value (reference-counted, mutable)
    Array(Rc<RefCell<Vec<Value>>>),
}

impl Value {
    /// Create a new string value
    pub fn string(s: impl Into<String>) -> Self {
        Value::String(Rc::new(s.into()))
    }

    /// Create a new array value
    pub fn array(values: Vec<Value>) -> Self {
        Value::Array(Rc::new(RefCell::new(values)))
    }

    /// Get a string representation of this value
    pub fn to_display_string(&self) -> String {
        match self {
            Value::Null => "null".to_string(),
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => s.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Array(_) => "[...]".to_string(),
        }
    }
}

/// Runtime error type
#[derive(Debug, Error, Clone)]
pub enum RuntimeError {
    /// Type error
    #[error("Type error: {0}")]
    TypeError(String),
    /// Undefined variable
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_creation() {
        let val = Value::Int(42);
        assert_eq!(val.to_display_string(), "42");
    }

    #[test]
    fn test_string_value() {
        let val = Value::string("hello");
        assert_eq!(val.to_display_string(), "hello");
    }

    #[test]
    fn test_array_value() {
        let val = Value::array(vec![Value::Int(1), Value::Int(2)]);
        assert_eq!(val.to_display_string(), "[...]");
    }
}
