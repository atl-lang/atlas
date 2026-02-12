//! Standard library functions

use crate::value::{RuntimeError, Value};

/// Print a value to stdout
pub fn print(value: &Value) {
    println!("{}", value.to_display_string());
}

/// Get the length of a string or array
pub fn len(value: &Value) -> Result<i64, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.len() as i64),
        Value::Array(arr) => Ok(arr.borrow().len() as i64),
        _ => Err(RuntimeError::TypeError(
            "len() requires string or array".to_string(),
        )),
    }
}

/// Convert a value to a string
pub fn str(value: &Value) -> String {
    value.to_display_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_len_string() {
        let val = Value::string("hello");
        assert_eq!(len(&val).unwrap(), 5);
    }

    #[test]
    fn test_len_array() {
        let val = Value::array(vec![Value::Int(1), Value::Int(2)]);
        assert_eq!(len(&val).unwrap(), 2);
    }

    #[test]
    fn test_str() {
        let val = Value::Int(42);
        assert_eq!(str(&val), "42");
    }
}
