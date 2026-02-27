use super::*;

#[test]
fn test_own_param_consumes_binding_debug() {
    let src = r#"
        fn consume(own data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        consume(arr);
        arr;
    "#;
    let result = run_interpreter(src);
    assert!(
        result.is_err(),
        "Expected error after consuming arr, got: {:?}",
        result
    );
    assert!(
        result.unwrap_err().contains("use of moved value"),
        "Error should mention 'use of moved value'"
    );
}

/// A `borrow` parameter must NOT consume the caller's binding.
#[test]
fn test_borrow_param_does_not_consume_binding() {
    let src = r#"
        fn read(borrow data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        read(arr);
        len(arr);
    "#;
    let result = run_interpreter(src);
    assert!(
        result.is_ok(),
        "borrow should not consume binding, got: {:?}",
        result
    );
    std::assert_eq!(result.unwrap(), "Number(3)");
}

/// An unannotated parameter must NOT consume the caller's binding.
#[test]
fn test_unannotated_param_does_not_consume_binding() {
    let src = r#"
        fn take(data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        take(arr);
        len(arr);
    "#;
    let result = run_interpreter(src);
    assert!(
        result.is_ok(),
        "unannotated param should not consume binding, got: {:?}",
        result
    );
    std::assert_eq!(result.unwrap(), "Number(3)");
}

/// Passing a literal to an `own` parameter must not attempt to consume any binding.
#[test]
fn test_own_param_with_literal_arg_no_consume() {
    let src = r#"
        fn consume(own data: array<number>) -> void { }
        consume([1, 2, 3]);
        42;
    "#;
    let result = run_interpreter(src);
    assert!(
        result.is_ok(),
        "literal arg to own param should not error, got: {:?}",
        result
    );
    std::assert_eq!(result.unwrap(), "Number(42)");
}

/// Passing an expression result to an `own` parameter must not consume any binding.
#[test]
fn test_own_param_with_expression_arg_no_consume() {
    let src = r#"
        fn make_arr() -> array<number> { [10, 20]; }
        fn consume(own data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        consume(make_arr());
        len(arr);
    "#;
    let result = run_interpreter(src);
    assert!(
        result.is_ok(),
        "expression arg to own param should not consume unrelated binding, got: {:?}",
        result
    );
    std::assert_eq!(result.unwrap(), "Number(3)");
}

// ============================================================================
// Phase 09: Runtime `shared` enforcement in interpreter (debug mode)
// ============================================================================

/// Passing a plain (non-shared) value to a `shared` param must produce a runtime error.
#[test]
fn test_shared_param_rejects_plain_value_debug() {
    let src = r#"
        fn register(shared handler: number[]) -> void { }
        let arr: number[] = [1, 2, 3];
        register(arr);
    "#;
    let result = run_interpreter(src);
    assert!(
        result.is_err(),
        "Expected ownership violation error, got: {:?}",
        result
    );
    assert!(
        result.unwrap_err().contains("ownership violation"),
        "Error should mention 'ownership violation'"
    );
}

/// Passing an actual SharedValue to a `shared` param must succeed.
#[test]
fn test_shared_param_accepts_shared_value() {
    use atlas_runtime::value::{Shared, Value};

    // Parse and register the function
    let src = r#"
        fn register(shared handler: number[]) -> void { }
        register(sv);
    "#;
    let mut lexer = atlas_runtime::lexer::Lexer::new(src);
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::parser::Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = atlas_runtime::binder::Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = atlas_runtime::typechecker::TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);

    let mut interp = Interpreter::new();
    // Inject a SharedValue into the interpreter's globals so Atlas source can reference it
    let shared_val = Value::SharedValue(Shared::new(Box::new(Value::array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
    ]))));
    interp.define_global("sv".to_string(), shared_val);

    let result = interp.eval(&program, &SecurityContext::allow_all());
    assert!(
        result.is_ok(),
        "SharedValue passed to shared param should succeed, got: {:?}",
        result
    );
}

/// Passing a SharedValue to an `own` param emits an advisory (not a hard error).
#[test]
fn test_shared_value_to_own_param_advisory_not_error() {
    use atlas_runtime::value::{Shared, Value};

    let src = r#"
        fn consume(own handler: number[]) -> void { }
        consume(sv);
    "#;
    let mut lexer = atlas_runtime::lexer::Lexer::new(src);
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::parser::Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = atlas_runtime::binder::Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = atlas_runtime::typechecker::TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);

    let mut interp = Interpreter::new();
    let shared_val = Value::SharedValue(Shared::new(Box::new(Value::array(vec![Value::Number(
        1.0,
    )]))));
    interp.define_global("sv".to_string(), shared_val);

    // Advisory warning only — must NOT be a hard error
    let result = interp.eval(&program, &SecurityContext::allow_all());
    assert!(
        result.is_ok(),
        "SharedValue to own param should be advisory (not hard error), got: {:?}",
        result
    );
}

// ============================================================================

// ─── Scenario 1: Unannotated function — no regression ────────────────────────

#[test]
fn test_parity_unannotated_function_no_regression() {
    assert_ownership_parity(
        r#"
        fn add(a: number, b: number) -> number { a + b; }
        add(3, 4);
        "#,
    );
}
