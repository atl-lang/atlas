use super::*;
use pretty_assertions::assert_eq;

fn test_array_method_push_inferred_type() {
    assert_eval_number(r#"let arr = [1, 2, 3]; arr.push(4); arr[3];"#, 4.0);
}

/// Parity: interpreter and VM produce same result for push
#[test]
fn test_array_method_push_parity_via_atlas_eval() {
    let code = r#"var arr: array = [1, 2, 3]; arr.push(99); arr[3];"#;
    assert_eval_number(code, 99.0);
}

// ============================================================================
// Value semantics regression tests — CoW behavior must never regress
// ============================================================================

/// Regression: assignment creates independent copy; mutation of source does not
/// affect the copy (CoW value semantics).
#[test]
fn test_value_semantics_regression_assign_copy() {
    let code = r#"
        let a: number[] = [1, 2, 3];
        let b: number[] = a;
        a[0] = 99;
        b[0]
    "#;
    assert_eval_number(code, 1.0);
}

/// Regression: mutation of assigned copy does not affect source.
#[test]
fn test_value_semantics_regression_copy_mutation_isolated() {
    let code = r#"
        let a: number[] = [1, 2, 3];
        let b: number[] = a;
        b[0] = 42;
        a[0]
    "#;
    assert_eval_number(code, 1.0);
}

/// Regression: push on assigned copy does not grow the source.
#[test]
fn test_value_semantics_regression_push_copy_isolated() {
    let code = r#"
        var a: array = [1, 2, 3];
        var b: array = a;
        b.push(4);
        len(a)
    "#;
    assert_eval_number(code, 3.0);
}

/// Regression: function parameter is an independent copy — mutations stay local.
#[test]
fn test_value_semantics_regression_fn_param_copy() {
    let code = r#"
        fn fill(arr: number[]) -> void {
            arr[0] = 999;
        }
        let nums: number[] = [1, 2, 3];
        fill(nums);
        nums[0]
    "#;
    assert_eval_number(code, 1.0);
}

/// Regression: three-way copy — each variable is independent.
#[test]
fn test_value_semantics_regression_three_way_copy() {
    let code = r#"
        let a: number[] = [1, 2, 3];
        let b: number[] = a;
        let c: number[] = b;
        b[0] = 10;
        c[1] = 20;
        a[0] + a[1]
    "#;
    assert_eval_number(code, 3.0);
}

// ============================================================================
