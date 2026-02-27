//! THIN ROUTER — DO NOT ADD TESTS HERE.
//! Add tests to the submodule files: tests/system/{path,filesystem,process,compression}.rs
//! This file only declares submodules and shared helpers.

use atlas_runtime::security::SecurityContext;
use atlas_runtime::span::Span;
use atlas_runtime::stdlib;
use atlas_runtime::stdlib::fs;
use atlas_runtime::value::Value;
use atlas_runtime::Atlas;
use std::fs as std_fs;
use std::path::Path;
use tempfile::TempDir;

fn test_span() -> Span {
    Span::dummy()
}

fn span() -> Span {
    Span::dummy()
}

fn call_fn(name: &str, args: &[Value]) -> Result<Value, atlas_runtime::value::RuntimeError> {
    let security = SecurityContext::allow_all();
    stdlib::call_builtin(name, args, test_span(), &security, &stdlib::stdout_writer())
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

fn eval_ok(code: &str) -> Value {
    let security = SecurityContext::allow_all();
    let runtime = Atlas::new_with_security(security);
    runtime.eval(code).unwrap()
}

fn bytes_to_atlas_array(bytes: &[u8]) -> Value {
    let values: Vec<Value> = bytes.iter().map(|&b| Value::Number(b as f64)).collect();
    Value::array(values)
}

fn atlas_array_to_bytes(value: &Value) -> Vec<u8> {
    match value {
        Value::Array(arr) => {
            let arr_guard = arr.as_slice();
            arr_guard
                .iter()
                .map(|v| match v {
                    Value::Number(n) => *n as u8,
                    _ => panic!("Expected number in array"),
                })
                .collect()
        }
        _ => panic!("Expected array"),
    }
}

// Domain submodules (files live in tests/system/)
#[path = "system/compression/mod.rs"]
mod system_compression;
#[path = "system/filesystem.rs"]
mod system_filesystem;
#[path = "system/path.rs"]
mod system_path;
#[path = "system/process.rs"]
mod system_process;
