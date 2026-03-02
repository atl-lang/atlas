use super::*;
#[allow(unused_imports)]
use pretty_assertions::assert_eq;

// VM parity tests for ? operator (Block 6 Phase 03)
//
// These tests verify the ? operator works identically through the VM pipeline
// (Lexer → Parser → Binder → TypeChecker → Compiler → VM).
// Uses compile_checked() to ensure TryTargetKind annotations are set.

// ============================================================================
// Result ? Operator — VM Parity
// ============================================================================

#[test]
fn test_vm_result_try_unwraps_ok() {
    let code = r#"
    fn get_value() -> Result<number, string> {
        let result = Ok(42);
        return Ok(result?);
    }
    unwrap(get_value());
"#;
    match vm_eval_checked(code) {
        Some(Value::Number(n)) => assert_eq!(n, 42.0),
        other => panic!("Expected Number(42), got {:?}", other),
    }
}

#[test]
fn test_vm_result_try_propagates_err() {
    let code = r#"
    fn get_value() -> Result<number, string> {
        let result = Err("failed");
        return Ok(result?);
    }
    is_err(get_value());
"#;
    match vm_eval_checked(code) {
        Some(Value::Bool(b)) => assert!(b, "Expected true (is_err)"),
        other => panic!("Expected Bool(true), got {:?}", other),
    }
}

#[test]
fn test_vm_result_try_multiple() {
    let code = r#"
    fn divide(a: number, b: number) -> Result<number, string> {
        if (b == 0) {
            return Err("division by zero");
        }
        return Ok(a / b);
    }
    fn calculate() -> Result<number, string> {
        let x = divide(100, 10)?;
        let y = divide(x, 2)?;
        let z = divide(y, 5)?;
        return Ok(z);
    }
    unwrap(calculate());
"#;
    match vm_eval_checked(code) {
        Some(Value::Number(n)) => assert_eq!(n, 1.0),
        other => panic!("Expected Number(1), got {:?}", other),
    }
}

#[test]
fn test_vm_result_try_early_return() {
    let code = r#"
    fn divide(a: number, b: number) -> Result<number, string> {
        if (b == 0) {
            return Err("division by zero");
        }
        return Ok(a / b);
    }
    fn calculate() -> Result<number, string> {
        let x = divide(100, 10)?;
        let y = divide(x, 0)?;
        let z = divide(y, 5)?;
        return Ok(z);
    }
    is_err(calculate());
"#;
    match vm_eval_checked(code) {
        Some(Value::Bool(b)) => assert!(b, "Expected true (is_err)"),
        other => panic!("Expected Bool(true), got {:?}", other),
    }
}

#[test]
fn test_vm_result_try_in_expression() {
    let code = r#"
    fn get_number() -> Result<number, string> {
        return Ok(21);
    }
    fn double_it() -> Result<number, string> {
        return Ok(get_number()? * 2);
    }
    unwrap(double_it());
"#;
    match vm_eval_checked(code) {
        Some(Value::Number(n)) => assert_eq!(n, 42.0),
        other => panic!("Expected Number(42), got {:?}", other),
    }
}

#[test]
fn test_vm_result_try_nested_calls() {
    let code = r#"
    fn inner() -> Result<number, string> { return Ok(42); }
    fn middle() -> Result<number, string> { return Ok(inner()?); }
    fn outer() -> Result<number, string> { return Ok(middle()?); }
    unwrap(outer());
"#;
    match vm_eval_checked(code) {
        Some(Value::Number(n)) => assert_eq!(n, 42.0),
        other => panic!("Expected Number(42), got {:?}", other),
    }
}

#[test]
fn test_vm_result_try_nested_error_propagation() {
    let code = r#"
    fn inner() -> Result<number, string> { return Err("inner failed"); }
    fn middle() -> Result<number, string> { return Ok(inner()?); }
    fn outer() -> Result<number, string> { return Ok(middle()?); }
    is_err(outer());
"#;
    match vm_eval_checked(code) {
        Some(Value::Bool(b)) => assert!(b, "Expected true (is_err)"),
        other => panic!("Expected Bool(true), got {:?}", other),
    }
}

