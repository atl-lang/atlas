use super::*;

#[test]
fn test_get_global_nonexistent_interpreter() {
    let runtime = Runtime::new(ExecutionMode::Interpreter);
    let value = runtime.get_global("nonexistent");
    assert!(value.is_none());
}

#[test]
fn test_set_global_and_get_global_roundtrip() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    runtime.set_global("x", Value::Number(100.0));
    let value = runtime.get_global("x").unwrap();
    assert!(matches!(value, Value::Number(n) if n == 100.0));
    // Note: Using set_global'd variables in eval() requires symbol table persistence (future phase)
}

#[test]
fn test_set_global_overwrite_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    runtime.set_global("x", Value::Number(10.0));
    runtime.set_global("x", Value::Number(20.0));
    let value = runtime.get_global("x").unwrap();
    assert!(matches!(value, Value::Number(n) if n == 20.0));
}

// get_global/set_global Tests - VM Mode (current limitations)

#[test]
fn test_get_global_vm_returns_none() {
    let runtime = Runtime::new(ExecutionMode::VM);
    // VM mode doesn't support direct global access yet
    let value = runtime.get_global("x");
    assert!(value.is_none());
}

// Mode Parity Tests

#[test]
fn test_parity_arithmetic_expression() {
    let mut interp = Runtime::new(ExecutionMode::Interpreter);
    let mut vm = Runtime::new(ExecutionMode::VM);

    let expr = "((10 + 5) * 2) - 3";
    let interp_result = interp.eval(expr).unwrap();
    let vm_result = vm.eval(expr).unwrap();

    assert!(matches!(interp_result, Value::Number(n) if n == 27.0));
    assert!(matches!(vm_result, Value::Number(n) if n == 27.0));
}

#[test]
fn test_parity_string_operations() {
    let mut interp = Runtime::new(ExecutionMode::Interpreter);
    let mut vm = Runtime::new(ExecutionMode::VM);

    let expr = "\"hello\" + \" \" + \"world\"";
    let interp_result = interp.eval(expr).unwrap();
    let vm_result = vm.eval(expr).unwrap();

    assert!(matches!(interp_result, Value::String(s) if s.as_ref() == "hello world"));
    assert!(matches!(vm_result, Value::String(s) if s.as_ref() == "hello world"));
}

#[test]
fn test_parity_boolean_logic() {
    let mut interp = Runtime::new(ExecutionMode::Interpreter);
    let mut vm = Runtime::new(ExecutionMode::VM);

    let expr = "(true && false) || (false || true)";
    let interp_result = interp.eval(expr).unwrap();
    let vm_result = vm.eval(expr).unwrap();

    assert!(matches!(interp_result, Value::Bool(true)));
    assert!(matches!(vm_result, Value::Bool(true)));
}

// Complex Program Tests

#[test]
fn test_complex_program_with_control_flow_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let program = r#"
        fn factorial(n: number) -> number {
            if (n <= 1) {
                return 1;
            } else {
                return n * factorial(n - 1);
            }
        }
        factorial(5)
    "#;
    let result = runtime.eval(program).unwrap();
    assert!(matches!(result, Value::Number(n) if n == 120.0));
}

#[test]
fn test_complex_program_with_loops_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let program = r#"
        var sum: number = 0;
        for (var i: number = 1; i <= 10; i = i + 1) {
            sum = sum + i;
        }
        sum
    "#;
    let result = runtime.eval(program).unwrap();
    assert!(matches!(result, Value::Number(n) if n == 55.0));
}

#[test]
fn test_multiple_function_definitions_single_eval() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    // Define all functions in a single eval()
    let result = runtime
        .eval(
            r#"
            fn add(x: number, y: number) -> number { return x + y; }
            fn sub(x: number, y: number) -> number { return x - y; }
            fn mul(x: number, y: number) -> number { return x * y; }
            add(10, sub(20, mul(2, 3)))
        "#,
        )
        .unwrap();
    assert!(matches!(result, Value::Number(n) if n == 24.0));
}
