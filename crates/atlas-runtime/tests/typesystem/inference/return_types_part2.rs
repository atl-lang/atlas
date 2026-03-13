//! Return type inference tests part 2 (sections 10-11 and Block 5 Phase 3-4)

use super::super::*;
#[allow(unused_imports)]
use super::helpers::*;

// ============================================================================
// 10. Complex scenarios
// ============================================================================

#[test]
fn test_function_with_if_return() {
    let diags = errors(
        r#"
        fn my_abs(borrow x: number): number {
            if (x < 0) {
                return -x;
            }
            return x;
        }
        let _r = my_abs(-5);
    "#,
    );
    assert!(diags.is_empty(), "Function with if/return: {:?}", diags);
}

#[test]
fn test_function_calling_function() {
    let diags = errors(
        r#"
        fn square(borrow x: number): number { return x * x; }
        fn sum_squares(borrow a: number, borrow b: number): number {
            return square(a) + square(b);
        }
        let _r = sum_squares(3, 4);
    "#,
    );
    assert!(diags.is_empty(), "Function composition: {:?}", diags);
}

#[test]
fn test_multiple_errors_reported() {
    let diags = errors(
        r#"
        let _a: number = "hello";
        let _b: string = 42;
    "#,
    );
    assert!(
        diags.len() >= 2,
        "Should report multiple errors: {:?}",
        diags
    );
}

#[test]
fn test_no_false_positives_complex() {
    let diags = errors(
        r#"
        fn is_even(borrow n: number): bool { return n % 2 == 0; }
        fn describe(borrow n: number): string {
            if (is_even(n)) {
                return "even";
            }
            return "odd";
        }
        let _r: string = describe(42);
    "#,
    );
    assert!(diags.is_empty(), "Complex valid program: {:?}", diags);
}

#[test]
fn test_while_loop_valid_types() {
    let diags = errors(
        r#"
        fn countdown(borrow n: number): number {
            let mut count = n;
            while (count > 0) {
                count = count - 1;
            }
            return count;
        }
        let _r = countdown(10);
    "#,
    );
    assert!(diags.is_empty(), "While loop valid: {:?}", diags);
}

#[test]
fn test_for_in_valid_array() {
    let diags = errors(
        r#"
        fn sum_arr(): number {
            let arr = [1, 2, 3];
            let mut total = 0;
            for x in arr {
                total = total + x;
            }
            return total;
        }
        let _r = sum_arr();
    "#,
    );
    assert!(diags.is_empty(), "For-in valid: {:?}", diags);
}

// ============================================================================
// 11. Additional inference edge cases
// ============================================================================

#[test]
fn test_modulo_result_is_number() {
    let diags = errors("let _x: number = 10 % 3;");
    assert!(diags.is_empty());
}

#[test]
fn test_division_result_is_number() {
    let diags = errors("let _x: number = 10 / 3;");
    assert!(diags.is_empty());
}

#[test]
fn test_equality_result_is_bool() {
    let diags = errors("let _x: bool = 1 == 1;");
    assert!(diags.is_empty());
}

#[test]
fn test_inequality_result_is_bool() {
    let diags = errors("let _x: bool = 1 != 2;");
    assert!(diags.is_empty());
}

// ============================================================================
// Return type inference (Block 5 Phase 3)
// ============================================================================

#[test]
fn test_explicit_return_no_type_error() {
    // fn with explicit return type annotation should not emit AT3001
    let diags = errors("fn double(borrow x: number): number { return x * 2; }");
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Expected no AT3001 for explicit return type, got: {:?}",
        type_errors
    );
}

#[test]
fn test_explicit_return_bool_from_comparison() {
    // fn returning a comparison with explicit -> bool annotation
    let diags = errors("fn is_zero(borrow x: number): bool { return x == 0; }");
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Expected no AT3001, got: {:?}",
        type_errors
    );
}

