use super::*;

#[test]
fn test_sandboxed_runtime_loops() {
    let mut runtime = Runtime::sandboxed();
    let result = runtime
        .eval(
            r#"
        let mut sum: number = 0;
        for i in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] {
            sum = sum + i;
        }
        sum
        "#,
        )
        .unwrap();

    assert_eq!(result.to_string(), "45");
}

#[test]
fn test_no_timeout_without_limit() {
    // Create config without timeout
    let config = RuntimeConfig::new().with_io_allowed(false);

    let mut runtime = Runtime::from_config(config);

    // Run a finite loop - should complete normally
    let result = runtime.eval(
        r#"
        let mut sum: number = 0;
        for h in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] {
            for t in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] {
                for o in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] {
                    sum = sum + (h * 100 + t * 10 + o);
                }
            }
        }
        sum
        "#,
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_string(), "499500");
}

#[test]
fn test_memory_limit_enforcement_interpreter_string_concat() {
    // Create config with very small memory limit (500 bytes)
    let config = RuntimeConfig::new()
        .with_max_memory_bytes(500)
        .with_io_allowed(false);

    let mut runtime = Runtime::from_config(config);

    // Try to create a large string through concatenation - should fail
    let result = runtime.eval(
        r#"
        let mut s: string = "a";
        for i in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19] {
            s = s + s;
            let _unused = i;
        }
        s
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
fn test_memory_limit_enforcement_vm_string_concat() {
    // Create config with very small memory limit (500 bytes)
    let config = RuntimeConfig::new()
        .with_max_memory_bytes(500)
        .with_io_allowed(false);

    let mut runtime = Runtime::from_config(config);

    // Try to create a large string through concatenation - should fail
    let result = runtime.eval(
        r#"
        let mut s: string = "a";
        for i in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19] {
            s = s + s;
            let _unused = i;
        }
        s
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
