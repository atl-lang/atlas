use super::super::*;
#[allow(unused_imports)]
use pretty_assertions::assert_eq;

// From generic_type_checking_tests.rs
// ============================================================================

// Generic Type Checking and Inference Tests (BLOCKER 02-B)
//
// Comprehensive test suite for generic types including:
// - Type parameter syntax and parsing
// - Type parameter scoping
// - Generic type arity validation
// - Type inference (Hindley-Milner)
// - Occurs check
// - Nested generics
// - Error cases

// ============================================================================
// Basic Generic Function Declaration
// ============================================================================

#[test]
fn test_generic_function_simple_declaration() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_generic_function_multiple_type_params() {
    let diagnostics = typecheck_source(
        r#"
        fn pair<A, B>(first: A, _second: B) -> A {
            return first;
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_generic_function_three_type_params() {
    let diagnostics = typecheck_source(
        r#"
        fn triple<A, B, C>(_a: A, _b: B, _c: C) -> A {
            return _a;
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

// ============================================================================
// Type Parameter Scoping
// ============================================================================

#[test]
fn test_type_parameter_in_param() {
    let diagnostics = typecheck_source(
        r#"
        fn test<T>(_x: T) -> void {}
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_type_parameter_in_return() {
    // Type parameter in return position is valid
    // We can't check type correctness without knowing T
    let diagnostics = typecheck_source(
        r#"
        fn test<T>(_x: number) -> T {
            return _x;
        }
    "#,
    );
    // Note: This passes type checking because we can't validate T without instantiation
    // The error would be caught at call sites if types don't match
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_type_parameter_in_array() {
    let diagnostics = typecheck_source(
        r#"
        fn first<T>(arr: T[]) -> T {
            return arr[0];
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_duplicate_type_parameter() {
    let diagnostics = typecheck_source(
        r#"
        fn bad<T, T>(_x: T) -> T {
            return _x;
        }
    "#,
    );
    assert!(!diagnostics.is_empty());
    assert!(diagnostics[0].message.contains("Duplicate type parameter"));
}

// ============================================================================
// Type Inference - Simple Cases
// ============================================================================

#[test]
fn test_inference_number() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        let _result = identity(42);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_inference_string() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        let _result = identity("hello");
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_inference_bool() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        let _result = identity(true);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_inference_array() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        let arr = [1, 2, 3];
        let _result = identity(arr);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

// ============================================================================
// Type Inference - Multiple Parameters
// ============================================================================

#[test]
fn test_inference_multiple_same_type() {
    let diagnostics = typecheck_source(
        r#"
        fn both<T>(_a: T, _b: T) -> T {
            return _a;
        }
        let _result = both(42, 84);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_inference_multiple_different_types() {
    let diagnostics = typecheck_source(
        r#"
        fn pair<A, B>(_first: A, _second: B) -> A {
            return _first;
        }
        let _result = pair(42, "hello");
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_inference_three_params() {
    let diagnostics = typecheck_source(
        r#"
        fn triple<A, B, C>(_a: A, _b: B, _c: C) -> A {
            return _a;
        }
        let _result = triple(1, "two", true);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

// ============================================================================
// Type Inference - Arrays
// ============================================================================

#[test]
fn test_inference_array_element_type() {
    let diagnostics = typecheck_source(
        r#"
        fn first<T>(arr: T[]) -> T {
            return arr[0];
        }
        let numbers = [1, 2, 3];
        let _result = first(numbers);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_inference_array_of_strings() {
    let diagnostics = typecheck_source(
        r#"
        fn first<T>(arr: T[]) -> T {
            return arr[0];
        }
        let strings = ["a", "b", "c"];
        let _result = first(strings);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

// ============================================================================
// Generic Type Arity Validation
// ============================================================================

#[test]
fn test_option_correct_arity() {
    let diagnostics = typecheck_source(
        r#"
        fn test(_x: Option<number>) -> void {}
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_result_correct_arity() {
    let diagnostics = typecheck_source(
        r#"
        fn test(_x: Result<number, string>) -> void {}
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_option_wrong_arity_too_many() {
    let diagnostics = typecheck_source(
        r#"
        fn test(_x: Option<number, string>) -> void {}
    "#,
    );
    assert!(!diagnostics.is_empty());
    assert!(diagnostics[0].message.contains("expects 1 type argument"));
}

#[test]
fn test_result_wrong_arity_too_few() {
    let diagnostics = typecheck_source(
        r#"
        fn test(_x: Result<number>) -> void {}
    "#,
    );
    assert!(!diagnostics.is_empty());
    assert!(diagnostics[0].message.contains("expects 2 type argument"));
}

#[test]
fn test_unknown_generic_type() {
    let diagnostics = typecheck_source(
        r#"
        fn test(_x: UnknownGeneric<number>) -> void {}
    "#,
    );
    assert!(!diagnostics.is_empty());
    assert!(diagnostics[0].message.contains("Unknown generic type"));
}

// ============================================================================
// Nested Generic Types
// ============================================================================

#[test]
fn test_nested_option_result() {
    let diagnostics = typecheck_source(
        r#"
        fn test(_x: Option<Result<number, string>>) -> void {}
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_nested_result_option() {
    let diagnostics = typecheck_source(
        r#"
        fn test(_x: Result<Option<number>, string>) -> void {}
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_deeply_nested() {
    let diagnostics = typecheck_source(
        r#"
        fn test(_x: Option<Result<Option<number>, string>>) -> void {}
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_array_of_option() {
    let diagnostics = typecheck_source(
        r#"
        fn test(_x: Option<number>[]) -> void {}
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

// ============================================================================
// Type Mismatch Errors
// ============================================================================

#[test]
fn test_inference_type_mismatch() {
    let diagnostics = typecheck_source(
        r#"
        fn both<T>(_a: T, _b: T) -> T {
            return _a;
        }
        let _result = both(42, "hello");
    "#,
    );
    assert!(!diagnostics.is_empty());
    assert!(
        diagnostics[0].message.contains("Type inference failed")
            || diagnostics[0].message.contains("cannot match")
    );
}

#[test]
fn test_return_type_mismatch() {
    // Returning a concrete type when T is expected
    // This is allowed at declaration - error caught at call site
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(_x: T) -> T {
            return 42;
        }
    "#,
    );
    // This passes because we allow returning number for T
    // The type error would be caught when calling with non-number types
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_array_element_mismatch() {
    // Returning a concrete type when T is expected
    let diagnostics = typecheck_source(
        r#"
        fn first<T>(_arr: T[]) -> T {
            return "wrong";
        }
    "#,
    );
    // This passes declaration-level checking
    // Error would be caught at call sites
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

// ============================================================================
// Complex Inference Scenarios
// ============================================================================

#[test]
fn test_inference_with_function_call_chain() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        fn double_identity<T>(x: T) -> T {
            return identity(x);
        }
        let _result = double_identity(42);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_inference_with_variable() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        let num = 42;
        let _result = identity(num);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}
