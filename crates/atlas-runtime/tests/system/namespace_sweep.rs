use super::*;

// ============================================================================
// B35: Namespace conversion completion — regression guard
// Verifies that all canonical namespaces resolve correctly (no AT1001
// undefined-variable errors). Tests sentinel registration and dispatch for
// every namespace introduced in B20-B35.
//
// Each test calls a minimal, side-effect-free method on the namespace and
// asserts it does NOT panic with "unknown identifier" / dispatch failure.
// Actual return values are not the point — dispatch working is the point.
// ============================================================================

/// Helper: verify that evaluating `code` does not produce an AT1001
/// (undefined variable) error. The eval may succeed or fail for other
/// reasons (e.g. arity errors, missing stdin) — that's fine. Only AT1001
/// means the namespace sentinel was missing.
fn assert_not_at1001(code: &str) {
    use atlas_runtime::api::{EvalError, ExecutionMode, Runtime};
    let mut runtime = Runtime::new();
    match runtime.eval(code) {
        Ok(_) => {}
        Err(EvalError::ParseError(diags)) | Err(EvalError::TypeError(diags)) => {
            for diag in &diags {
                assert!(
                    diag.code != "AT1001",
                    "namespace sentinel missing (AT1001) for: {}\n  diag: {:?}",
                    code,
                    diag
                );
            }
        }
        Err(EvalError::RuntimeError(_)) => {} // runtime error is fine — dispatch worked
    }
}

// --- console ---

#[test]
fn test_namespace_console_log_resolves() {
    // console.log returns Null
    let result = eval_ok("console.log(\"ok\");");
    assert!(
        matches!(result, Value::Null),
        "console.log should return Null"
    );
}

// --- Math ---

#[test]
fn test_namespace_math_sqrt_resolves() {
    assert_not_at1001("Math.sqrt(4.0);");
}

#[test]
fn test_namespace_math_abs_resolves() {
    let result = eval_ok("Math.abs(-5.0);");
    assert!(
        matches!(result, Value::Number(_)),
        "Math.abs should return Number"
    );
}

// --- Json ---

#[test]
fn test_namespace_json_stringify_resolves() {
    let result = eval_ok("Json.stringify(42);");
    assert!(
        matches!(result, Value::String(_)),
        "Json.stringify should return String"
    );
}

#[test]
fn test_namespace_json_parse_resolves() {
    // Json.parse returns JsonValue — assert dispatch works (not AT1001)
    assert_not_at1001("Json.parse(\"42\");");
}

// --- env ---

#[test]
fn test_namespace_env_get_resolves() {
    // env.get returns Option — assert dispatch works (not AT1001)
    assert_not_at1001("env.get(\"PATH\");");
}

// --- file ---

#[test]
fn test_namespace_file_exists_resolves() {
    let result = eval_ok("file.exists(\"/tmp\");");
    assert!(
        matches!(result, Value::Bool(_)),
        "file.exists should return Bool"
    );
}

// --- process ---

#[test]
fn test_namespace_process_cwd_resolves() {
    let result = eval_ok("process.cwd();");
    assert!(
        matches!(result, Value::String(_)),
        "process.cwd should return String"
    );
}

// --- datetime ---

#[test]
fn test_namespace_datetime_now_resolves() {
    // datetime.now() returns a struct/record — assert dispatch works (not AT1001)
    assert_not_at1001("datetime.now();");
}

// --- path ---

#[test]
fn test_namespace_path_join_resolves() {
    let result = eval_ok("path.join(\"/tmp\", \"foo\");");
    assert!(
        matches!(result, Value::String(_)),
        "path.join should return String"
    );
}

// --- encoding ---

#[test]
fn test_namespace_encoding_base64_encode_resolves() {
    // Assert sentinel resolves (no AT1001) — arity dispatch details tested in stdlib/encoding tests
    assert_not_at1001("encoding.base64Encode(\"hello\");");
}

// --- regex ---

#[test]
fn test_namespace_regex_is_match_resolves() {
    assert_not_at1001("regex.isMatch(regex.new(\"a+\"), \"aaa\");");
}

// --- io ---

#[test]
fn test_namespace_io_write_resolves() {
    // Assert sentinel resolves (no AT1001) — io.write typechecker return type filed separately
    assert_not_at1001("io.write(\"\");");
}

// --- gzip ---

#[test]
fn test_namespace_gzip_compress_resolves() {
    let result = eval_ok("gzip.compress([104, 105]);");
    assert!(
        matches!(result, Value::Array(_)),
        "gzip.compress should return Array (bytes), got {:?}",
        result
    );
}

// --- tar ---

#[test]
fn test_namespace_tar_list_resolves() {
    assert_not_at1001("tar.list(\"/nonexistent.tar\");");
}

// --- zip ---

#[test]
fn test_namespace_zip_list_resolves() {
    assert_not_at1001("zip.list(\"/nonexistent.zip\");");
}

// --- task ---

#[test]
fn test_namespace_task_sleep_resolves() {
    let result = eval_ok("task.sleep(0);");
    assert!(
        matches!(result, Value::Future(_)),
        "task.sleep should return Future"
    );
}

// --- sync ---

#[test]
fn test_namespace_sync_atomic_resolves() {
    let result = eval_ok("sync.atomic(0);");
    // atomic_new returns Value::Array([tag, id]) as a handle
    assert!(
        matches!(result, Value::Array(_)),
        "sync.atomic should return an Array handle"
    );
}

// --- future ---

#[test]
fn test_namespace_future_resolve_resolves() {
    let result = eval_ok("future.resolve(42);");
    assert!(
        matches!(result, Value::Future(_)),
        "future.resolve should return Future"
    );
}

// --- test namespace ---

#[test]
fn test_namespace_test_assert_resolves() {
    let result = eval_ok("test.assert(true, \"should pass\");");
    assert!(
        matches!(result, Value::Null | Value::Bool(_)),
        "test.assert should not panic"
    );
}
