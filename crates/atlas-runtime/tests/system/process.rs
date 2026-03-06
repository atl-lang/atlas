use super::*;
use pretty_assertions::assert_eq;

// Command Execution Tests
// ============================================================================

#[test]
fn test_exec_simple_command() {
    // Test executing a simple command (echo on Unix, similar on Windows)
    let code = if cfg!(target_os = "windows") {
        r#"exec(["cmd", "/C", "echo", "hello"])"#
    } else {
        r#"exec(["echo", "hello"])"#
    };

    let result = eval_ok(code);
    // Should return Result<object, string>
    assert!(matches!(result, Value::Result(_)));
}

#[test]
fn test_shell_command() {
    let code = r#"shell("echo hello")"#;

    let result = eval_ok(code);
    // Should return Result<object, string>
    assert!(matches!(result, Value::Result(_)));
}

// ============================================================================
// Environment Variable Tests
// ============================================================================

#[test]
fn test_set_get_env() {
    let code = r#"
        set_env("TEST_VAR_ATLAS", "test_value");
        get_env("TEST_VAR_ATLAS")
    "#;
    let result = eval_ok(code);
    match result {
        Value::Option(Some(inner)) => match *inner {
            Value::String(s) => assert_eq!(&*s, "test_value"),
            other => panic!(
                "Expected Option(Some(String)), got Option(Some({:?}))",
                other
            ),
        },
        other => panic!("Expected Option(Some(String(...))), got {:?}", other),
    }
}

#[test]
fn test_get_env_nonexistent() {
    let code = r#"get_env("NONEXISTENT_VAR_ATLAS_12345")"#;
    let result = eval_ok(code);
    assert!(matches!(result, Value::Option(None)));
}

#[test]
fn test_unset_env() {
    let code = r#"
        set_env("TEST_VAR_UNSET", "value");
        unset_env("TEST_VAR_UNSET");
        get_env("TEST_VAR_UNSET")
    "#;
    let result = eval_ok(code);
    assert!(matches!(result, Value::Option(None)));
}

#[test]
fn test_list_env() {
    let code = r#"list_env()"#;
    let result = eval_ok(code);
    // Should return an object (JsonValue)
    assert!(matches!(result, Value::JsonValue(_)));
}

// ============================================================================
// Working Directory Tests
// ============================================================================

#[test]
fn test_get_cwd() {
    let code = r#"get_cwd()"#;
    let result = eval_ok(code);
    // Should return a string
    assert!(matches!(result, Value::String(_)));
}

// ============================================================================
// Process Info Tests
// ============================================================================

#[test]
fn test_get_pid() {
    let code = r#"get_pid()"#;
    let result = eval_ok(code);
    // Should return a number
    match result {
        Value::Number(n) => assert!(n > 0.0),
        other => panic!("Expected Number, got {:?}", other),
    }
}

// ============================================================================
// Async Process Management Tests
// ============================================================================

#[test]
fn test_spawn_process_output() {
    let code = if cfg!(target_os = "windows") {
        r#"
        let handle = spawn_process(["cmd", "/C", "echo", "hello"]);
        let mut status = process_wait(handle);
        while (is_err(status)) {
            status = process_wait(handle);
        }
        let output = unwrap(process_output(handle));
        trim(output)
    "#
    } else {
        r#"
        let handle = spawn_process(["sh", "-c", "echo hello"]);
        let mut status = process_wait(handle);
        while (is_err(status)) {
            status = process_wait(handle);
        }
        let output = unwrap(process_output(handle));
        trim(output)
    "#
    };

    let result = eval_ok(code);
    match result {
        Value::String(s) => assert_eq!(&*s, "hello"),
        other => panic!("Expected string output, got {:?}", other),
    }
}

#[test]
fn test_spawn_process_kill() {
    let code = if cfg!(target_os = "windows") {
        r#"
        let handle = spawn_process(["cmd", "/C", "timeout", "/T", "5", "/NOBREAK"]);
        let was_running = process_is_running(handle);
        unwrap(process_kill(handle, 9));
        let mut status = process_wait(handle);
        while (is_err(status)) {
            status = process_wait(handle);
        }
        let still_running = process_is_running(handle);
        [was_running, still_running]
    "#
    } else {
        r#"
        let handle = spawn_process(["sh", "-c", "sleep 5"]);
        let was_running = process_is_running(handle);
        unwrap(process_kill(handle, 9));
        let mut status = process_wait(handle);
        while (is_err(status)) {
            status = process_wait(handle);
        }
        let still_running = process_is_running(handle);
        [was_running, still_running]
    "#
    };

    let result = eval_ok(code);
    match result {
        Value::Array(arr) => {
            let arr = arr.as_slice();
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::Bool(true));
            assert_eq!(arr[1], Value::Bool(false));
        }
        other => panic!("Expected array result, got {:?}", other),
    }
}

#[test]
fn test_process_stdio_handles() {
    let code = if cfg!(target_os = "windows") {
        r#"
        let handle = spawn_process(["cmd", "/C", "echo", "hello"]);
        let stdin = process_stdin(handle);
        let stdout = process_stdout(handle);
        let stderr = process_stderr(handle);
        let stdout_again = process_stdout(handle);
        [stdin, stdout, stderr, stdout_again]
    "#
    } else {
        r#"
        let handle = spawn_process(["sh", "-c", "echo hello"]);
        let stdin = process_stdin(handle);
        let stdout = process_stdout(handle);
        let stderr = process_stderr(handle);
        let stdout_again = process_stdout(handle);
        [stdin, stdout, stderr, stdout_again]
    "#
    };

    let result = eval_ok(code);
    let handles = match result {
        Value::Array(arr) => arr,
        other => panic!("Expected array result, got {:?}", other),
    };
    let handles = handles.as_slice();
    assert_eq!(handles.len(), 4);

    let stdout_handle = &handles[1];
    let stdout_again_handle = &handles[3];
    assert_eq!(stdout_handle, stdout_again_handle);

    for (index, handle) in handles.iter().enumerate() {
        let Value::Array(arr) = handle else {
            panic!("Expected handle array at index {}, got {:?}", index, handle);
        };
        let arr = arr.as_slice();
        assert_eq!(arr.len(), 2);
        assert!(matches!(arr[0], Value::String(_)));
        assert!(matches!(arr[1], Value::Number(_)));
    }
}

// ============================================================================
// Security Tests
// ============================================================================

#[test]
fn test_exec_requires_permission() {
    let code = r#"exec("ls")"#;
    // Default context denies all
    let security = SecurityContext::new();
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(code);
    // Should fail due to permission denial
    assert!(result.is_err());
}

#[test]
fn test_env_requires_permission() {
    let code = r#"get_env("PATH")"#;
    // Default context denies all
    let security = SecurityContext::new();
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(code);
    // Should fail due to permission denial
    assert!(result.is_err());
}

// --- Gzip compression ---

// Gzip compression tests
//
// Comprehensive tests for gzip compression and decompression
