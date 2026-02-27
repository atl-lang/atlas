// Thin router for compression tests - subdirectory split for maintainability
// Original file: system/compression.rs (2516 lines / 80KB) split into 8 domain files

use atlas_runtime::security::SecurityContext;
use atlas_runtime::value::Value;
use atlas_runtime::Atlas;
use std::fs as std_fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Import test helpers from parent for submodules to use via `use super::*`
use super::{
    atlas_array_to_bytes, bytes_to_atlas_array, call_fn, extract_bool, extract_number, span,
};

// Test helpers for tar/zip tests
fn create_test_file(dir: &std::path::Path, name: &str, content: &str) {
    let path = dir.join(name);
    std_fs::write(path, content).unwrap();
}

fn create_test_dir(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std_fs::create_dir(&path).unwrap();
    path
}

fn str_value(s: &str) -> Value {
    Value::string(s.to_string())
}

fn str_array_value(paths: &[&str]) -> Value {
    let values: Vec<Value> = paths.iter().map(|p| str_value(p)).collect();
    Value::array(values)
}

fn num_value(n: f64) -> Value {
    Value::Number(n)
}

// IO/FS test helpers (used by io_fs_hardening submodule)
#[allow(dead_code)]
fn with_io() -> Atlas {
    Atlas::new_with_security(SecurityContext::allow_all())
}

#[allow(dead_code)]
fn eval_str_io(code: &str) -> String {
    match with_io().eval(code) {
        Ok(v) => v.to_string(),
        Err(e) => panic!("Expected success, got error: {:?}", e),
    }
}

#[allow(dead_code)]
fn eval_err_io(code: &str) -> bool {
    with_io().eval(code).is_err()
}

// Domain submodules
mod gzip;
mod io_fs_hardening;
mod tar_creation;
mod tar_extraction;
mod tar_utils_errors;
mod zip_advanced;
mod zip_creation;
mod zip_extraction_utils;