// ============================================================================
// Result ? — VM/Interpreter Parity
// ============================================================================

#[test]
fn test_parity_result_try_ok() {
    let code = r#"
    fn get() -> Result<number, string> {
        let r = Ok(42);
        return Ok(r?);
    }
    unwrap(get());
"#;
    // Run through both engines and compare
    let vm_result = vm_eval_checked(code).unwrap_or(Value::Null);
    let interp_result = interp_eval(code);
    assert_eq!(vm_result, interp_result, "Parity mismatch for Result ? Ok");
}

#[test]
fn test_parity_result_try_err() {
    let code = r#"
    fn get() -> Result<number, string> {
        let r = Err("fail");
        return Ok(r?);
    }
    is_err(get());
"#;
    let vm_result = vm_eval_checked(code).unwrap_or(Value::Null);
    let interp_result = interp_eval(code);
    assert_eq!(
        vm_result, interp_result,
        "Parity mismatch for Result ? Err"
    );
}

// ============================================================================
// Option ? Operator — VM Parity
// ============================================================================

#[test]
fn test_vm_option_try_unwraps_some() {
    let code = r#"
    fn find() -> Option<number> {
        let opt = Some(42);
        return Some(opt?);
    }
    unwrap(find());
"#;
    match vm_eval_checked(code) {
        Some(Value::Number(n)) => assert_eq!(n, 42.0),
        other => panic!("Expected Number(42), got {:?}", other),
    }
}

#[test]
fn test_vm_option_try_propagates_none() {
    let code = r#"
    fn find() -> Option<number> {
        let opt: Option<number> = None();
        return Some(opt?);
    }
    is_none(find());
"#;
    match vm_eval_checked(code) {
        Some(Value::Bool(b)) => assert!(b, "Expected true (is_none)"),
        other => panic!("Expected Bool(true), got {:?}", other),
    }
}

#[test]
fn test_vm_option_try_multiple() {
    let code = r#"
    fn a() -> Option<number> { return Some(10); }
    fn b() -> Option<number> { return Some(20); }
    fn calc() -> Option<number> {
        let x = a()?;
        let y = b()?;
        return Some(x + y);
    }
    unwrap(calc());
"#;
    match vm_eval_checked(code) {
        Some(Value::Number(n)) => assert_eq!(n, 30.0),
        other => panic!("Expected Number(30), got {:?}", other),
    }
}

#[test]
fn test_vm_option_try_early_none() {
    let code = r#"
    fn a() -> Option<number> { return Some(10); }
    fn b() -> Option<number> { return None(); }
    fn calc() -> Option<number> {
        let x = a()?;
        let y = b()?;
        return Some(x + y);
    }
    is_none(calc());
"#;
    match vm_eval_checked(code) {
        Some(Value::Bool(b)) => assert!(b, "Expected true (is_none)"),
        other => panic!("Expected Bool(true), got {:?}", other),
    }
}

#[test]
fn test_vm_option_try_nested() {
    let code = r#"
    fn inner() -> Option<number> { return Some(42); }
    fn middle() -> Option<number> { return Some(inner()?); }
    fn outer() -> Option<number> { return Some(middle()?); }
    unwrap(outer());
"#;
    match vm_eval_checked(code) {
        Some(Value::Number(n)) => assert_eq!(n, 42.0),
        other => panic!("Expected Number(42), got {:?}", other),
    }
}

// ============================================================================
// Option ? — VM/Interpreter Parity
// ============================================================================

#[test]
fn test_parity_option_try_some() {
    let code = r#"
    fn find() -> Option<number> {
        let opt = Some(42);
        return Some(opt?);
    }
    unwrap(find());
"#;
    let vm_result = vm_eval_checked(code).unwrap_or(Value::Null);
    let interp_result = interp_eval(code);
    assert_eq!(
        vm_result, interp_result,
        "Parity mismatch for Option ? Some"
    );
}