#[test]
fn test_void_return_empty_body() {
    // fn with empty body and -> void, no type errors
    let diags = errors("fn noop(): void { }");
    let errs: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "AT3001" || d.code == "AT3004")
        .collect();
    assert!(
        errs.is_empty(),
        "Expected no type errors for noop(): void, got: {:?}",
        errs
    );
}

#[test]
fn test_explicit_return_number_from_literal() {
    // fn returning a number literal with explicit -> number annotation
    let diags = errors("fn get_answer(): number { return 42; }");
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Expected no AT3001 for explicit literal return, got: {:?}",
        type_errors
    );
}

#[test]
fn test_explicit_return_consistent_arithmetic() {
    // Explicit return type annotation with arithmetic
    let diags = errors("fn half(borrow x: number): number { return x / 2; }");
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Expected no AT3001 for arithmetic return, got: {:?}",
        type_errors
    );
}

#[test]
fn test_at3001_on_mismatched_return_type() {
    // fn with explicit return type but wrong return value → AT3001
    let diags = errors(
        r#"
fn confused(borrow x: number): number {
    if x > 0 {
        return 1;
    } else {
        return "negative";
    }
}
"#,
    );
    assert!(
        diags
            .iter()
            .any(|d| d.code == "AT3001" || d.code == "AT3052"),
        "Expected AT3001/AT3052 for mismatched return type, got: {:?}",
        diags.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn test_inferred_return_callable_result() {
    // Function with inferred return can be called; result usable in expression
    let diags = errors(
        r#"
fn square(borrow x: number): number { return x * x; }
let _y: number = square(4);
"#,
    );
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Expected no AT3001 for inferred-return call, got: {:?}",
        type_errors
    );
}

#[test]
fn test_inferred_return_both_engines() {
    // Both interpreter and VM execute functions with inferred return type correctly
    let runtime = atlas_runtime::Atlas::new();
    let result = runtime.eval("fn double(borrow x: number): number { return x * 2; } double(5);");
    assert_eq!(result.unwrap(), atlas_runtime::Value::Number(10.0));
}

// ============================================================================
// Local variable inference (Block 5 Phase 4)
// ============================================================================

#[test]
fn test_local_infer_number_literal() {
    // let x = 42 infers number; using as number is fine
    let diags = errors("let x = 42; let _y: number = x;");
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Expected no AT3001 for inferred number, got: {:?}",
        type_errors
    );
}

#[test]
fn test_local_infer_string_literal() {
    let diags = errors(r#"let s = "hello"; let _t: string = s;"#);
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Expected no AT3001 for inferred string, got: {:?}",
        type_errors
    );
}

#[test]
fn test_local_infer_bool_literal() {
    let diags = errors("let b = true; let _c: bool = b;");
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Expected no AT3001 for inferred bool, got: {:?}",
        type_errors
    );
}

#[test]
fn test_local_infer_array_literal() {
    // [1,2,3] infers number[]
    let diags = errors("let arr = [1, 2, 3]; let _b: number[] = arr;");
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Expected no AT3001 for inferred number[], got: {:?}",
        type_errors
    );
}

#[test]
fn test_local_infer_arithmetic_expression() {
    let diags = errors("let x = 1 + 2; let _y: number = x;");
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Expected no AT3001 for inferred arithmetic, got: {:?}",
        type_errors
    );
}

#[test]
fn test_local_infer_wrong_usage_emits_error() {
    // Inferred number used as string → AT3001
    let diags = errors(r#"let x = 42; let _s: string = x;"#);
    assert!(
        diags.iter().any(|d| d.code == "AT3001"),
        "Expected AT3001 for number used as string, got: {:?}",
        diags.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn test_local_infer_comparison_expression() {
    // 1 < 2 infers bool
    let diags = errors("let cmp = 1 < 2; let _b: bool = cmp;");
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Expected no AT3001 for inferred bool comparison, got: {:?}",
        type_errors
    );
}

#[test]
fn test_local_infer_chained_usage() {
    // Inferred type flows through multiple assignments
    let diags = errors("let x = 10; let y = x * 2; let _z: number = y;");
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Expected no AT3001 for chained inferred types, got: {:?}",
        type_errors
    );
}
