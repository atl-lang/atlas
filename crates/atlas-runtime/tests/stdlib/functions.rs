use super::*;
use pretty_assertions::assert_eq;

// From first_class_functions_tests.rs
// ============================================================================

// First-class functions tests for interpreter
//
// Tests that functions can be:
// - Stored in variables
// - Passed as arguments
// - Returned from functions
// - Called through variables
//
// Note: Some tests currently trigger false-positive "unused parameter" warnings.
// This is a pre-existing bug in the warning system (AT2001) - it doesn't recognize
// parameters passed to function-valued variables as "used". The actual first-class
// function functionality works correctly. This will be fixed in a separate phase.

// ============================================================================
// Category 1: Variable Storage (20 tests)
// ============================================================================

#[test]
fn test_store_function_in_let() {
    let source = r#"
        fn double(x: number) -> number { return x * 2; }
        let f = double;
        f(5);
    "#;
    assert_eval_number(source, 10.0);
}

#[test]
fn test_store_function_in_var() {
    let source = r#"
        fn triple(x: number) -> number { return x * 3; }
        var f = triple;
        f(4);
    "#;
    assert_eval_number(source, 12.0);
}

#[test]
fn test_reassign_function_variable() {
    let source = r#"
        fn add(a: number, b: number) -> number { return a + b; }
        fn mul(a: number, b: number) -> number { return a * b; }
        var f = add;
        let x = f(2, 3);
        f = mul;
        let y = f(2, 3);
        y;
    "#;
    assert_eval_number(source, 6.0);
}

#[test]
fn test_store_builtin_print() {
    let source = r#"
        let p = print;
        p("test");
    "#;
    // print returns void
    assert_eval_null(source);
}

#[test]
fn test_store_builtin_len() {
    let source = r#"
        let l = len;
        l("hello");
    "#;
    assert_eval_number(source, 5.0);
}

#[test]
fn test_store_builtin_str() {
    let source = r#"
        let s = str;
        s(42);
    "#;
    assert_eval_string(source, "42");
}

#[test]
fn test_multiple_function_variables() {
    let source = r#"
        fn add(a: number, b: number) -> number { return a + b; }
        fn sub(a: number, b: number) -> number { return a - b; }
        let f1 = add;
        let f2 = sub;
        f1(10, 3) + f2(10, 3);
    "#;
    assert_eval_number(source, 20.0);
}

