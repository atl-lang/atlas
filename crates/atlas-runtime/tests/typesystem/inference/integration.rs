//! Advanced type inference integration tests

use super::super::*;
#[allow(unused_imports)]
use super::helpers::*;

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_integration_complex_program_no_annotations() {
    // Complex program with minimal annotations
    let diags = typecheck_source(
        r#"
        fn fibonacci(n: number) -> number {
            if (n <= 1) {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
        let _result = fibonacci(10);
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_generic_identity_minimal_annotations() {
    let diags = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        let _a = identity(42);
        let _b = identity("hello");
        let _c = identity(true);
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_real_world_string_processing() {
    let diags = typecheck_source(
        r#"
        fn greet(name: string) -> string {
            return "Hello, " + name + "!";
        }
        let _message = greet("World");
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_nested_function_inference() {
    let diags = typecheck_source(
        r#"
        fn outer(x: number) -> number {
            fn inner(y: number) -> number {
                return y * 2;
            }
            return inner(x) + 1;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_type_checking_across_variables() {
    let diags = typecheck_source(
        r#"
        let a = 10;
        let b = 20;
        let _sum: number = a + b;
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_array_operations() {
    let diags = typecheck_source(
        r#"
        fn first<T>(arr: T[]) -> T {
            return arr[0];
        }
        let nums = [1, 2, 3];
        let _n = first(nums);
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_multiple_functions_call_chain() {
    let diags = typecheck_source(
        r#"
        fn double(x: number) -> number {
            return x * 2;
        }
        fn add_one(x: number) -> number {
            return x + 1;
        }
        let _result = add_one(double(5));
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_generic_with_constraint() {
    let diags = typecheck_source(
        r#"
        fn max_num<T extends Comparable>(a: T, b: T) -> T {
            if (a > b) {
                return a;
            }
            return b;
        }
        let _m = max_num(3, 7);
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_option_type_usage() {
    // Option<number> should be recognized as a valid generic type
    let diags = typecheck_source(
        r#"
        fn accepts_option(_x: Option<number>) -> void {}
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_result_type_usage() {
    // Result<number, string> should be recognized as a valid generic type
    let diags = typecheck_source(
        r#"
        fn accepts_result(_x: Result<number, string>) -> void {}
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_for_in_with_inferred_element_type() {
    let diags = typecheck_source(
        r#"
        fn sum_array(nums: number[]) -> number {
            var total = 0;
            for n in nums {
                total = total + n;
            }
            return total;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_type_alias_in_function() {
    let diags = typecheck_source(
        r#"
        type Predicate<T> = (T) -> bool;
        fn always_true(_x: number) -> bool {
            return true;
        }
        let _pred: Predicate<number> = always_true;
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_union_type_function_params() {
    let diags = typecheck_source(
        r#"
        fn show_value(x: number | string) -> string {
            if (typeof(x) == "number") {
                return "it is a number";
            }
            return "it is a string";
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_structural_type_inference() {
    // Structural types accepted as function parameters
    let diags = typecheck_source(
        r#"
        fn accepts_point(_point: { x: number, y: number }) -> void {}
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_deeply_nested_generics() {
    let diags = typecheck_source(
        r#"
        fn nested(_x: Option<Result<number, string>>) -> void {}
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

// ============================================================================