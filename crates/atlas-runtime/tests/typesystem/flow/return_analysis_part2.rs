use super::super::*;

#[test]
fn test_if_else_with_early_return() {
    let diagnostics = typecheck_source(
        r#"
        fn complex(x: number, y: number) -> number {
            if (x < 0) {
                return -1;
            }
            if (y < 0) {
                return -2;
            }
            if (x > y) {
                return 1;
            } else {
                return 2;
            }
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_multiple_if_without_final_return() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number, y: number) -> number {
            if (x < 0) {
                return -1;
            }
            if (y < 0) {
                return -2;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // no final return
}

#[test]
fn test_nested_loops_with_return() {
    let diagnostics = typecheck_source(
        r#"
        fn test() -> number {
            var i: number = 0;
            while (i < 10) {
                var j: number = 0;
                while (j < 10) {
                    j = j + 1;
                }
                i = i + 1;
            }
            return i;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Return Type Matching ==========

#[test]
fn test_return_number_to_number() {
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
fn test_return_string_to_string() {
    let diagnostics = typecheck_source(
        r#"
        fn getString() -> string {
            return "hello";
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_bool_to_bool() {
    let diagnostics = typecheck_source(
        r#"
        fn getBool() -> bool {
            return true;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_array() {
    let diagnostics = typecheck_source(
        r#"
        fn getArray() -> number {
            let arr = [1, 2, 3];
            return arr[0];
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Edge Cases ==========

#[test]
fn test_function_returning_number_no_body_error() {
    // Even an empty function needs to return if return type is non-void
    let diagnostics = typecheck_source(
        r#"
        fn getNumber() -> number {
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004");
}

#[test]
fn test_function_with_only_declaration() {
    let diagnostics = typecheck_source(
        r#"
        fn test() -> number {
            let x: number = 42;
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004");
}

#[test]
fn test_all_branches_return_same_value() {
    let diagnostics = typecheck_source(
        r#"
        fn alwaysOne() -> number {
            if (true) {
                return 1;
            } else {
                return 1;
            }
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_if_else_if_else_all_return() {
    let diagnostics = typecheck_source(
        r#"
        fn classify(x: number) -> number {
            if (x < 0) {
                return -1;
            } else {
                if (x == 0) {
                    return 0;
                } else {
                    return 1;
                }
            }
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_simple_return_without_nesting() {
    // Direct return statement works
    let diagnostics = typecheck_source(
        r#"
        fn test() -> number {
            return 42;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_after_if_without_else() {
    let diagnostics = typecheck_source(
        r#"
        fn myMax(a: number, b: number) -> number {
            if (a > b) {
                return a;
            }
            return b;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Multiple Functions ==========

#[test]
fn test_multiple_functions_all_valid() {
    let diagnostics = typecheck_source(
        r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        fn multiply(a: number, b: number) -> number {
            return a * b;
        }

        fn greet() -> string {
            return "Hello";
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_multiple_functions_one_invalid() {
    let diagnostics = typecheck_source(
        r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        fn broken() -> number {
            let x: number = 42;
        }

        fn greet() -> string {
            return "Hello";
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // broken() doesn't return
}

// ============================================================================
