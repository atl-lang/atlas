use super::*;

// ============================================================================

#[test]
fn test_return_function() {
    let source = r#"
        fn getDouble() -> (number) -> number {
            fn double(borrow x: number) -> number { return x * 2; }
            return double;
        }
        let f = getDouble();
        f(7);
    "#;
    assert_eval_number(source, 14.0);
}

#[test]
fn test_return_builtin() {
    let source = r#"
        fn getLen() -> (string) -> number {
            return len;
        }
        let f = getLen();
        f("test");
    "#;
    assert_eval_number(source, 4.0);
}

#[test]
fn test_return_function_from_parameter() {
    let source = r#"
        fn identity(borrow f: (number) -> number) -> (number) -> number {
            return f;
        }
        fn triple(borrow x: number) -> number { return x * 3; }
        let f = identity(triple);
        f(4);
    "#;
    assert_eval_number(source, 12.0);
}

#[test]
fn test_conditional_function_return() {
    let source = r#"
        fn getFunc(borrow flag: bool) -> (number) -> number {
            fn double(borrow x: number) -> number { return x * 2; }
            fn triple(borrow x: number) -> number { return x * 3; }
            if (flag) {
                return double;
            }
            return triple;
        }
        let f = getFunc(true);
        f(5);
    "#;
    assert_eval_number(source, 10.0);
}

#[test]
fn test_return_function_and_call_immediately() {
    let source = r#"
        fn getDouble() -> (number) -> number {
            fn double(borrow x: number) -> number { return x * 2; }
            return double;
        }
        getDouble()(6);
    "#;
    assert_eval_number(source, 12.0);
}

// ============================================================================
// Category 4: Type Checking (15 tests)
// ============================================================================

#[test]
fn test_type_error_wrong_function_type() {
    let source = r#"
        fn add(borrow a: number, borrow b: number) -> number { return a + b; }
        let f: (number) -> number = add;
    "#;
    assert_error_code(source, "AT3001");
}

#[test]
fn test_type_error_not_a_function() {
    let source = r#"
        let x: number = 5;
        x();
    "#;
    assert_error_code(source, "AT3006");
}

#[test]
fn test_type_error_wrong_return_type() {
    let source = r#"
        fn getString() -> string {
            fn getNum() -> number { return 42; }
            return getNum;
        }
    "#;
    assert_error_code(source, "AT3001");
}

#[test]
fn test_type_valid_function_assignment() {
    let source = r#"
        fn double(borrow x: number) -> number { return x * 2; }
        let f: (number) -> number = double;
        f(5);
    "#;
    assert_eval_number(source, 10.0);
}

#[test]
fn test_type_valid_function_parameter() {
    let source = r#"
        fn apply(borrow f: (string) -> number, s: string) -> number {
            return f(s);
        }
        apply(len, "test");
    "#;
    assert_eval_number(source, 4.0);
}

// ============================================================================
// Category 5: Edge Cases (15 tests)
// ============================================================================

#[test]
fn test_function_returning_void() {
    let source = r#"
        fn getVoid() -> (string) -> void {
            return print;
        }
        let f = getVoid();
        f("test");
    "#;
    assert_eval_null(source);
}

#[test]
fn test_nested_function_calls_through_variables() {
    let source = r#"
        fn add(borrow a: number, borrow b: number) -> number { return a + b; }
        let f = add;
        let g = f;
        let h = g;
        h(2, 3);
    "#;
    assert_eval_number(source, 5.0);
}

#[test]
fn test_function_with_no_params() {
    let source = r#"
        fn getFortyTwo() -> number { return 42; }
        let f: () -> number = getFortyTwo;
        f();
    "#;
    assert_eval_number(source, 42.0);
}

#[test]
fn test_function_with_many_params() {
    let source = r#"
        fn sum4(borrow a: number, borrow b: number, borrow c: number, borrow d: number) -> number {
            return a + b + c + d;
        }
        let f = sum4;
        f(1, 2, 3, 4);
    "#;
    assert_eval_number(source, 10.0);
}

#[test]
fn test_function_variable_in_global_scope() {
    let source = r#"
        fn double(borrow x: number) -> number { return x * 2; }
        let globalFunc = double;
        fn useGlobal(borrow x: number) -> number {
            return globalFunc(x);
        }
        useGlobal(5);
    "#;
    assert_eval_number(source, 10.0);
}

// ============================================================================
// Category 6: Integration Tests (15 tests)
// ============================================================================

#[test]
fn test_function_composition() {
    let source = r#"
        fn compose(
            borrow f: (number) -> number,
            g: (number) -> number
        ) -> (number) -> number {
            fn composed(borrow x: number) -> number {
                return f(g(x));
            }
            return composed;
        }
        fn double(borrow x: number) -> number { return x * 2; }
        fn inc(borrow x: number) -> number { return x + 1; }
        let doubleAndInc = compose(inc, double);
        doubleAndInc(5);
    "#;
    assert_eval_number(source, 11.0);
}

#[test]
fn test_callback_pattern() {
    let source = r#"
        fn processValue(
            borrow x: number,
            borrow callback: (number) -> void
        ) -> void {
            callback(x * 2);
        }
        let mut result = 0;
        fn setResult(borrow x: number) -> void {
            result = x;
        }
        processValue(5, setResult);
        result;
    "#;
    assert_eval_number(source, 10.0);
}

#[test]
fn test_function_array_element() {
    let source = r#"
        fn double(borrow x: number) -> number { return x * 2; }
        fn triple(borrow x: number) -> number { return x * 3; }
        let funcs: ((number) -> number)[] = [double, triple];
        funcs[0](5) + funcs[1](5);
    "#;
    assert_eval_number(source, 25.0);
}

// ============================================================================

// From test_primitives.rs
// ============================================================================

// Integration tests for testing primitives (phase-15)
//
// Verifies that assertion functions work correctly in Atlas code
// and through the stdlib API directly.
//
// Test categories:
// - Basic assertions (assert, assert_false)
// - Equality assertions (assert_equal, assert_not_equal)
// - Result assertions (assert_ok, assert_err)
// - Option assertions (assert_some, assert_none)
// - Collection assertions (assert_contains, assert_empty, assert_length)
// - Error assertions (assert_throws, assert_no_throw via NativeFunction)
// - Stdlib registration (is_builtin, call_builtin)
// - Interpreter/VM parity

// ============================================================================
// Helpers
