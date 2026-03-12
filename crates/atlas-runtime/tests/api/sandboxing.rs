use super::*;

mod sandboxing_loops;

// --- Sandboxing ---

// Tests for Runtime sandboxing and configuration

/// A thin Write wrapper around Arc<Mutex<Vec<u8>>> for capturing output in integration tests.
struct VecWriter(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for VecWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_runtime_captures_print_output() {
    use atlas_runtime::stdlib::OutputWriter;
    let buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let output: OutputWriter = Arc::new(Mutex::new(Box::new(VecWriter(buf.clone()))));
    let config = RuntimeConfig::new().with_output(output);
    let mut runtime = Runtime::from_config(config);
    runtime.eval(r#"console.log("captured")"#).unwrap();
    let s = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert_eq!(s, "captured\n");
}

#[test]
fn test_default_config_is_permissive() {
    let config = RuntimeConfig::default();
    assert!(config.allow_io);
    assert!(config.allow_network);
    assert!(config.max_execution_time.is_none());
    assert!(config.max_memory_bytes.is_none());
}

#[test]
fn test_sandboxed_config_is_restrictive() {
    let config = RuntimeConfig::sandboxed();
    assert!(!config.allow_io);
    assert!(!config.allow_network);
    assert_eq!(config.max_execution_time, Some(Duration::from_secs(5)));
    assert_eq!(config.max_memory_bytes, Some(10_000_000));
}

#[test]
fn test_custom_config_fluent_api() {
    let config = RuntimeConfig::new()
        .with_max_execution_time(Duration::from_secs(30))
        .with_max_memory_bytes(100_000_000)
        .with_io_allowed(false)
        .with_network_allowed(true);

    assert_eq!(config.max_execution_time, Some(Duration::from_secs(30)));
    assert_eq!(config.max_memory_bytes, Some(100_000_000));
    assert!(!config.allow_io);
    assert!(config.allow_network);
}

#[test]
fn test_runtime_with_sandboxed_config() {
    let mut runtime = Runtime::sandboxed();

    // Basic expressions should still work
    let result = runtime.eval("1 + 2").unwrap();
    assert_eq!(result.to_string(), "3");
}

#[test]
fn test_runtime_with_custom_config() {
    let config = RuntimeConfig::new()
        .with_io_allowed(false)
        .with_network_allowed(false);

    let mut runtime = Runtime::from_config(config);

    // Basic expressions should work
    let result = runtime.eval("let x: number = 42; x").unwrap();
    assert_eq!(result.to_string(), "42");
}

#[test]
fn test_sandboxed_runtime_basic_arithmetic() {
    let mut runtime = Runtime::sandboxed();
    let result = runtime.eval("10 * 5 + 3").unwrap();
    assert_eq!(result.to_string(), "53");
}

#[test]
fn test_sandboxed_runtime_function_definitions() {
    let mut runtime = Runtime::sandboxed();
    runtime
        .eval("fn add(borrow a: number, borrow b: number): number { return a + b; }")
        .unwrap();

    let result = runtime.eval("add(10, 20)").unwrap();
    assert_eq!(result.to_string(), "30");
}

#[test]
fn test_sandboxed_runtime_string_operations() {
    let mut runtime = Runtime::sandboxed();
    let result = runtime.eval(r#""Hello, " + "World!""#).unwrap();
    assert_eq!(result.to_string(), "Hello, World!");
}

#[test]
fn test_sandboxed_runtime_array_operations() {
    let mut runtime = Runtime::sandboxed();
    let result = runtime.eval("len([1, 2, 3])").unwrap();
    assert_eq!(result.to_string(), "3");
}

#[test]
fn test_sandboxed_runtime_conditionals() {
    let mut runtime = Runtime::sandboxed();
    runtime
        .eval(
            r#"
        fn maximum(borrow a: number, borrow b: number): number {
            if (a > b) {
                return a;
            } else {
                return b;
            }
        }
        "#,
        )
        .unwrap();

    let result = runtime.eval("maximum(10, 20)").unwrap();
    assert_eq!(result.to_string(), "20");
}

#[test]
fn test_sandboxed_runtime_native_functions() {
    let mut runtime = Runtime::sandboxed();

    runtime.register_function("double", 1, |args| {
        if let atlas_runtime::value::Value::Number(n) = &args[0] {
            Ok(atlas_runtime::value::Value::Number(n * 2.0))
        } else {
            Err(atlas_runtime::value::RuntimeError::TypeError {
                msg: "Expected number".to_string(),
                span: atlas_runtime::span::Span::dummy(),
            })
        }
    });

    let result = runtime.eval("double(21)").unwrap();
    assert_eq!(result.to_string(), "42");
}

#[test]
fn test_config_clone() {
    let config1 = RuntimeConfig::sandboxed();
    let config2 = config1.clone();

    assert_eq!(config1.allow_io, config2.allow_io);
    assert_eq!(config1.allow_network, config2.allow_network);
    assert_eq!(config1.max_execution_time, config2.max_execution_time);
    assert_eq!(config1.max_memory_bytes, config2.max_memory_bytes);
}

#[test]
fn test_multiple_sandboxed_runtimes_independent() {
    let mut runtime1 = Runtime::sandboxed();
    let mut runtime2 = Runtime::sandboxed();

    let result1 = runtime1.eval("let x: number = 10; x").unwrap();
    let result2 = runtime2.eval("let x: number = 20; x").unwrap();

    assert_eq!(result1.to_string(), "10");
    assert_eq!(result2.to_string(), "20");
}

#[test]
fn test_config_with_only_time_limit() {
    let config = RuntimeConfig::new().with_max_execution_time(Duration::from_secs(10));

    assert_eq!(config.max_execution_time, Some(Duration::from_secs(10)));
    assert!(config.max_memory_bytes.is_none());
    assert!(config.allow_io);
    assert!(config.allow_network);
}

#[test]
fn test_config_with_only_memory_limit() {
    let config = RuntimeConfig::new().with_max_memory_bytes(50_000_000);

    assert!(config.max_execution_time.is_none());
    assert_eq!(config.max_memory_bytes, Some(50_000_000));
    assert!(config.allow_io);
    assert!(config.allow_network);
}

#[test]
fn test_config_disable_only_io() {
    let config = RuntimeConfig::new().with_io_allowed(false);

    assert!(!config.allow_io);
    assert!(config.allow_network);
}

#[test]
fn test_config_disable_only_network() {
    let config = RuntimeConfig::new().with_network_allowed(false);

    assert!(config.allow_io);
    assert!(!config.allow_network);
}

#[test]
fn test_sandboxed_runtime_error_handling() {
    let mut runtime = Runtime::sandboxed();

    // Type errors should still be caught
    let result = runtime.eval(r#"let x: number = "not a number";"#);
    assert!(result.is_err());
}

#[test]
fn test_sandboxed_runtime_persistent_state() {
    let mut runtime = Runtime::sandboxed();

    // Define a function in one eval()
    runtime
        .eval("fn increment(borrow x: number): number { return x + 1; }")
        .unwrap();

    // Call it in subsequent eval() calls
    let result1 = runtime.eval("increment(5)").unwrap();
    let result2 = runtime.eval("increment(10)").unwrap();

    assert_eq!(result1.to_string(), "6");
    assert_eq!(result2.to_string(), "11");
}

// --- Timeout Enforcement Tests (H-001) ---

#[test]
fn test_timeout_enforcement_interpreter() {
    // Create config with very short timeout (100ms)
    let config = RuntimeConfig::new()
        .with_max_execution_time(Duration::from_millis(100))
        .with_io_allowed(false);

    let mut runtime = Runtime::from_config(config);

    // Run an infinite loop - should timeout
    let result = runtime.eval(
        r#"
        let mut x: number = 0;
        while (true) {
            x = x + 1;
        }
        "#,
    );

    // Should fail with timeout error
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = format!("{:?}", err);
    assert!(
        err_msg.contains("Timeout") || err_msg.contains("timeout"),
        "Expected timeout error, got: {}",
        err_msg
    );
}

#[test]
fn test_timeout_enforcement_vm() {
    // Create config with very short timeout (100ms)
    let config = RuntimeConfig::new()
        .with_max_execution_time(Duration::from_millis(100))
        .with_io_allowed(false);

    let mut runtime = Runtime::from_config(config);

    // Run an infinite loop - should timeout
    let result = runtime.eval(
        r#"
        let mut x: number = 0;
        while (true) {
            x = x + 1;
        }
        "#,
    );

    // Should fail with timeout error
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = format!("{:?}", err);
    assert!(
        err_msg.contains("Timeout") || err_msg.contains("timeout"),
        "Expected timeout error, got: {}",
        err_msg
    );
}

#[test]
fn test_timeout_respects_limit() {
    // Create config with 500ms timeout
    let config = RuntimeConfig::new()
        .with_max_execution_time(Duration::from_millis(500))
        .with_io_allowed(false);

    let mut runtime = Runtime::from_config(config);

    // Run a fast operation - should complete before timeout
    let result = runtime.eval("1 + 2 + 3");

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_string(), "6");
}

// --- Memory Limit Enforcement Tests (H-001) ---

#[test]
fn test_memory_limit_enforcement_interpreter_array() {
    // Create config with very small memory limit (1KB)
    let config = RuntimeConfig::new()
        .with_max_memory_bytes(1024)
        .with_io_allowed(false);

    let mut runtime = Runtime::from_config(config);

    // Try to create a large array - should fail with memory limit
    // Each element is ~64 bytes, so 40 elements = ~2560 bytes + overhead > 1024
    let result = runtime.eval(
        r#"
        let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
                   11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
                   21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
                   31, 32, 33, 34, 35, 36, 37, 38, 39, 40];
        arr
        "#,
    );

    // Should fail with memory limit error
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = format!("{:?}", err);
    assert!(
        err_msg.contains("Memory") || err_msg.contains("memory"),
        "Expected memory limit error, got: {}",
        err_msg
    );
}

#[test]
fn test_memory_limit_enforcement_vm_array() {
    // Create config with very small memory limit (1KB)
    let config = RuntimeConfig::new()
        .with_max_memory_bytes(1024)
        .with_io_allowed(false);

    let mut runtime = Runtime::from_config(config);

    // Try to create a large array - should fail with memory limit
    // Each element is ~64 bytes, so 40 elements = ~2560 bytes + overhead > 1024
    let result = runtime.eval(
        r#"
        let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
                   11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
                   21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
                   31, 32, 33, 34, 35, 36, 37, 38, 39, 40];
        arr
        "#,
    );

    // Should fail with memory limit error
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = format!("{:?}", err);
    assert!(
        err_msg.contains("Memory") || err_msg.contains("memory"),
        "Expected memory limit error, got: {}",
        err_msg
    );
}

#[test]
fn test_no_memory_limit_without_config() {
    // Create config without memory limit
    let config = RuntimeConfig::new().with_io_allowed(false);

    let mut runtime = Runtime::from_config(config);

    // Create a reasonably sized array - should complete normally
    let result = runtime.eval(
        r#"
        let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        len(arr)
        "#,
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_string(), "10");
}

#[test]
fn test_memory_limit_allows_small_allocations() {
    // Create config with generous memory limit (10MB)
    let config = RuntimeConfig::new()
        .with_max_memory_bytes(10_000_000)
        .with_io_allowed(false);

    let mut runtime = Runtime::from_config(config);

    // Create small arrays and strings - should work fine
    let result = runtime.eval(
        r#"
        let arr = [1, 2, 3, 4, 5];
        let s = "hello" + " world";
        len(arr)
        "#,
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_string(), "5");
}

// --- Security Split Tests (H-001 fix verification) ---

#[test]
fn test_allow_io_true_network_false_blocks_http() {
    // CRITICAL: This tests the security split fix
    // allow_io=true should NOT grant network access when allow_network=false
    let config = RuntimeConfig::new()
        .with_io_allowed(true)
        .with_network_allowed(false);

    let mut runtime = Runtime::from_config(config);

    // HTTP request should be blocked even though IO is allowed
    let result = runtime.eval(r#"http.get("https://example.com")"#);

    // Should fail with security/permission error
    assert!(
        result.is_err(),
        "HTTP request should be blocked when allow_network=false"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("ermission") || err_msg.contains("denied") || err_msg.contains("ecurity"),
        "Expected permission/security error, got: {}",
        err_msg
    );
}

#[test]
fn test_allow_io_true_network_false_blocks_http_vm() {
    // Same test but for VM mode
    let config = RuntimeConfig::new()
        .with_io_allowed(true)
        .with_network_allowed(false);

    let mut runtime = Runtime::from_config(config);

    // HTTP request should be blocked even though IO is allowed
    let result = runtime.eval(r#"http.get("https://example.com")"#);

    // Should fail with security/permission error
    assert!(
        result.is_err(),
        "HTTP request should be blocked when allow_network=false"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("ermission") || err_msg.contains("denied") || err_msg.contains("ecurity"),
        "Expected permission/security error, got: {}",
        err_msg
    );
}

#[test]
fn test_allow_io_false_network_true_blocks_filesystem() {
    // allow_network=true should NOT grant filesystem access when allow_io=false.
    // Write a real temp file so canonicalize() succeeds, then verify file.read()
    // returns an Atlas Err (permission denied), not an Ok.
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
    writeln!(tmp, "secret").unwrap();
    let path = tmp.path().to_string_lossy().to_string();

    let config = RuntimeConfig::new()
        .with_io_allowed(false)
        .with_network_allowed(true);
    let mut runtime = Runtime::from_config(config);

    // file.read() returns Result<string, string> — blocked → Atlas Err("permission denied ...")
    let result = runtime
        .eval(&format!(
            r#"
let r = file.read("{path}");
match r {{
    Ok(_) => "allowed",
    Err(e) => e,
}}
"#
        ))
        .expect("eval should succeed");

    let msg = format!("{}", result);
    assert!(
        msg.contains("permission") || msg.contains("denied"),
        "Expected permission denied, got: {}",
        msg
    );
}

#[test]
fn test_both_permissions_independent() {
    // Test that both can be enabled independently
    let config = RuntimeConfig::new()
        .with_io_allowed(true)
        .with_network_allowed(true);

    let mut runtime = Runtime::from_config(config);

    // Both should work (we're just checking it doesn't error on setup)
    // Actual network calls might fail for other reasons (no network), but not permission
    let result = runtime.eval("1 + 1");
    assert!(result.is_ok());
}

#[test]
fn test_neither_permission_blocks_both() {
    // Neither permission = both blocked
    let config = RuntimeConfig::new()
        .with_io_allowed(false)
        .with_network_allowed(false);

    let mut runtime = Runtime::from_config(config);

    // Filesystem should be blocked
    let fs_result = runtime.eval(r#"read_file("/etc/passwd")"#);
    assert!(fs_result.is_err(), "Filesystem should be blocked");

    // HTTP should also be blocked
    let http_result = runtime.eval(r#"http.get("https://example.com")"#);
    assert!(http_result.is_err(), "HTTP should be blocked");
}
