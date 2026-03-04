use super::*;
fn span() -> Span {
    Span::dummy()
}

fn extract_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.as_ref().clone(),
        _ => panic!("Expected string value"),
    }
}

fn extract_number(value: &Value) -> f64 {
    match value {
        Value::Number(n) => *n,
        _ => panic!("Expected number value"),
    }
}

fn extract_bool(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        _ => panic!("Expected bool value"),
    }
}

fn extract_array(value: &Value) -> Vec<Value> {
    match value {
        Value::Array(arr) => arr.as_slice().to_vec(),
        _ => panic!("Expected array value"),
    }
}

mod dir_ops;
mod edge;
mod metadata;
mod symlink;
mod temp;
mod watch;
