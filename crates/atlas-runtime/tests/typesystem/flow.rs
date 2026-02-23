use super::*;
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

// From type_guard_tests.rs
// ============================================================================

// Tests for type guard predicates and narrowing.

fn eval(code: &str) -> Value {
    let runtime = Atlas::new();
    runtime.eval(code).expect("Interpretation failed")
}

// =============================================================================
// Predicate syntax + validation
// =============================================================================

#[rstest]
#[case(
    r#"
    fn isStr(x: number | string) -> bool is x: string { return isString(x); }
    fn test(x: number | string) -> number {
        if (isStr(x)) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isNum(x: number | string) -> bool is x: number { return isNumber(x); }
    fn test(x: number | string) -> number {
        if (isNum(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isBoolish(x: bool | null) -> bool is x: bool { return isBool(x); }
    fn test(x: bool | null) -> bool {
        if (isBoolish(x)) { let _y: bool = x; return _y; }
        else { return false; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn hasName(x: WithName | WithId) -> bool is x: WithName { return hasField(x, "name"); }
    fn test(x: WithName | WithId) -> number {
        if (hasName(x)) { let _y: WithName = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithLen = { len: () -> number };
    type WithId = { id: number };
    fn hasLen(x: WithLen | WithId) -> bool is x: WithLen { return hasMethod(x, "len"); }
    fn test(x: WithLen | WithId) -> number {
        if (hasLen(x)) { let _y: WithLen = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type Ok = { tag: string, value: number };
    type Err = { tag: number, message: string };
    fn isOk(x: Ok | Err) -> bool is x: Ok { return hasTag(x, "ok"); }
    fn test(x: Ok | Err) -> number {
        if (isOk(x)) { let _y: Ok = x; return 1; }
        else { let _y: Err = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isNullish(x: null | string) -> bool is x: null { return isNull(x); }
    fn test(x: null | string) -> number {
        if (isNullish(x)) { let _y: null = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isObj(x: json | string) -> bool is x: json { return isObject(x); }
    fn test(x: json | string) -> number {
        if (isObj(x)) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isArr(x: number[] | string) -> bool is x: number[] { return isArray(x); }
    fn test(x: number[] | string) -> number {
        if (isArr(x)) { let _y: number[] = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isFunc(x: ((number) -> number) | string) -> bool is x: (number) -> number { return isFunction(x); }
    fn test(x: ((number) -> number) | string) -> number {
        if (isFunc(x)) { let _y: (number) -> number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
fn test_predicate_syntax_valid(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

#[rstest]
#[case(
    r#"
    fn isStr(x: number) -> number is x: number { return 1; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is y: number { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: string { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: number {
        return 1; // return type mismatch
    }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) is x: number { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: number { let _y: string = x; return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number | string) -> bool is x: bool { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number | string) -> bool is missing: string { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: number { return false; }
    fn test(x: number | string) -> number { if (isStr(x)) { return 1; } return 2; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: number { return true; }
    fn test(x: number) -> number { if (isStr(x)) { let _y: string = x; } return 1; }
    "#
)]
fn test_predicate_syntax_errors(#[case] source: &str) {
    let diags = errors(source);
    assert!(!diags.is_empty(), "Expected errors, got none");
}

// =============================================================================
// Built-in guard narrowing
// =============================================================================

#[rstest]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x)) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isNumber(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: bool | null) -> number {
        if (isBool(x)) { let _y: bool = x; return 1; }
        else { let _y: null = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: null | string) -> number {
        if (isNull(x)) { let _y: null = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number[] | string) -> number {
        if (isArray(x)) { let _y: number[] = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn f(x: number) -> number { return x; }
    fn test(x: ((number) -> number) | string) -> number {
        if (isFunction(x)) { let _y: (number) -> number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: json | string) -> number {
        if (isObject(x)) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (!isString(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) || isNumber(x)) { let _y: number | string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) && isType(x, "string")) { let _y: string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "number")) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
fn test_builtin_guard_narrowing(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// User-defined guards
// =============================================================================

#[rstest]
#[case(
    r#"
    fn isText(x: number | string) -> bool is x: string { return isString(x); }
    fn test(x: number | string) -> number {
        if (isText(x)) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn isNamed(x: WithName | WithId) -> bool is x: WithName { return hasField(x, "name"); }
    fn test(x: WithName | WithId) -> number {
        if (isNamed(x)) { let _y: WithName = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithLen = { len: () -> number };
    type WithId = { id: number };
    fn isLen(x: WithLen | WithId) -> bool is x: WithLen { return hasMethod(x, "len"); }
    fn test(x: WithLen | WithId) -> number {
        if (isLen(x)) { let _y: WithLen = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isNum(x: number | string) -> bool is x: number { return isNumber(x); }
    fn test(x: number | string) -> number {
        if (isNum(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isArr(x: number[] | string) -> bool is x: number[] { return isArray(x); }
    fn test(x: number[] | string) -> number {
        if (isArr(x)) { let _y: number[] = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isNullish(x: null | string) -> bool is x: null { return isNull(x); }
    fn test(x: null | string) -> number {
        if (isNullish(x)) { let _y: null = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type Ok = { tag: string, value: number };
    type Err = { tag: number, message: string };
    fn isOk(x: Ok | Err) -> bool is x: Ok { return hasTag(x, "ok"); }
    fn test(x: Ok | Err) -> number {
        if (isOk(x)) { let _y: Ok = x; return 1; }
        else { let _y: Err = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isObj(x: json | string) -> bool is x: json { return isObject(x); }
    fn test(x: json | string) -> number {
        if (isObj(x)) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isFunc(x: ((number) -> number) | string) -> bool is x: (number) -> number { return isFunction(x); }
    fn test(x: ((number) -> number) | string) -> number {
        if (isFunc(x)) { let _y: (number) -> number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isTypeString(x: number | string) -> bool is x: string { return isType(x, "string"); }
    fn test(x: number | string) -> number {
        if (isTypeString(x)) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
fn test_user_defined_guards(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// Guard composition + control flow
// =============================================================================

#[rstest]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) || isNumber(x)) { let _y: number | string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) && isType(x, "string")) { let _y: string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (!isString(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) && !isNull(x)) { let _y: string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "string") || isType(x, "number")) { let _y: number | string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) && isNumber(x)) { return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x)) { let _y: string = x; }
        if (isNumber(x)) { let _y: number = x; }
        return 1;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x)) { let _y: string = x; return 1; }
        if (isNumber(x)) { let _y: number = x; return 2; }
        return 3;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        var result: number = 0;
        if (isString(x)) { result = 1; }
        if (isNumber(x)) { result = 2; }
        return result;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        while (isString(x)) { let _y: string = x; return 1; }
        return 2;
    }
    "#
)]
fn test_guard_composition_and_flow(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// Structural + discriminated guards
// =============================================================================

#[rstest]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (hasField(x, "name")) { let _y: WithName = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithLen = { len: () -> number };
    type WithId = { id: number };
    fn test(x: WithLen | WithId) -> number {
        if (hasMethod(x, "len")) { let _y: WithLen = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithTag = { tag: string, value: number };
    type WithNumTag = { tag: number, message: string };
    fn test(x: WithTag | WithNumTag) -> number {
        if (hasTag(x, "ok")) { let _y: WithTag = x; return 1; }
        else { let _y: WithNumTag = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type One = { name: string, id: number };
    type Two = { id: number };
    fn test(x: One | Two) -> number {
        if (hasField(x, "name")) { let _y: One = x; return 1; }
        else { let _y: Two = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type One = { len: () -> number, id: number };
    type Two = { id: number };
    fn test(x: One | Two) -> number {
        if (hasMethod(x, "len")) { let _y: One = x; return 1; }
        else { let _y: Two = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type One = { tag: string, id: number };
    type Two = { tag: number, id: number };
    fn test(x: One | Two) -> number {
        if (hasTag(x, "one")) { let _y: One = x; return 1; }
        else { let _y: Two = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (hasField(x, "name") && hasField(x, "name")) { let _y: WithName = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (hasField(x, "name") || hasField(x, "id")) { let _y: WithName | WithId = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (!hasField(x, "name")) { let _y: WithId = x; return 1; }
        else { let _y: WithName = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (hasField(x, "name")) { let _y: { name: string } = x; return 1; }
        else { let _y: { id: number } = x; return 2; }
    }
    "#
)]
fn test_structural_guards(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// isType guard tests
// =============================================================================

#[rstest]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "string")) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "number")) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: bool | null) -> number {
        if (isType(x, "bool")) { let _y: bool = x; return 1; }
        else { let _y: null = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: null | string) -> number {
        if (isType(x, "null")) { let _y: null = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number[] | string) -> number {
        if (isType(x, "array")) { let _y: number[] = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn f(x: number) -> number { return x; }
    fn test(x: ((number) -> number) | string) -> number {
        if (isType(x, "function")) { let _y: (number) -> number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: json | string) -> number {
        if (isType(x, "json")) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: json | string) -> number {
        if (isType(x, "object")) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "number") || isType(x, "string")) { let _y: number | string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (!isType(x, "string")) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
fn test_is_type_guard(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// Runtime guard behavior
// =============================================================================

#[rstest]
#[case("isString(\"ok\")", Value::Bool(true))]
#[case("isString(1)", Value::Bool(false))]
#[case("isNumber(1)", Value::Bool(true))]
#[case("isBool(true)", Value::Bool(true))]
#[case("isNull(null)", Value::Bool(true))]
#[case("isArray([1, 2])", Value::Bool(true))]
#[case("isType(\"ok\", \"string\")", Value::Bool(true))]
#[case("isType(1, \"number\")", Value::Bool(true))]
#[case("isType([1, 2], \"array\")", Value::Bool(true))]
#[case("isType(null, \"null\")", Value::Bool(true))]
fn test_runtime_basic_guards(#[case] expr: &str, #[case] expected: Value) {
    let code = expr.to_string();
    let result = eval(&code);
    assert_eq!(result, expected);
}

#[rstest]
#[case(
    r#"
    let obj = parseJSON("{\"tag\":\"ok\", \"value\": 1}");
    hasTag(obj, "ok")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let obj = parseJSON("{\"tag\":\"bad\", \"value\": 1}");
    hasTag(obj, "ok")
    "#,
    Value::Bool(false)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    hasField(obj, "name")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    hasField(obj, "missing")
    "#,
    Value::Bool(false)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    hasMethod(obj, "name")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    hasMethod(obj, "missing")
    "#,
    Value::Bool(false)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    isObject(obj)
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    isType(obj, "object")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let hmap = hashMapNew();
    hashMapPut(hmap, "name", 1);
    hasField(hmap, "name")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let hmap = hashMapNew();
    hashMapPut(hmap, "tag", "ok");
    hasTag(hmap, "ok")
    "#,
    Value::Bool(true)
)]
fn test_runtime_structural_guards(#[case] code: &str, #[case] expected: Value) {
    let result = eval(code);
    assert_eq!(result, expected);
}

// ============================================================================
