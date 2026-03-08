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
        fn fibonacci(borrow n: number) -> number {
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
        fn identity<T>(borrow x: T) -> T {
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
        fn greet(borrow name: string) -> string {
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
        fn outer(borrow x: number) -> number {
            fn inner(borrow y: number) -> number {
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
        fn first<T>(borrow arr: []T) -> T {
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
        fn double(borrow x: number) -> number {
            return x * 2;
        }
        fn add_one(borrow x: number) -> number {
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
        fn max_num<T extends Comparable>(borrow a: T, borrow b: T) -> T {
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
        fn accepts_option(borrow _x: Option<number>) -> void {}
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_result_type_usage() {
    // Result<number, string> should be recognized as a valid generic type
    let diags = typecheck_source(
        r#"
        fn accepts_result(borrow _x: Result<number, string>) -> void {}
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_for_in_with_inferred_element_type() {
    let diags = typecheck_source(
        r#"
        fn sum_array(borrow nums: []number) -> number {
            let mut total = 0;
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
        fn always_true(borrow _x: number) -> bool {
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
        fn show_value(borrow x: number | string) -> string {
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
        fn accepts_point(borrow _point: { x: number, borrow y: number }) -> void {}
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_anonymous_struct_literal_inference() {
    let diags = typecheck_source(
        r#"
        let x = 1;
        let y = "hi";
        let _point: { x: number, y: string } = { x, y };
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_anonymous_struct_literal_return_type() {
    let diags = typecheck_source(
        r#"
        fn get() -> { id: number } {
            return { id: 42 };
        }
        let _value = get();
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_deeply_nested_generics() {
    let diags = typecheck_source(
        r#"
        fn nested(borrow _x: Option<Result<number, string>>) -> void {}
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

// ============================================================================

// ============================================================================
// H-162: empty array literal in struct field accepts declared field type
// ============================================================================

#[test]
fn test_h162_empty_array_in_struct_field_typed() {
    // args: [] where field is []string — should not produce AT3001
    let diags = typecheck_source(
        r#"
        struct ServerConfig {
            command: string,
            args: []string,
            port: number,
        }
        let cfg = ServerConfig {
            command: "node",
            args: [],
            port: 3000,
        };
        "#,
    );
    assert!(
        !has_error(&diags),
        "empty array in typed struct field should be valid. Errors: {:?}",
        diags
    );
}

#[test]
fn test_h162_empty_array_in_nested_struct_field() {
    // Nested struct with multiple empty array fields
    let diags = typecheck_source(
        r#"
        struct WatchConfig {
            paths: []string,
            extensions: []string,
            ignore: []string,
            enabled: bool,
        }
        let w = WatchConfig {
            paths: [],
            extensions: [],
            ignore: [],
            enabled: true,
        };
        "#,
    );
    assert!(
        !has_error(&diags),
        "multiple empty array fields should all be valid. Errors: {:?}",
        diags
    );
}
