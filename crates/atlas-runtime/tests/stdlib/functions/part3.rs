use super::*;

// ============================================================================
// 1. Basic assertions — Atlas code integration
// ============================================================================

#[test]
fn test_assert_passes_in_atlas_code() {
    eval_ok("assert(true, \"should pass\");");
}

#[test]
fn test_assert_false_passes_in_atlas_code() {
    eval_ok("assert_false(false, \"should pass\");");
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
        "assert_false(true, \"was unexpectedly true\");",
        "was unexpectedly true",
    );
}

#[test]
fn test_assert_in_function_body() {
    eval_ok(
        r#"
        fn test_basic() -> void {
            assert(true, "should pass");
            assert_false(false, "should also pass");
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
    eval_ok("assert_equal(5, 5);");
}

#[test]
fn test_assert_equal_strings_in_atlas_code() {
    eval_ok(r#"assert_equal("hello", "hello");"#);
}

#[test]
fn test_assert_equal_bools_in_atlas_code() {
    eval_ok("assert_equal(true, true);");
}

#[test]
fn test_assert_equal_failure_shows_diff() {
    let runtime = Atlas::new();
    match runtime.eval("assert_equal(5, 10);") {
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
    eval_ok("assert_not_equal(1, 2);");
}

#[test]
fn test_assert_not_equal_failure() {
    eval_err_contains("assert_not_equal(5, 5);", "equal");
}

// ============================================================================
// 3. Result assertions — Atlas code integration
// ============================================================================

#[test]
fn test_assert_ok_in_atlas_code() {
    eval_ok(
        r#"
        fn divide(borrow a: number, borrow b: number) -> Result<number, string> {
            if (b == 0) { return Err("division by zero"); }
            return Ok(a / b);
        }

        let result = divide(10, 2);
        let value = assert_ok(result);
        assert_equal(value, 5);
    "#,
    );
}

#[test]
fn test_assert_ok_failure_on_err_value() {
    eval_err_contains(
        r#"
        let result = Err("something broke");
        assert_ok(result);
    "#,
        "Err",
    );
}

#[test]
fn test_assert_err_in_atlas_code() {
    eval_ok(
        r#"
        let result = Err("expected failure");
        let err_value = assert_err(result);
        assert_equal(err_value, "expected failure");
    "#,
    );
}

#[test]
fn test_assert_err_failure_on_ok_value() {
    eval_err_contains(
        r#"
        let result = Ok(42);
        assert_err(result);
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
        let value = assert_some(opt);
        assert_equal(value, 42);
    "#,
    );
}

#[test]
fn test_assert_some_failure_on_none() {
    eval_err_contains(
        r#"
        let opt = None();
        assert_some(opt);
    "#,
        "None",
    );
}

#[test]
fn test_assert_none_in_atlas_code() {
    eval_ok(
        r#"
        let opt = None();
        assert_none(opt);
    "#,
    );
}

#[test]
fn test_assert_none_failure_on_some() {
    eval_err_contains(
        r#"
        let opt = Some(99);
        assert_none(opt);
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
        assert_contains(arr, 2);
    "#,
    );
}

#[test]
fn test_assert_contains_failure() {
    eval_err_contains(
        r#"
        let arr = [1, 2, 3];
        assert_contains(arr, 99);
    "#,
        "does not contain",
    );
}

#[test]
fn test_assert_empty_in_atlas_code() {
    eval_ok(
        r#"
        let arr: []number = [];
        assert_empty(arr);
    "#,
    );
}

#[test]
fn test_assert_empty_failure() {
    eval_err_contains(
        r#"
        let arr = [1];
        assert_empty(arr);
    "#,
        "length",
    );
}

#[test]
fn test_assert_length_in_atlas_code() {
    eval_ok(
        r#"
        let arr = [10, 20, 30];
        assert_length(arr, 3);
    "#,
    );
}

#[test]
fn test_assert_length_failure() {
    eval_err_contains(
        r#"
        let arr = [1, 2];
        assert_length(arr, 5);
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