#[test]
fn test_parity_option_try_none() {
    let code = r#"
    fn find() -> Option<number> {
        let opt: Option<number> = None();
        return Some(opt?);
    }
    is_none(find());
"#;
    let vm_result = vm_eval_checked(code).unwrap_or(Value::Null);
    let interp_result = interp_eval(code);
    assert_eq!(
        vm_result, interp_result,
        "Parity mismatch for Option ? None"
    );
}

// ============================================================================
// Integration Tests — Cross-Feature (Phase 05)
// ============================================================================

#[test]
fn test_vm_try_multiple_in_expression() {
    let code = r#"
    fn a() -> Result<number, string> { return Ok(10); }
    fn b() -> Result<number, string> { return Ok(32); }
    fn calc() -> Result<number, string> {
        return Ok(a()? + b()?);
    }
    unwrap(calc());
"#;
    match vm_eval_checked(code) {
        Some(Value::Number(n)) => assert_eq!(n, 42.0),
        other => panic!("Expected Number(42), got {:?}", other),
    }
}

#[test]
fn test_vm_try_chained_transforms() {
    let code = r#"
    fn parse_num(s: string) -> Result<number, string> {
        if (s == "42") { return Ok(42); }
        return Err("not 42");
    }
    fn double(n: number) -> Result<number, string> {
        return Ok(n * 2);
    }
    fn process(s: string) -> Result<number, string> {
        let n = parse_num(s)?;
        let d = double(n)?;
        return Ok(d);
    }
    unwrap(process("42"));
"#;
    match vm_eval_checked(code) {
        Some(Value::Number(n)) => assert_eq!(n, 84.0),
        other => panic!("Expected Number(84), got {:?}", other),
    }
}

#[test]
fn test_parity_try_multiple_expr() {
    let code = r#"
    fn a() -> Result<number, string> { return Ok(10); }
    fn b() -> Result<number, string> { return Ok(32); }
    fn calc() -> Result<number, string> {
        return Ok(a()? + b()?);
    }
    unwrap(calc());
"#;
    let vm_result = vm_eval_checked(code).unwrap_or(Value::Null);
    let interp_result = interp_eval(code);
    assert_eq!(vm_result, interp_result, "Parity mismatch for multiple ? in expression");
}

#[test]
fn test_parity_try_first_fails() {
    let code = r#"
    fn a() -> Result<number, string> { return Err("a failed"); }
    fn b() -> Result<number, string> { return Ok(32); }
    fn calc() -> Result<number, string> {
        return Ok(a()? + b()?);
    }
    is_err(calc());
"#;
    let vm_result = vm_eval_checked(code).unwrap_or(Value::Null);
    let interp_result = interp_eval(code);
    assert_eq!(vm_result, interp_result, "Parity mismatch for first ? fails");
}

#[test]
fn test_parity_option_try_multiple_expr() {
    let code = r#"
    fn a() -> Option<number> { return Some(10); }
    fn b() -> Option<number> { return Some(32); }
    fn calc() -> Option<number> {
        return Some(a()? + b()?);
    }
    unwrap(calc());
"#;
    let vm_result = vm_eval_checked(code).unwrap_or(Value::Null);
    let interp_result = interp_eval(code);
    assert_eq!(vm_result, interp_result, "Parity mismatch for Option multiple ?");
}

#[test]
fn test_vm_try_in_if_condition() {
    let code = r#"
    fn check() -> Result<bool, string> { return Ok(true); }
    fn run() -> Result<number, string> {
        if (check()?) {
            return Ok(42);
        }
        return Ok(0);
    }
    unwrap(run());
"#;
    match vm_eval_checked(code) {
        Some(Value::Number(n)) => assert_eq!(n, 42.0),
        other => panic!("Expected Number(42), got {:?}", other),
    }
}
