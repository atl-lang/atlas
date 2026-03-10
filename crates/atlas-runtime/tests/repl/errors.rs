use atlas_runtime::repl::ReplCore;

use super::helpers::{eval_err, eval_ok};

// ============================================================================
// Error Recovery
// ============================================================================

#[test]
fn test_errors_do_not_reset_state() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let x = 100;");
    eval_err(&mut repl, "let y: number = \"bad\";"); // Type error
    assert_eq!(
        eval_ok(&mut repl, "x;"),
        atlas_runtime::Value::Number(100.0)
    );
}

#[test]
fn test_parse_error_does_not_reset_state() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let x = 50;");
    eval_err(&mut repl, "let = ;"); // Parse error
    assert_eq!(eval_ok(&mut repl, "x;"), atlas_runtime::Value::Number(50.0));
}

#[test]
fn test_runtime_error_does_not_reset_state() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let x = 25;");
    eval_err(&mut repl, "let y = x / 0;"); // Runtime error
    assert_eq!(eval_ok(&mut repl, "x;"), atlas_runtime::Value::Number(25.0));
}

// ============================================================================
// Redefinition Rules
// ============================================================================

#[test]
fn test_cannot_redeclare_variable() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let x = 1;");
    eval_err(&mut repl, "let x = 2;"); // Should error: already declared
}

#[test]
fn test_cannot_redeclare_function() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "fn foo(): number { return 1; }");
    eval_err(&mut repl, "fn foo(): number { return 2; }"); // Should error
}
