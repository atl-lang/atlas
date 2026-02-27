use super::super::*;
#[allow(unused_imports)]
use pretty_assertions::assert_eq;

// From function_return_analysis_tests.rs
// ============================================================================

// Comprehensive tests for function return analysis
//
// Tests cover:
// - All code paths must return for non-void/non-null functions
// - If/else branch return analysis
// - Nested control flow return analysis
// - Early returns
// - Functions that don't need to return (void/null)
// - Missing return diagnostics (AT3004)

// ========== Functions That Always Return ==========

#[test]
fn test_simple_return() {
    let diagnostics = typecheck_source(
        r#"
        fn getNumber() -> number {
            return 42;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_expression() {
    let diagnostics = typecheck_source(
        r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_after_statements() {
    let diagnostics = typecheck_source(
        r#"
        fn calculate(x: number) -> number {
            let y: number = x * 2;
            let z: number = y + 10;
            return z;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_early_return() {
    let diagnostics = typecheck_source(
        r#"
        fn myAbs(x: number) -> number {
            if (x < 0) {
                return -x;
            }
            return x;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Missing Return Errors ==========

#[test]
fn test_missing_return_error() {
    let diagnostics = typecheck_source(
        r#"
        fn getNumber() -> number {
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // Not all code paths return
}

#[test]
fn test_missing_return_with_statements() {
    let diagnostics = typecheck_source(
        r#"
        fn calculate(x: number) -> number {
            let y: number = x * 2;
            let z: number = y + 10;
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // Not all code paths return
}

#[test]
fn test_missing_return_string_function() {
    let diagnostics = typecheck_source(
        r#"
        fn getMessage() -> string {
            let msg: string = "hello";
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // Not all code paths return
}

#[test]
fn test_missing_return_bool_function() {
    let diagnostics = typecheck_source(
        r#"
        fn isPositive(x: number) -> bool {
            let result: bool = x > 0;
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // Not all code paths return
}

// ========== If/Else Return Analysis ==========

#[test]
fn test_if_else_both_return() {
    let diagnostics = typecheck_source(
        r#"
        fn myAbs(x: number) -> number {
            if (x < 0) {
                return -x;
            } else {
                return x;
            }
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_if_else_only_if_returns_error() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            if (x > 0) {
                return x;
            } else {
                let y: number = 1;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // else branch doesn't return
}

#[test]
fn test_if_else_only_else_returns_error() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            if (x > 0) {
                let y: number = 1;
            } else {
                return x;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // if branch doesn't return
}

#[test]
fn test_if_without_else_returns_error() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            if (x > 0) {
                return x;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // no else branch
}

#[test]
fn test_if_without_else_then_return() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            if (x > 0) {
                return x * 2;
            }
            return x;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Nested If/Else Return Analysis ==========

#[test]
fn test_nested_if_else_all_return() {
    let diagnostics = typecheck_source(
        r#"
        fn classify(x: number) -> number {
            if (x > 0) {
                if (x > 10) {
                    return 2;
                } else {
                    return 1;
                }
            } else {
                return 0;
            }
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_nested_if_missing_inner_return() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            if (x > 0) {
                if (x > 10) {
                    return 2;
                } else {
                    let y: number = 1;
                }
            } else {
                return 0;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // inner else doesn't return
}

#[test]
fn test_nested_if_missing_outer_return() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            if (x > 0) {
                if (x > 10) {
                    return 2;
                } else {
                    return 1;
                }
            } else {
                let y: number = 0;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // outer else doesn't return
}

#[test]
fn test_deeply_nested_all_return() {
    let diagnostics = typecheck_source(
        r#"
        fn classify(x: number, y: number) -> number {
            if (x > 0) {
                if (y > 0) {
                    if (x > y) {
                        return 1;
                    } else {
                        return 2;
                    }
                } else {
                    return 3;
                }
            } else {
                return 4;
            }
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Void Functions (Don't Need Return) ==========

// NOTE: 'void' as a return type may not be fully supported in the parser yet.
// Functions that don't need to return a value can use any return type and
// the compiler will check that all paths return appropriately.

// ========== Multiple Returns in Same Block ==========

#[test]
fn test_multiple_early_returns() {
    let diagnostics = typecheck_source(
        r#"
        fn classify(x: number) -> number {
            if (x < 0) {
                return -1;
            }
            if (x == 0) {
                return 0;
            }
            return 1;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_unreachable_code_after_return() {
    let diagnostics = typecheck_source(
        r#"
        fn test() -> number {
            return 42;
            let x: number = 1;
        }
    "#,
    );
    // This should work (unreachable code is a warning, not an error)
    assert_no_errors(&diagnostics);
}

// ========== Returns in Loops ==========

#[test]
fn test_return_in_while_loop_not_sufficient() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            while (x > 0) {
                return x;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // loop might not execute
}

#[test]
fn test_return_after_loop() {
    let diagnostics = typecheck_source(
        r#"
        fn sum(n: number) -> number {
            var s: number = 0;
            var i: number = 0;
            while (i < n) {
                s = s + i;
                i = i + 1;
            }
            return s;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_in_for_loop_not_sufficient() {
    let diagnostics = typecheck_source(
        r#"
        fn test() -> number {
            for (var i: number = 0; i < 10; i = i + 1) {
                return i;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // loop might not execute
}

#[test]
fn test_return_after_for_loop() {
    let diagnostics = typecheck_source(
        r#"
        fn sum() -> number {
            var s: number = 0;
            for (var i: number = 0; i < 10; i = i + 1) {
                s = s + i;
            }
            return s;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Complex Control Flow ==========