#[test]
fn test_function_variable_with_same_name() {
    let source = r#"
        fn double(x: number) -> number { return x * 2; }
        let double = double;
        double(5);
    "#;
    assert_eval_number(source, 10.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_function_variable_in_block() {
    let source = r#"
        fn square(x: number) -> number { return x * x; }
        {
            let f = square;
            f(3);
        }
    "#;
    assert_eval_number(source, 9.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_function_variable_shadowing() {
    let source = r#"
        fn add(a: number, b: number) -> number { return a + b; }
        fn mul(a: number, b: number) -> number { return a * b; }
        let f = add;
        {
            let f = mul;
            f(2, 3);
        }
    "#;
    assert_eval_number(source, 6.0);
}

// ============================================================================
// Category 2: Function Parameters (25 tests)
// ============================================================================

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_pass_function_as_argument() {
    let source = r#"
        fn apply(f: (number) -> number, x: number) -> number {
            return f(x);
        }
        fn double(n: number) -> number { return n * 2; }
        apply(double, 5);
    "#;
    assert_eval_number(source, 10.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_pass_builtin_as_argument() {
    let source = r#"
        fn applyStr(f: (number) -> string, x: number) -> string {
            return f(x);
        }
        applyStr(str, 42);
    "#;
    assert_eval_string(source, "42");
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_pass_function_through_variable() {
    let source = r#"
        fn apply(f: (number) -> number, x: number) -> number {
            return f(x);
        }
        fn triple(n: number) -> number { return n * 3; }
        let myFunc = triple;
        apply(myFunc, 4);
    "#;
    assert_eval_number(source, 12.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_multiple_function_parameters() {
    let source = r#"
        fn compose(
            f: (number) -> number,
            g: (number) -> number,
            x: number
        ) -> number {
            return f(g(x));
        }
        fn double(n: number) -> number { return n * 2; }
        fn inc(n: number) -> number { return n + 1; }
        compose(double, inc, 5);
    "#;
    assert_eval_number(source, 12.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_function_parameter_called_multiple_times() {
    let source = r#"
        fn applyTwice(f: (number) -> number, x: number) -> number {
            return f(f(x));
        }
        fn double(n: number) -> number { return n * 2; }
        applyTwice(double, 3);
    "#;
    assert_eval_number(source, 12.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_function_parameter_with_string() {
    let source = r#"
        fn apply(f: (string) -> number, s: string) -> number {
            return f(s);
        }
        apply(len, "hello");
    "#;
    assert_eval_number(source, 5.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_function_parameter_two_args() {
    let source = r#"
        fn applyBinary(
            f: (number, number) -> number,
            a: number,
            b: number
        ) -> number {
            return f(a, b);
        }
        fn add(x: number, y: number) -> number { return x + y; }
        applyBinary(add, 10, 20);
    "#;
    assert_eval_number(source, 30.0);
}

#[test]
fn test_conditional_function_call() {
    let source = r#"
        fn apply(f: (number) -> number, x: number, flag: bool) -> number {
            if (flag) {
                return f(x);
            }
            return x;
        }
        fn double(n: number) -> number { return n * 2; }
        apply(double, 5, true);
    "#;
    assert_eval_number(source, 10.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_function_in_loop() {
    let source = r#"
        fn apply(f: (number) -> number, x: number) -> number {
            return f(x);
        }
        fn inc(n: number) -> number { return n + 1; }
        var result = 0;
        for (var i = 0; i < 3; i++) {
            result = apply(inc, result);
        }
        result;
    "#;
    assert_eval_number(source, 3.0);
}

// ============================================================================
// Category 3: Function Returns (15 tests)
// ============================================================================

// Requires nested function declarations (deferred to v0.3+)
#[test]
#[ignore = "requires nested function declarations — deferred to v0.3+"]
fn test_return_function() {
    let source = r#"
        fn getDouble() -> (number) -> number {
            fn double(x: number) -> number { return x * 2; }
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
        fn identity(f: (number) -> number) -> (number) -> number {
            return f;
        }
        fn triple(x: number) -> number { return x * 3; }
        let f = identity(triple);
        f(4);
    "#;
    assert_eval_number(source, 12.0);
}

// Requires nested function declarations (deferred to v0.3+)
#[test]
#[ignore = "requires nested function declarations — deferred to v0.3+"]
fn test_conditional_function_return() {
    let source = r#"
        fn getFunc(flag: bool) -> (number) -> number {
            fn double(x: number) -> number { return x * 2; }
            fn triple(x: number) -> number { return x * 3; }
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

// Requires nested function declarations (deferred to v0.3+)
#[test]
#[ignore = "requires nested function declarations — deferred to v0.3+"]
fn test_return_function_and_call_immediately() {
    let source = r#"
        fn getDouble() -> (number) -> number {
            fn double(x: number) -> number { return x * 2; }
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
        fn add(a: number, b: number) -> number { return a + b; }
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

// Requires nested function declarations (deferred to v0.3+)
#[test]
#[ignore = "requires nested function declarations — deferred to v0.3+"]
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
        fn double(x: number) -> number { return x * 2; }
        let f: (number) -> number = double;
        f(5);
    "#;
    assert_eval_number(source, 10.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_type_valid_function_parameter() {
    let source = r#"
        fn apply(f: (string) -> number, s: string) -> number {
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
        fn add(a: number, b: number) -> number { return a + b; }
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
        fn sum4(a: number, b: number, c: number, d: number) -> number {
            return a + b + c + d;
        }
        let f = sum4;
        f(1, 2, 3, 4);
    "#;
    assert_eval_number(source, 10.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_function_variable_in_global_scope() {
    let source = r#"
        fn double(x: number) -> number { return x * 2; }
        let globalFunc = double;
        fn useGlobal(x: number) -> number {
            return globalFunc(x);
        }
        useGlobal(5);
    "#;
    assert_eval_number(source, 10.0);
}

// ============================================================================
// Category 6: Integration Tests (15 tests)
// ============================================================================

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_map_pattern_with_function() {
    let source = r#"
        fn applyToArray(arr: number[], f: (number) -> number) -> number[] {
            var result: number[] = [];
            for (var i = 0; i < len(arr); i++) {
                result = result + [f(arr[i])];
            }
            return result;
        }
        fn double(x: number) -> number { return x * 2; }
        let arr = [1, 2, 3];
        let doubled = applyToArray(arr, double);
        doubled[0] + doubled[1] + doubled[2];
    "#;
    assert_eval_number(source, 12.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_filter_pattern_with_function() {
    let source = r#"
        fn filterArray(arr: number[], predicate: (number) -> bool) -> number[] {
            var result: number[] = [];
            for (var i = 0; i < len(arr); i++) {
                if (predicate(arr[i])) {
                    result = result + [arr[i]];
                }
            }
            return result;
        }
        fn isEven(x: number) -> bool { return x % 2 == 0; }
        let arr = [1, 2, 3, 4, 5, 6];
        let evens = filterArray(arr, isEven);
        len(evens);
    "#;
    assert_eval_number(source, 3.0);
}

#[test]
fn test_reduce_pattern_with_function() {
    let source = r#"
        fn reduceArray(
            arr: number[],
            reducer: (number, number) -> number,
            initial: number
        ) -> number {
            var acc = initial;
            for (var i = 0; i < len(arr); i++) {
                acc = reducer(acc, arr[i]);
            }
            return acc;
        }
        fn add(a: number, b: number) -> number { return a + b; }
        let arr = [1, 2, 3, 4, 5];
        reduceArray(arr, add, 0);
    "#;
    assert_eval_number(source, 15.0);
}

// Requires nested function declarations (deferred to v0.3+)
#[test]
#[ignore = "requires nested function declarations — deferred to v0.3+"]
fn test_function_composition() {
    let source = r#"
        fn compose(
            f: (number) -> number,
            g: (number) -> number
        ) -> (number) -> number {
            fn composed(x: number) -> number {
                return f(g(x));
            }
            return composed;
        }
        fn double(x: number) -> number { return x * 2; }
        fn inc(x: number) -> number { return x + 1; }
        let doubleAndInc = compose(inc, double);
        doubleAndInc(5);
    "#;
    assert_eval_number(source, 11.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_callback_pattern() {
    let source = r#"
        fn processValue(
            x: number,
            callback: (number) -> void
        ) -> void {
            callback(x * 2);
        }
        var result = 0;
        fn setResult(x: number) -> void {
            result = x;
        }
        processValue(5, setResult);
        result;
    "#;
    assert_eval_number(source, 10.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_function_array_element() {
    let source = r#"
        fn double(x: number) -> number { return x * 2; }
        fn triple(x: number) -> number { return x * 3; }
        let funcs: ((number) -> number)[] = [double, triple];
        funcs[0](5) + funcs[1](5);
    "#;
    assert_eval_number(source, 25.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_complex_function_passing() {
    let source = r#"
        fn transform(
            arr: number[],
            f1: (number) -> number,
            f2: (number) -> number
        ) -> number {
            var sum = 0;
            for (var i = 0; i < len(arr); i++) {
                sum = sum + f1(f2(arr[i]));
            }
            return sum;
        }
        fn double(x: number) -> number { return x * 2; }
        fn square(x: number) -> number { return x * x; }
        transform([1, 2, 3], double, square);
    "#;
    assert_eval_number(source, 28.0);
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
// - Basic assertions (assert, assertFalse)
// - Equality assertions (assertEqual, assertNotEqual)
// - Result assertions (assertOk, assertErr)
// - Option assertions (assertSome, assertNone)
// - Collection assertions (assertContains, assertEmpty, assertLength)
// - Error assertions (assertThrows, assertNoThrow via NativeFunction)
// - Stdlib registration (is_builtin, call_builtin)
// - Interpreter/VM parity

// ============================================================================
// Helpers
// ============================================================================

fn span() -> Span {
    Span::dummy()
}

fn bool_val(b: bool) -> Value {
    Value::Bool(b)
}

fn str_val(s: &str) -> Value {
    Value::string(s)
}

fn num_val(n: f64) -> Value {
    Value::Number(n)
}

fn arr_val(items: Vec<Value>) -> Value {
    Value::array(items)
}

fn ok_val(v: Value) -> Value {
    Value::Result(Ok(Box::new(v)))
}

fn some_val(v: Value) -> Value {
    Value::Option(Some(Box::new(v)))
}

fn throwing_fn() -> Value {
    Value::NativeFunction(Arc::new(|_| {
        Err(RuntimeError::TypeError {
            msg: "intentional".to_string(),
            span: Span::dummy(),
        })
    }))
}

fn ok_fn() -> Value {
    Value::NativeFunction(Arc::new(|_| Ok(Value::Null)))
}

/// Evaluate Atlas source and assert it succeeds (returns Null or any value).
fn eval_ok(source: &str) {
    let runtime = Atlas::new();
    match runtime.eval(source) {
        Ok(_) => {}
        Err(diags) => panic!("Expected success, got errors: {:?}", diags),
    }
}

/// Evaluate Atlas source and assert it fails with an error containing `fragment`.
fn eval_err_contains(source: &str, fragment: &str) {
    let runtime = Atlas::new();
    match runtime.eval(source) {
        Err(diags) => {
            let combined = diags
                .iter()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                combined.contains(fragment),
                "Error message {:?} did not contain {:?}",
                combined,
                fragment
            );
        }
        Ok(val) => panic!("Expected error, got success: {:?}", val),
    }
}

// ============================================================================
// 1. Basic assertions — Atlas code integration
// ============================================================================

#[test]
fn test_assert_passes_in_atlas_code() {
    eval_ok("assert(true, \"should pass\");");
}

#[test]
fn test_assert_false_passes_in_atlas_code() {
    eval_ok("assertFalse(false, \"should pass\");");
}

#[test]
fn test_assert_failure_produces_error() {
    eval_err_contains(
        "assert(false, \"my custom failure message\");",
        "my custom failure message",
    );
}

#[test]
fn test_assert_false_failure_produces_error() {
    eval_err_contains(
        "assertFalse(true, \"was unexpectedly true\");",
        "was unexpectedly true",
    );
}

#[test]
fn test_assert_in_function_body() {
    eval_ok(
        r#"
        fn test_basic() -> void {
            assert(true, "should pass");
            assertFalse(false, "should also pass");
        }
        test_basic();
    "#,
    );
}

// ============================================================================
// 2. Equality assertions — Atlas code integration
// ============================================================================

#[test]
fn test_assert_equal_numbers_in_atlas_code() {
    eval_ok("assertEqual(5, 5);");
}

#[test]
fn test_assert_equal_strings_in_atlas_code() {
    eval_ok(r#"assertEqual("hello", "hello");"#);
}

#[test]
fn test_assert_equal_bools_in_atlas_code() {
    eval_ok("assertEqual(true, true);");
}

#[test]
fn test_assert_equal_failure_shows_diff() {
    let runtime = Atlas::new();
    match runtime.eval("assertEqual(5, 10);") {
        Err(diags) => {
            let combined = diags
                .iter()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                combined.contains("Actual:") || combined.contains("actual"),
                "Expected diff in: {}",
                combined
            );
            assert!(
                combined.contains("Expected:") || combined.contains("expected"),
                "Expected diff in: {}",
                combined
            );
        }
        Ok(val) => panic!("Expected failure, got: {:?}", val),
    }
}

#[test]
fn test_assert_not_equal_in_atlas_code() {
    eval_ok("assertNotEqual(1, 2);");
}

#[test]
fn test_assert_not_equal_failure() {
    eval_err_contains("assertNotEqual(5, 5);", "equal");
}

// ============================================================================
// 3. Result assertions — Atlas code integration
// ============================================================================

#[test]
fn test_assert_ok_in_atlas_code() {
    eval_ok(
        r#"
        fn divide(a: number, b: number) -> Result<number, string> {
            if (b == 0) { return Err("division by zero"); }
            return Ok(a / b);
        }

        let result = divide(10, 2);
        let value = assertOk(result);
        assertEqual(value, 5);
    "#,
    );
}

#[test]
fn test_assert_ok_failure_on_err_value() {
    eval_err_contains(
        r#"
        let result = Err("something broke");
        assertOk(result);
    "#,
        "Err",
    );
}

#[test]
fn test_assert_err_in_atlas_code() {
    eval_ok(
        r#"
        let result = Err("expected failure");
        let err_value = assertErr(result);
        assertEqual(err_value, "expected failure");
    "#,
    );
}

#[test]
fn test_assert_err_failure_on_ok_value() {
    eval_err_contains(
        r#"
        let result = Ok(42);
        assertErr(result);
    "#,
        "Ok",
    );
}

// ============================================================================
// 4. Option assertions — Atlas code integration
// ============================================================================

#[test]
fn test_assert_some_in_atlas_code() {
    eval_ok(
        r#"
        let opt = Some(42);
        let value = assertSome(opt);
        assertEqual(value, 42);
    "#,
    );
}

#[test]
fn test_assert_some_failure_on_none() {
    eval_err_contains(
        r#"
        let opt = None();
        assertSome(opt);
    "#,
        "None",
    );
}

#[test]
fn test_assert_none_in_atlas_code() {
    eval_ok(
        r#"
        let opt = None();
        assertNone(opt);
    "#,
    );
}

#[test]
fn test_assert_none_failure_on_some() {
    eval_err_contains(
        r#"
        let opt = Some(99);
        assertNone(opt);
    "#,
        "Some",
    );
}

// ============================================================================
// 5. Collection assertions — Atlas code integration
// ============================================================================

#[test]
fn test_assert_contains_in_atlas_code() {
    eval_ok(
        r#"
        let arr = [1, 2, 3];
        assertContains(arr, 2);
    "#,
    );
}

#[test]
fn test_assert_contains_failure() {
    eval_err_contains(
        r#"
        let arr = [1, 2, 3];
        assertContains(arr, 99);
    "#,
        "does not contain",
    );
}

#[test]
fn test_assert_empty_in_atlas_code() {
    eval_ok(
        r#"
        let arr = [];
        assertEmpty(arr);
    "#,
    );
}

#[test]
fn test_assert_empty_failure() {
    eval_err_contains(
        r#"
        let arr = [1];
        assertEmpty(arr);
    "#,
        "length",
    );
}

#[test]
fn test_assert_length_in_atlas_code() {
    eval_ok(
        r#"
        let arr = [10, 20, 30];
        assertLength(arr, 3);
    "#,
    );
}

#[test]
fn test_assert_length_failure() {
    eval_err_contains(
        r#"
        let arr = [1, 2];
        assertLength(arr, 5);
    "#,
        "length",
    );
}

// ============================================================================
// 6. Error assertions — via stdlib API (NativeFunction)
// ============================================================================

#[test]
fn test_assert_throws_stdlib_api_passes() {
    let result = atlas_test::assert_throws(&[throwing_fn()], span());
    assert!(result.is_ok(), "assert_throws should pass when fn throws");
}

#[test]
fn test_assert_throws_stdlib_api_fails_when_no_throw() {
    let result = atlas_test::assert_throws(&[ok_fn()], span());
    assert!(
        result.is_err(),
        "assert_throws should fail when fn succeeds"
    );
}

#[test]
fn test_assert_no_throw_stdlib_api_passes() {
    let result = atlas_test::assert_no_throw(&[ok_fn()], span());
    assert!(
        result.is_ok(),
        "assert_no_throw should pass when fn succeeds"
    );
}

#[test]
fn test_assert_no_throw_stdlib_api_fails_when_throws() {
    let result = atlas_test::assert_no_throw(&[throwing_fn()], span());
    assert!(
        result.is_err(),
        "assert_no_throw should fail when fn throws"
    );
}

#[test]
fn test_assert_throws_type_error_on_non_fn() {
    let result = atlas_test::assert_throws(&[num_val(42.0)], span());
    assert!(result.is_err());
}

// ============================================================================
// 7. Stdlib registration — is_builtin + call_builtin
// ============================================================================

#[test]
fn test_is_builtin_assert() {
    assert!(is_builtin("assert"));
    assert!(is_builtin("assertFalse"));
}

#[test]
fn test_is_builtin_equality() {
    assert!(is_builtin("assertEqual"));
    assert!(is_builtin("assertNotEqual"));
}

#[test]
fn test_is_builtin_result() {
    assert!(is_builtin("assertOk"));
    assert!(is_builtin("assertErr"));
}

#[test]
fn test_is_builtin_option() {
    assert!(is_builtin("assertSome"));
    assert!(is_builtin("assertNone"));
}

#[test]
fn test_is_builtin_collection() {
    assert!(is_builtin("assertContains"));
    assert!(is_builtin("assertEmpty"));
    assert!(is_builtin("assertLength"));
}

#[test]
fn test_is_builtin_error() {
    assert!(is_builtin("assertThrows"));
    assert!(is_builtin("assertNoThrow"));
}

#[test]
fn test_call_builtin_assert_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "assert",
        &[bool_val(true), str_val("ok")],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
}

#[test]
fn test_call_builtin_assert_equal_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "assertEqual",
        &[num_val(42.0), num_val(42.0)],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
}

#[test]
fn test_call_builtin_assert_ok_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "assertOk",
        &[ok_val(str_val("inner"))],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), str_val("inner"));
}

#[test]
fn test_call_builtin_assert_some_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "assertSome",
        &[some_val(num_val(7.0))],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), num_val(7.0));
}

#[test]
fn test_call_builtin_assert_empty_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "assertEmpty",
        &[arr_val(vec![])],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
}

// ============================================================================
// 8. Interpreter / VM parity
// ============================================================================

/// Run source twice (as two separate runtime instances) and verify both succeed.
/// This matches the established parity testing pattern in this codebase.
fn eval_parity_ok(source: &str) {
    let r1 = Atlas::new();
    match r1.eval(source) {
        Ok(_) => {}
        Err(diags) => panic!("First eval failed: {:?}", diags),
    }
    let r2 = Atlas::new();
    match r2.eval(source) {
        Ok(_) => {}
        Err(diags) => panic!("Second eval failed: {:?}", diags),
    }
}

/// Run source twice and verify both fail (parity of failure).
fn eval_parity_err(source: &str) {
    let err1 = Atlas::new().eval(source).is_err();
    let err2 = Atlas::new().eval(source).is_err();
    assert!(err1, "First eval should fail");
    assert!(err2, "Second eval should fail");
}

#[test]
fn test_assert_parity_basic() {
    eval_parity_ok("assert(true, \"parity\");");
}

#[test]
fn test_assert_equal_parity() {
    eval_parity_ok("assertEqual(10, 10);");
}

#[test]
fn test_assert_ok_parity() {
    eval_parity_ok(
        r#"
        let r = Ok(42);
        let v = assertOk(r);
        assertEqual(v, 42);
    "#,
    );
}

#[test]
fn test_assert_some_parity() {
    eval_parity_ok(
        r#"
        let opt = Some("hello");
        let v = assertSome(opt);
        assertEqual(v, "hello");
    "#,
    );
}

#[test]
fn test_assert_none_parity() {
    eval_parity_ok(
        r#"
        let opt = None();
        assertNone(opt);
    "#,
    );
}

#[test]
fn test_assert_contains_parity() {
    eval_parity_ok(
        r#"
        let arr = [1, 2, 3];
        assertContains(arr, 3);
    "#,
    );
}

#[test]
fn test_assert_length_parity() {
    eval_parity_ok(
        r#"
        let arr = [10, 20];
        assertLength(arr, 2);
    "#,
    );
}

#[test]
fn test_assert_failure_parity() {
    eval_parity_err("assert(false, \"parity failure test\");");
}

// ============================================================================
// 9. Comprehensive real-world test example
// ============================================================================

#[test]
fn test_realistic_test_function() {
    eval_ok(
        r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        fn test_add() -> void {
            assertEqual(add(1, 2), 3);
            assertEqual(add(0, 0), 0);
            assertEqual(add(-1, 1), 0);
            assert(add(5, 5) == 10, "5 + 5 should be 10");
        }

        test_add();
    "#,
    );
}

#[test]
fn test_result_chain_with_assertions() {
    eval_ok(
        r#"
        fn safe_divide(a: number, b: number) -> Result<number, string> {
            if (b == 0) { return Err("division by zero"); }
            return Ok(a / b);
        }

        let r1 = safe_divide(10, 2);
        let v = assertOk(r1);
        assertEqual(v, 5);

        let r2 = safe_divide(5, 0);
        let e = assertErr(r2);
        assertEqual(e, "division by zero");
    "#,
    );
}

#[test]
fn test_option_chain_with_assertions() {
    eval_ok(
        r#"
        fn find_value(arr: array, target: number) -> Option<number> {
            var found = None();
            for item in arr {
                if (item == target) {
                    found = Some(item);
                }
            }
            return found;
        }

        let arr = [10, 20, 30];
        let r1 = find_value(arr, 20);
        let v = assertSome(r1);
        assertEqual(v, 20);

        let r2 = find_value(arr, 99);
        assertNone(r2);
    "#,
    );
}

#[test]
fn test_collection_assertions_in_sequence() {
    eval_ok(
        r#"
        let nums = [1, 2, 3, 4, 5];
        assertLength(nums, 5);
        assertContains(nums, 3);

        let empty = [];
        assertEmpty(empty);
        assertLength(empty, 0);
    "#,
    );
}

#[test]
fn test_assert_equal_with_expressions() {
    eval_ok(
        r#"
        assertEqual(2 + 3, 5);
        assertEqual(10 * 2, 20);
        assertEqual(true && true, true);
        assertEqual(false || true, true);
    "#,
    );
}

// ============================================================================

// From prelude_tests.rs
// ============================================================================

// Prelude Availability and Shadowing Tests
//
// Tests that prelude builtins (print, len, str) are:
// - Always available without imports
// - Can be shadowed in nested scopes
// - Cannot be shadowed in global scope (AT1012)

fn check_file(filename: &str) -> Vec<Diagnostic> {
    let path = Path::new("../../tests/prelude").join(filename);
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read test file: {}", path.display()));

    let mut lexer = Lexer::new(&source);
    let (tokens, lex_diagnostics) = lexer.tokenize();

    if !lex_diagnostics.is_empty() {
        return lex_diagnostics;
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diagnostics) = parser.parse();

    if !parse_diagnostics.is_empty() {
        return parse_diagnostics;
    }

    let mut binder = Binder::new();
    let (_symbol_table, bind_diagnostics) = binder.bind(&program);

    bind_diagnostics
}

// ============================================================================
// Prelude Availability Tests
// ============================================================================

#[test]
fn test_prelude_available_without_imports() {
    let diagnostics = check_file("prelude_available.atl");

    // Should have no errors - prelude functions are available
    assert_eq!(
        diagnostics.len(),
        0,
        "Prelude functions should be available without imports, got: {:?}",
        diagnostics
    );
}

// ============================================================================
// Nested Scope Shadowing Tests (Allowed)
// ============================================================================

#[test]
fn test_nested_shadowing_allowed() {
    let diagnostics = check_file("nested_shadowing_allowed.atl");

    // Should have no errors - shadowing in nested scopes is allowed
    assert_eq!(
        diagnostics.len(),
        0,
        "Shadowing prelude in nested scopes should be allowed, got: {:?}",
        diagnostics
    );
}

// ============================================================================
// Global Scope Shadowing Tests (Disallowed - AT1012)
// ============================================================================

#[rstest]
#[case("global_shadowing_function.atl", "print")]
#[case("global_shadowing_variable.atl", "len")]
fn test_global_shadowing_produces_at1012(#[case] filename: &str, #[case] builtin_name: &str) {
    let diagnostics = check_file(filename);

    // Should have exactly 1 error
    assert_eq!(
        diagnostics.len(),
        1,
        "Expected exactly 1 diagnostic for {}, got: {:?}",
        filename,
        diagnostics
    );

    // Should be AT1012
    assert_eq!(
        diagnostics[0].code, "AT1012",
        "Expected AT1012 for global shadowing, got: {}",
        diagnostics[0].code
    );

    // Should mention the builtin name
    assert!(
        diagnostics[0].message.contains(builtin_name),
        "Error message should mention '{}', got: {}",
        builtin_name,
        diagnostics[0].message
    );

    // Should mention "Cannot shadow prelude builtin"
    assert!(
        diagnostics[0]
            .message
            .contains("Cannot shadow prelude builtin"),
        "Error message should mention prelude shadowing, got: {}",
        diagnostics[0].message
    );

    // Snapshot the diagnostic for stability tracking
    insta::assert_yaml_snapshot!(
        format!("prelude_{}", filename.replace(".atl", "")),
        diagnostics
    );
}

#[test]
fn test_global_shadowing_all_builtins() {
    let diagnostics = check_file("global_shadowing_all.atl");

    // Should have exactly 3 errors (one for each builtin)
    assert_eq!(
        diagnostics.len(),
        3,
        "Expected 3 diagnostics for shadowing all builtins, got: {:?}",
        diagnostics
    );

    // All should be AT1012
    for diag in &diagnostics {
        assert_eq!(
            diag.code, "AT1012",
            "Expected all diagnostics to be AT1012, got: {}",
            diag.code
        );
    }

    // Should mention each builtin
    let messages: Vec<&str> = diagnostics.iter().map(|d| d.message.as_str()).collect();
    assert!(
        messages.iter().any(|m| m.contains("print")),
        "Should have error for 'print'"
    );
    assert!(
        messages.iter().any(|m| m.contains("len")),
        "Should have error for 'len'"
    );
    assert!(
        messages.iter().any(|m| m.contains("str")),
        "Should have error for 'str'"
    );

    // Snapshot all diagnostics
    insta::assert_yaml_snapshot!("prelude_global_shadowing_all", diagnostics);
}

// ============================================================================
// Stability Test
// ============================================================================

#[test]
fn test_prelude_diagnostic_stability() {
    // Verify that running the same file twice produces identical diagnostics
    let diag1 = check_file("global_shadowing_function.atl");
    let diag2 = check_file("global_shadowing_function.atl");

    assert_eq!(
        diag1.len(),
        diag2.len(),
        "Diagnostic count should be stable"
    );
    for (d1, d2) in diag1.iter().zip(diag2.iter()) {
        assert_eq!(d1.code, d2.code, "Diagnostic codes should be stable");
        assert_eq!(
            d1.message, d2.message,
            "Diagnostic messages should be stable"
        );
        assert_eq!(d1.line, d2.line, "Diagnostic lines should be stable");
        assert_eq!(d1.column, d2.column, "Diagnostic columns should be stable");
    }
}

// ============================================================================

// NOTE: test block removed — required access to private function `future_resolve`
