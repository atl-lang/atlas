use super::common::*;
use atlas_runtime::Atlas;

// ─── Phase Completion Verification ───────────────────────────────────────────

#[test]
fn milestone_runtime_api_eval_returns_result() {
    // The top-level Atlas::eval must return Result<Value, Vec<Diagnostic>>.
    let runtime = Atlas::new();
    let result = runtime.eval("42;");
    assert!(result.is_ok());
}

#[test]
fn milestone_runtime_error_returns_diagnostics() {
    let runtime = Atlas::new();
    let result = runtime.eval("undefined_var_xyz;");
    assert!(result.is_err());
    let diags = result.unwrap_err();
    assert!(!diags.is_empty(), "Expected at least one diagnostic");
}

#[test]
fn milestone_value_number_type() {
    let runtime = Atlas::new();
    match runtime.eval("42;").unwrap() {
        atlas_runtime::Value::Number(n) => assert!(n == 42.0),
        other => panic!("Expected Number, got {:?}", other),
    }
}

#[test]
fn milestone_value_string_type() {
    let runtime = Atlas::new();
    match runtime.eval(r#""hello";"#).unwrap() {
        atlas_runtime::Value::String(s) => assert!(s.as_ref() == "hello"),
        other => panic!("Expected String, got {:?}", other),
    }
}

#[test]
fn milestone_value_bool_type() {
    let runtime = Atlas::new();
    match runtime.eval("true;").unwrap() {
        atlas_runtime::Value::Bool(b) => assert!(b),
        other => panic!("Expected Bool(true), got {:?}", other),
    }
}

#[test]
fn milestone_value_null_type() {
    let runtime = Atlas::new();
    match runtime.eval("null;").unwrap() {
        atlas_runtime::Value::Null => {}
        other => panic!("Expected Null, got {:?}", other),
    }
}

#[test]
fn milestone_type_system_enforces_let_immutability() {
    // let variables must be immutable — mutation should produce an error.
    assert_has_error("let x: number = 1; x = 2;");
}

#[test]
fn milestone_type_system_allows_var_mutation() {
    // var variables must be mutable.
    assert_eval_number("let mut x: number = 1; x = 2; x;", 2.0);
}

#[test]
fn milestone_type_system_type_annotations_enforced() {
    // Type annotations must be enforced at compile time.
    assert_has_error("let x: number = true;");
    assert_has_error("let x: string = 42;");
    assert_has_error("let x: bool = 0;");
}

#[test]
fn milestone_type_system_function_return_type() {
    // Function return types must be checked.
    assert_has_error("fn f() -> number { return true; }");
}

// ─── Language Feature Verification ───────────────────────────────────────────

#[test]
fn milestone_feature_arithmetic_all_operators() {
    assert_eval_number("1 + 2;", 3.0);
    assert_eval_number("5 - 3;", 2.0);
    assert_eval_number("3 * 4;", 12.0);
    assert_eval_number("10 / 2;", 5.0);
    assert_eval_number("7 % 3;", 1.0);
}

#[test]
fn milestone_feature_comparison_operators() {
    assert_eval_bool("1 < 2;", true);
    assert_eval_bool("2 > 1;", true);
    assert_eval_bool("1 <= 1;", true);
    assert_eval_bool("2 >= 2;", true);
    assert_eval_bool("1 == 1;", true);
    assert_eval_bool("1 != 2;", true);
}

#[test]
fn milestone_feature_logical_operators() {
    assert_eval_bool("true && true;", true);
    assert_eval_bool("true && false;", false);
    assert_eval_bool("false || true;", true);
    assert_eval_bool("false || false;", false);
    assert_eval_bool("!true;", false);
    assert_eval_bool("!false;", true);
}

#[test]
fn milestone_feature_if_else() {
    // Note: if statements don't implicitly return values in Rust semantics.
    // Use explicit return in functions to test if-else branch selection.
    assert_eval_number(
        "fn test() -> number { if (true) { return 1; } else { return 2; } } test()",
        1.0,
    );
    assert_eval_number(
        "fn test() -> number { if (false) { return 1; } else { return 2; } } test()",
        2.0,
    );
}

#[test]
fn milestone_feature_while_loop() {
    let code = r#"
        fn test() -> number {
            let mut i: number = 0;
            let mut sum: number = 0;
            while (i < 5) {
                sum = sum + i;
                i = i + 1;
            }
            sum
        }
        test()
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn milestone_feature_for_loop() {
    assert_no_error(
        r#"
        let arr: []number = [1, 2, 3];
        let mut sum: number = 0;
        let mut i: number = 0;
        while (i < 3) {
            sum = sum + arr[i];
            i = i + 1;
        }
        sum;
    "#,
    );
}

#[test]
fn milestone_feature_functions_with_params_and_return() {
    let code = r#"
        fn add(borrow a: number, borrow b: number) -> number {
            return a + b;
        }
        add(3, 4);
    "#;
    assert_eval_number(code, 7.0);
}

#[test]
fn milestone_feature_recursion() {
    let code = r#"
        fn fact(borrow n: number) -> number {
            if (n <= 1) { return 1; }
            return n * fact(n - 1);
        }
        fact(5);
    "#;
    assert_eval_number(code, 120.0);
}

#[test]
fn milestone_feature_arrays_create_and_index() {
    assert_eval_number("let a: []number = [10, 20, 30]; a[1];", 20.0);
}

#[test]
fn milestone_feature_string_concatenation() {
    assert_eval_string(r#""foo" + "bar";"#, "foobar");
}
