use super::super::*;

#[test]
fn test_if_else_with_early_return() {
    let diagnostics = typecheck_source(
        r#"
        fn complex(borrow x: number, borrow y: number): number {
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
        fn test(borrow x: number, borrow y: number): number {
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
        fn test(): number {
            let mut i: number = 0;
            while (i < 10) {
                let mut j: number = 0;
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
        fn getNumber(): number {
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
        fn getString(): string {
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
        fn getBool(): bool {
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
        fn getArray(): number {
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
        fn getNumber(): number {
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004");
}

#[test]
fn test_function_with_only_declaration() {
    let diagnostics = typecheck_source(
        r#"
        fn test(): number {
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
        fn alwaysOne(): number {
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
        fn classify(borrow x: number): number {
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
        fn test(): number {
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
        fn myMax(borrow a: number, borrow b: number): number {
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
        fn add(borrow a: number, borrow b: number): number {
            return a + b;
        }

        fn multiply(borrow a: number, borrow b: number): number {
            return a * b;
        }

        fn greet(): string {
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
        fn add(borrow a: number, borrow b: number): number {
            return a + b;
        }

        fn broken(): number {
            let x: number = 42;
        }

        fn greet(): string {
            return "Hello";
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // broken() doesn't return
}

// ============================================================================
#[test]
fn test_return_in_match_arm_typed_as_never_h184() {
    // H-184: `return` in a match arm must be typed as Never (bottom type),
    // which is compatible with any other arm type.
    // Pattern: match result { Ok(v) => v, Err(e) => return Err(e) }
    let diagnostics = typecheck_source(
        r#"
        fn try_parse(borrow _s: string): Result<number, string> { return Ok(42); }
        fn parse_and_double(borrow s: string): Result<number, string> {
            let n = match try_parse(s) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };
            return Ok(n * 2.0);
        }
        "#,
    );
    assert_no_errors(&diagnostics);
}

// ============================================================================
// H-238: block_always_returns missing match/while/for coverage
// ============================================================================

#[test]
fn test_h238_match_only_body_no_false_at3004() {
    // Function body is ONLY a match with all arms returning — must NOT emit AT3004.
    let diagnostics = typecheck_source(
        r#"
        fn classify(borrow n: number): string {
            match n {
                0 => return "zero",
                1 => return "one",
                _ => return "many",
            }
        }
        "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_h238_while_with_trailing_return_no_at3004() {
    // While loop followed by a trailing return — must NOT emit AT3004.
    let diagnostics = typecheck_source(
        r#"
        fn find_first(borrow arr: number[]): number {
            let mut i = 0;
            while i < len(arr) {
                if arr[i] > 0 {
                    return arr[i];
                }
                i = i + 1;
            }
            return -1;
        }
        "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_h238_for_with_trailing_return_no_at3004() {
    // For loop followed by a trailing return — must NOT emit AT3004.
    let diagnostics = typecheck_source(
        r#"
        fn sum(borrow arr: number[]): number {
            let mut total = 0;
            for x in arr {
                total = total + x;
            }
            return total;
        }
        "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_h238_match_in_if_else_no_at3004() {
    // Match inside if/else — both branches return via match — must NOT emit AT3004.
    let diagnostics = typecheck_source(
        r#"
        fn describe(borrow n: number): string {
            if n < 0 {
                match n {
                    _ => return "negative",
                }
            } else {
                match n {
                    0 => return "zero",
                    _ => return "positive",
                }
            }
        }
        "#,
    );
    assert_no_errors(&diagnostics);
}
