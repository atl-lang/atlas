//! THIN ROUTER — DO NOT ADD TESTS HERE.
//! Add tests to the submodule files: tests/stdlib/{integration,strings,json,io,types,functions,collections,parity,vm_stdlib,docs_verification}.rs
//! This file only declares submodules and shared helpers.

mod common;

use atlas_runtime::diagnostic::Diagnostic;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::span::Span;
use atlas_runtime::stdlib::test as atlas_test;
use atlas_runtime::stdlib::{call_builtin, is_builtin, stdout_writer};
use atlas_runtime::typechecker::TypeChecker;
use atlas_runtime::value::{RuntimeError, Value};
use atlas_runtime::{Atlas, Binder, SecurityContext};
use common::{
    assert_error_code, assert_eval_bool, assert_eval_null, assert_eval_number, assert_eval_string,
    assert_has_error, path_for_atlas, temp_file_path,
};
use rstest::rstest;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

// Shared helpers used by submodules via `use super::*;`
fn assert_eval_number_with_io(source: &str, expected: f64) {
    let security = SecurityContext::allow_all();
    let runtime = Atlas::new_with_security(security);
    match runtime.eval(source) {
        Ok(Value::Number(n)) if (n - expected).abs() < f64::EPSILON => {}
        Ok(val) => panic!("Expected number {}, got {:?}", expected, val),
        Err(e) => panic!("Execution error: {:?}", e),
    }
}

fn assert_eval_bool_with_io(source: &str, expected: bool) {
    let security = SecurityContext::allow_all();
    let runtime = Atlas::new_with_security(security);
    match runtime.eval(source) {
        Ok(Value::Bool(b)) if b == expected => {}
        Ok(val) => panic!("Expected bool {}, got {:?}", expected, val),
        Err(e) => panic!("Execution error: {:?}", e),
    }
}

fn assert_eval_string_with_io(source: &str, expected: &str) {
    let security = SecurityContext::allow_all();
    let runtime = Atlas::new_with_security(security);
    match runtime.eval(source) {
        Ok(Value::String(s)) if s.as_ref() == expected => {}
        Ok(val) => panic!("Expected string '{}', got {:?}", expected, val),
        Err(e) => panic!("Execution error: {:?}", e),
    }
}

// Domain submodules (files live in tests/stdlib/)
#[path = "stdlib/collections.rs"]
mod collections;
#[path = "stdlib/docs_verification.rs"]
mod docs_verification;
#[path = "stdlib/functions.rs"]
mod functions;
#[path = "stdlib/integration.rs"]
mod integration;
#[path = "stdlib/io.rs"]
mod io;
#[path = "stdlib/json.rs"]
mod json;
#[path = "stdlib/parity.rs"]
mod parity;
#[path = "stdlib/strings.rs"]
mod strings;
#[path = "stdlib/types.rs"]
mod types;
#[path = "stdlib/vm_stdlib.rs"]
mod vm_stdlib;
