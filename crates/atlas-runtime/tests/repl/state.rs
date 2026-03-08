use atlas_runtime::repl::ReplCore;
use atlas_runtime::Value;

use super::helpers::{assert_value, eval_ok};

// ============================================================================
// Variable Persistence
// ============================================================================

#[test]
fn test_variable_persistence() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let x = 42;");
    assert_value(&mut repl, "x;", Value::Number(42.0));
    assert_value(&mut repl, "x + 8;", Value::Number(50.0));
}

#[test]
fn test_mutable_variable_reassignment() {
    let mut repl = ReplCore::new();
    // Use `let mut` (recommended) instead of deprecated `var`
    eval_ok(&mut repl, "let mut count = 0;");
    eval_ok(&mut repl, "count = count + 1;");
    assert_value(&mut repl, "count;", Value::Number(1.0));
    eval_ok(&mut repl, "count = count + 10;");
    assert_value(&mut repl, "count;", Value::Number(11.0));
}

#[test]
fn test_multiple_variables_persist() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let a = 1;");
    eval_ok(&mut repl, "let b = 2;");
    eval_ok(&mut repl, "let c = 3;");
    assert_value(&mut repl, "a + b + c;", Value::Number(6.0));
}

// ============================================================================
// Function Persistence
// ============================================================================

#[test]
fn test_function_persistence() {
    let mut repl = ReplCore::new();
    eval_ok(
        &mut repl,
        "fn double(borrow x: number) -> number { return x * 2; }",
    );
    assert_value(&mut repl, "double(21);", Value::Number(42.0));
}

#[test]
fn test_multiple_functions_persist() {
    let mut repl = ReplCore::new();
    eval_ok(
        &mut repl,
        "fn add(borrow a: number, borrow b: number) -> number { return a + b; }",
    );
    eval_ok(
        &mut repl,
        "fn mul(borrow a: number, borrow b: number) -> number { return a * b; }",
    );
    assert_value(&mut repl, "add(5, 3);", Value::Number(8.0));
    assert_value(&mut repl, "mul(6, 7);", Value::Number(42.0));
}

#[test]
fn test_functions_call_other_functions() {
    let mut repl = ReplCore::new();
    eval_ok(
        &mut repl,
        "fn square(borrow x: number) -> number { return x * x; }",
    );
    eval_ok(
        &mut repl,
        "fn sum_of_squares(borrow a: number, borrow b: number) -> number { return square(a) + square(b); }",
    );
    assert_value(&mut repl, "sum_of_squares(3, 4);", Value::Number(25.0));
}

// ============================================================================
// Mixed Variables and Functions
// ============================================================================

#[test]
fn test_functions_use_global_variables() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let multiplier = 10;");
    eval_ok(
        &mut repl,
        "fn scale(borrow x: number) -> number { return x * multiplier; }",
    );
    assert_value(&mut repl, "scale(5);", Value::Number(50.0));
}

#[test]
fn test_variables_use_functions() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "fn getValue() -> number { return 42; }");
    eval_ok(&mut repl, "let result = getValue();");
    assert_value(&mut repl, "result;", Value::Number(42.0));
}

// ============================================================================
// Complex State Interactions
// ============================================================================

#[test]
fn test_nested_function_calls_with_state() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let base = 10;");
    eval_ok(
        &mut repl,
        "fn add_base(borrow x: number) -> number { return x + base; }",
    );
    eval_ok(
        &mut repl,
        "fn double_and_add(borrow x: number) -> number { return add_base(x * 2); }",
    );
    assert_value(&mut repl, "double_and_add(5);", Value::Number(20.0));
}

#[test]
fn test_repl_can_use_arrays() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let arr = [1, 2, 3];");
    assert_value(&mut repl, "arr[1];", Value::Number(2.0));
    eval_ok(&mut repl, "arr[1] = 42;");
    assert_value(&mut repl, "arr[1];", Value::Number(42.0));
}

#[test]
fn test_multiple_statements_in_one_input() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let x = 1; let y = 2;");
    assert_value(&mut repl, "x + y;", Value::Number(3.0));
}

#[test]
fn test_recursion_across_repl_inputs() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "fn factorial(borrow n: number) -> number { if (n <= 1) { return 1; } return n * factorial(n - 1); }");
    assert_value(&mut repl, "factorial(5);", Value::Number(120.0));
}

// ============================================================================
// Additional REPL State Tests
// ============================================================================

#[test]
fn repl_type_of_function_call() {
    let mut repl = ReplCore::new();
    repl.eval_line("fn double(borrow x: number) -> number { return x * 2; }");
    let result = repl.type_of_expression("double(5);");
    assert!(result.diagnostics.is_empty());
    assert_eq!(result.ty.unwrap().display_name(), "number");
}

#[test]
fn repl_variables_after_multiple_declarations() {
    let mut repl = ReplCore::new();
    repl.eval_line("let a = 1;");
    repl.eval_line("let b = 2;");
    repl.eval_line("let c = 3;");
    let vars = repl.variables();
    assert_eq!(vars.len(), 3);
}

#[test]
fn repl_variables_sorted_alphabetically() {
    let mut repl = ReplCore::new();
    repl.eval_line("let zz = 1;");
    repl.eval_line("let aa = 2;");
    repl.eval_line("let mm = 3;");
    let vars = repl.variables();
    let names: Vec<_> = vars.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(names, vec!["aa", "mm", "zz"]);
}

#[test]
fn repl_reset_clears_variables() {
    let mut repl = ReplCore::new();
    repl.eval_line("let x = 42;");
    assert!(!repl.variables().is_empty());
    repl.reset();
    assert!(repl.variables().is_empty());
}

#[test]
fn repl_eval_after_reset() {
    let mut repl = ReplCore::new();
    repl.eval_line("let x = 42;");
    repl.reset();
    let result = repl.eval_line("let y = 10;");
    assert!(result.diagnostics.is_empty());
}
