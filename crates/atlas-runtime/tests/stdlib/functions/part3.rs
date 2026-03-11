use super::*;

// ============================================================================
// 1. Basic assertions — test.* namespace (B34)
// ============================================================================

#[test]
fn test_assert_passes_in_atlas_code() {
    eval_ok("test.assert(true);");
}

#[test]
fn test_assert_with_message_passes() {
    eval_ok("test.assert(true, \"should pass\");");
}

#[test]
fn test_assert_failure_produces_error() {
    eval_err_contains(
        "test.assert(false, \"my custom failure message\");",
        "my custom failure message",
    );
}

#[test]
fn test_assert_failure_no_message() {
    eval_err_contains("test.assert(false);", "assertion failed");
}

#[test]
fn test_assert_in_function_body() {
    eval_ok(
        r#"
        fn test_basic(): void {
            test.assert(true, "should pass");
            test.assert(1 == 1, "math works");
        }
        test_basic();
    "#,
    );
}

// ============================================================================
// 2. Equality assertions — test.* namespace
// ============================================================================

#[test]
fn test_equal_numbers_in_atlas_code() {
    eval_ok("test.equal(5, 5);");
}

#[test]
fn test_equal_strings_in_atlas_code() {
    eval_ok(r#"test.equal("hello", "hello");"#);
}

#[test]
fn test_equal_bools_in_atlas_code() {
    eval_ok("test.equal(true, true);");
}

#[test]
fn test_equal_failure_shows_diff() {
    let runtime = Atlas::new();
    match runtime.eval("test.equal(5, 10);") {
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
fn test_not_equal_in_atlas_code() {
    eval_ok("test.notEqual(1, 2);");
}

#[test]
fn test_not_equal_failure() {
    eval_err_contains("test.notEqual(5, 5);", "equal");
}

// ============================================================================
// 3. Result assertions — test.* namespace
// ============================================================================

#[test]
fn test_ok_in_atlas_code() {
    eval_ok(
        r#"
        fn divide(borrow a: number, borrow b: number): Result<number, string> {
            if (b == 0) { return Err("division by zero"); }
            return Ok(a / b);
        }

        let result = divide(10, 2);
        let value = test.ok(result);
        test.equal(value, 5);
    "#,
    );
}

#[test]
fn test_ok_failure_on_err_value() {
    eval_err_contains(
        r#"
        let result = Err("something broke");
        test.ok(result);
    "#,
        "Err",
    );
}

#[test]
fn test_err_in_atlas_code() {
    eval_ok(
        r#"
        let result = Err("expected failure");
        let err_value = test.err(result);
        test.equal(err_value, "expected failure");
    "#,
    );
}

#[test]
fn test_err_failure_on_ok_value() {
    eval_err_contains(
        r#"
        let result = Ok(42);
        test.err(result);
    "#,
        "Ok",
    );
}

// ============================================================================
// 4. Collection assertions — test.* namespace
// ============================================================================

#[test]
fn test_contains_in_atlas_code() {
    eval_ok(
        r#"
        let arr = [1, 2, 3];
        test.contains(arr, 2);
    "#,
    );
}

#[test]
fn test_contains_failure() {
    eval_err_contains(
        r#"
        let arr = [1, 2, 3];
        test.contains(arr, 99);
    "#,
        "does not contain",
    );
}

#[test]
fn test_empty_in_atlas_code() {
    eval_ok(
        r#"
        let arr: number[] = [];
        test.empty(arr);
    "#,
    );
}

#[test]
fn test_empty_failure() {
    eval_err_contains(
        r#"
        let arr = [1];
        test.empty(arr);
    "#,
        "length",
    );
}

// ============================================================================
// 5. Approx assertion — test.approx
// ============================================================================

#[test]
fn test_approx_in_atlas_code() {
    eval_ok("test.approx(1.0, 1.001, 0.01);");
}

#[test]
fn test_approx_failure() {
    eval_err_contains("test.approx(1.0, 2.0, 0.5);", "epsilon");
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
