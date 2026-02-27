use super::*;

#[test]
fn test_runtime_captures_print_output() {
    use atlas_runtime::stdlib::OutputWriter;
    let buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let output: OutputWriter = Arc::new(Mutex::new(Box::new(VecWriter(buf.clone()))));
    let config = RuntimeConfig::new().with_output(output);
    let mut runtime = Runtime::with_config(ExecutionMode::Interpreter, config);
    runtime.eval(r#"print("captured")"#).unwrap();
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
    let mut runtime = Runtime::sandboxed(ExecutionMode::Interpreter);

    // Basic expressions should still work
    let result = runtime.eval("1 + 2").unwrap();
    assert_eq!(result.to_string(), "3");
}

#[test]
fn test_runtime_with_custom_config() {
    let config = RuntimeConfig::new()
        .with_io_allowed(false)
        .with_network_allowed(false);

    let mut runtime = Runtime::with_config(ExecutionMode::VM, config);

    // Basic expressions should work
    let result = runtime.eval("let x: number = 42; x").unwrap();
    assert_eq!(result.to_string(), "42");
}

#[test]
fn test_sandboxed_runtime_basic_arithmetic() {
    let mut runtime = Runtime::sandboxed(ExecutionMode::Interpreter);
    let result = runtime.eval("10 * 5 + 3").unwrap();
    assert_eq!(result.to_string(), "53");
}

#[test]
fn test_sandboxed_runtime_function_definitions() {
    let mut runtime = Runtime::sandboxed(ExecutionMode::VM);
    runtime
        .eval("fn add(a: number, b: number) -> number { return a + b; }")
        .unwrap();

    let result = runtime.eval("add(10, 20)").unwrap();
    assert_eq!(result.to_string(), "30");
}

#[test]
fn test_sandboxed_runtime_string_operations() {
    let mut runtime = Runtime::sandboxed(ExecutionMode::Interpreter);
    let result = runtime.eval(r#""Hello, " + "World!""#).unwrap();
    assert_eq!(result.to_string(), "Hello, World!");
}

#[test]
fn test_sandboxed_runtime_array_operations() {
    let mut runtime = Runtime::sandboxed(ExecutionMode::VM);
    let result = runtime.eval("len([1, 2, 3])").unwrap();
    assert_eq!(result.to_string(), "3");
}

#[test]
fn test_sandboxed_runtime_loops() {
    let mut runtime = Runtime::sandboxed(ExecutionMode::Interpreter);
    let result = runtime
        .eval(
            r#"
        var sum: number = 0;
        for (var i: number = 0; i < 10; i = i + 1) {
            sum = sum + i;
        }
        sum
        "#,
        )
        .unwrap();

    assert_eq!(result.to_string(), "45");
}

#[test]
fn test_sandboxed_runtime_conditionals() {
    let mut runtime = Runtime::sandboxed(ExecutionMode::VM);
    runtime
        .eval(
            r#"
        fn maximum(a: number, b: number) -> number {
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
    let mut runtime = Runtime::sandboxed(ExecutionMode::Interpreter);

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
    let mut runtime1 = Runtime::sandboxed(ExecutionMode::Interpreter);
    let mut runtime2 = Runtime::sandboxed(ExecutionMode::Interpreter);

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
    let mut runtime = Runtime::sandboxed(ExecutionMode::Interpreter);

    // Type errors should still be caught
    let result = runtime.eval(r#"let x: number = "not a number";"#);
    assert!(result.is_err());
}

#[test]
fn test_sandboxed_runtime_persistent_state() {
    let mut runtime = Runtime::sandboxed(ExecutionMode::VM);

    // Define a function in one eval()
    runtime
        .eval("fn increment(x: number) -> number { return x + 1; }")
        .unwrap();

    // Call it in subsequent eval() calls
    let result1 = runtime.eval("increment(5)").unwrap();
    let result2 = runtime.eval("increment(10)").unwrap();

    assert_eq!(result1.to_string(), "6");
    assert_eq!(result2.to_string(), "11");
}

