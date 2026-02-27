//! Return type inference tests part 1 (sections 1-9)

use super::super::*;
use super::helpers::*;

// ============================================================================
// 1. Return type inference - uniform returns
// ============================================================================
#[test]
fn test_infer_return_number() {
    let diags = errors(
        r#"
        fn double(x: number) -> number { return x * 2; }
        let _r = double(5);
    "#,
    );
    assert!(
        diags.is_empty(),
        "Valid return should have no errors: {:?}",
        diags
    );
}

#[test]
fn test_infer_return_string() {
    let diags = errors(
        r#"
        fn greet(name: string) -> string { return "hello " + name; }
        let _r = greet("world");
    "#,
    );
    assert!(diags.is_empty(), "Valid string return: {:?}", diags);
}

#[test]
fn test_infer_return_bool() {
    let diags = errors(
        r#"
        fn is_positive(x: number) -> bool { return x > 0; }
        let _r = is_positive(5);
    "#,
    );
    assert!(diags.is_empty(), "Valid bool return: {:?}", diags);
}

#[test]
fn test_infer_return_void() {
    let diags = errors(
        r#"
        fn do_nothing() -> void { }
        do_nothing();
    "#,
    );
    assert!(diags.is_empty(), "Void function: {:?}", diags);
}

// ============================================================================
// 2. Return type mismatch detection
// ============================================================================

#[test]
fn test_return_number_expected_string() {
    let diags = errors(
        r#"
        fn foo() -> string { return 42; }
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected string"));
    assert!(diags[0].message.contains("found number"));
}

#[test]
fn test_return_string_expected_number() {
    let diags = errors(
        r#"
        fn foo() -> number { return "hello"; }
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected number"));
}

#[test]
fn test_return_bool_expected_string() {
    let diags = errors(
        r#"
        fn foo() -> string { return true; }
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected string"));
}

// ============================================================================
// 3. Bidirectional: variable type annotation guides inference
// ============================================================================

#[test]
fn test_bidi_number_annotation_valid() {
    let diags = errors("let _x: number = 42;");
    assert!(diags.is_empty());
}

#[test]
fn test_bidi_string_annotation_valid() {
    let diags = errors(r#"let _x: string = "hello";"#);
    assert!(diags.is_empty());
}

#[test]
fn test_bidi_bool_annotation_valid() {
    let diags = errors("let _x: bool = true;");
    assert!(diags.is_empty());
}

#[test]
fn test_bidi_number_annotation_mismatch() {
    let diags = errors(r#"let _x: number = "hello";"#);
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected number"));
    assert!(diags[0].message.contains("found string"));
}

#[test]
fn test_bidi_string_annotation_mismatch() {
    let diags = errors("let _x: string = true;");
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected string"));
}

// ============================================================================
// 4. Expression type inference
// ============================================================================

#[test]
fn test_infer_arithmetic_result() {
    let diags = errors("let _x: number = 1 + 2;");
    assert!(diags.is_empty());
}

#[test]
fn test_infer_comparison_result() {
    let diags = errors("let _x: bool = 1 > 2;");
    assert!(diags.is_empty());
}

#[test]
fn test_infer_logical_result() {
    let diags = errors("let _x: bool = (1 > 0) && (2 > 1);");
    assert!(diags.is_empty(), "Logical result: {:?}", diags);
}

#[test]
fn test_infer_negation_result() {
    let diags = errors("let _x: number = -42;");
    assert!(diags.is_empty());
}

#[test]
fn test_infer_not_result() {
    let diags = errors("let _x: bool = !true;");
    assert!(diags.is_empty());
}

#[test]
fn test_infer_string_concat_result() {
    let diags = errors(r#"let _x: string = "a" + "b";"#);
    assert!(diags.is_empty());
}

// ============================================================================
// 5. Array type inference
// ============================================================================

#[test]
fn test_infer_number_array() {
    let diags = errors(
        r#"
        let arr = [1, 2, 3];
        let _x: number = arr[0];
    "#,
    );
    assert!(diags.is_empty(), "Number array indexing: {:?}", diags);
}

#[test]
fn test_infer_string_array() {
    let diags = errors(
        r#"
        let arr = ["a", "b", "c"];
        let _x: string = arr[0];
    "#,
    );
    assert!(diags.is_empty(), "String array indexing: {:?}", diags);
}

#[test]
fn test_array_assigned_to_wrong_type() {
    let diags = errors(
        r#"
        let arr = [1, 2, 3];
        let _x: string = arr;
    "#,
    );
    assert!(!diags.is_empty());
}

// ============================================================================
// 6. Function call return type inference
// ============================================================================

#[test]
fn test_infer_function_call_return() {
    let diags = errors(
        r#"
        fn add(a: number, b: number) -> number { return a + b; }
        let _x: number = add(1, 2);
    "#,
    );
    assert!(diags.is_empty(), "Function call return type: {:?}", diags);
}

#[test]
fn test_function_call_return_mismatch() {
    let diags = errors(
        r#"
        fn add(a: number, b: number) -> number { return a + b; }
        let _x: string = add(1, 2);
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected string"));
    assert!(diags[0].message.contains("found number"));
}

// ============================================================================
// 7. Nested expression inference
// ============================================================================

#[test]
fn test_nested_arithmetic() {
    let diags = errors("let _x: number = (1 + 2) * 3;");
    assert!(diags.is_empty());
}

#[test]
fn test_nested_comparison() {
    let diags = errors("let _x: bool = (1 + 2) > 3;");
    assert!(diags.is_empty());
}

#[test]
fn test_nested_logical() {
    let diags = errors("let _x: bool = (1 > 0) && (2 > 1) || (3 > 2);");
    assert!(diags.is_empty(), "Nested logical: {:?}", diags);
}

// ============================================================================
// 8. Variable usage inference
// ============================================================================

#[test]
fn test_var_inferred_number() {
    let diags = errors(
        r#"
        let x = 42;
        let _y: number = x;
    "#,
    );
    assert!(diags.is_empty(), "Inferred number variable: {:?}", diags);
}

#[test]
fn test_var_inferred_string() {
    let diags = errors(
        r#"
        let x = "hello";
        let _y: string = x;
    "#,
    );
    assert!(diags.is_empty(), "Inferred string variable: {:?}", diags);
}

#[test]
fn test_var_inferred_bool() {
    let diags = errors(
        r#"
        let x = true;
        let _y: bool = x;
    "#,
    );
    assert!(diags.is_empty(), "Inferred bool variable: {:?}", diags);
}

// ============================================================================
// 9. Mutable variable type tracking
// ============================================================================

#[test]
fn test_var_mutable_same_type() {
    let diags = errors(
        r#"
        var x = 1;
        x = 2;
        x = 3;
    "#,
    );
    assert!(diags.is_empty(), "Same-type mutation: {:?}", diags);
}

#[test]
fn test_var_mutable_wrong_type() {
    let diags = errors(
        r#"
        var x: number = 1;
        x = "hello";
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected number"));
}
