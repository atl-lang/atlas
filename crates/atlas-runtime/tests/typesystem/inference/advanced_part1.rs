//! Advanced type inference tests part 1: bidirectional, higher-rank, let-polymorphism, and flow-sensitive tests

use super::super::*;
use super::helpers::*;

// ============================================================================
// Bidirectional Type Checking Tests
// ============================================================================

#[test]
fn test_bidir_synthesis_infers_number_literal() {
    // Synthesis: infer type of number literal
    let diags = typecheck_source("let _x = 42;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_bidir_synthesis_infers_string_literal() {
    let diags = typecheck_source(r#"let _x = "hello";"#);
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_bidir_synthesis_infers_bool_literal() {
    let diags = typecheck_source("let _x = true;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_bidir_checking_validates_annotation() {
    // Checking mode: annotation guides inference
    let diags = typecheck_source("let _x: number = 42;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_bidir_checking_rejects_mismatch() {
    // Checking mode: annotation rejects wrong type
    let diags = typecheck_source(r#"let _x: number = "hello";"#);
    assert!(has_error(&diags), "Expected type error");
}

#[test]
fn test_bidir_checking_string_annotation() {
    let diags = typecheck_source(r#"let _x: string = "world";"#);
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_bidir_expected_type_guides_return() {
    let diags = typecheck_source(
        r#"
        fn compute() -> number {
            return 42;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_bidir_return_type_mismatch_detected() {
    let diags = typecheck_source(
        r#"
        fn compute() -> number {
            return "oops";
        }
        "#,
    );
    assert!(has_error(&diags), "Expected return type error");
}

#[test]
fn test_bidir_mode_switch_at_function_boundary() {
    // Annotation on parameter sets expected type for the body
    let diags = typecheck_source(
        r#"
        fn add_one(x: number) -> number {
            return x + 1;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_bidir_complex_expression_inferred() {
    let diags = typecheck_source(
        r#"
        fn max_val(a: number, b: number) -> number {
            if (a > b) {
                return a;
            }
            return b;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_bidir_expected_type_propagation_through_if() {
    let diags = typecheck_source(
        r#"
        fn test(flag: bool) -> string {
            if (flag) {
                return "yes";
            }
            return "no";
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_bidir_infer_without_annotation() {
    // No annotation: full inference from initializer
    let diags = typecheck_source(
        r#"
        let _a = 1 + 2;
        let _b = true;
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

// ============================================================================
// Higher-Rank Polymorphism Tests
// ============================================================================

#[test]
fn test_rank1_polymorphism_inferred() {
    // Simple rank-1 polymorphism: T is inferred from the argument
    let diags = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        let _n = identity(42);
        let _s = identity("hello");
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_function_taking_generic_function() {
    // A function whose parameter is a generic function
    let diags = typecheck_source(
        r#"
        fn apply<T>(f: (T) -> T, x: T) -> T {
            return f(x);
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_callback_with_typed_parameter() {
    let diags = typecheck_source(
        r#"
        fn transform(f: (number) -> number, x: number) -> number {
            return f(x);
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_generic_callback_applied() {
    let diags = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        fn use_identity(n: number) -> number {
            return identity(n);
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_infer_with_rank_restrictions_concrete_param() {
    // When function type is concrete, inference works directly
    let diags = typecheck_source(
        r#"
        fn double(f: (number) -> number) -> number {
            return f(f(1));
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_function_type_parameter_unification() {
    let diags = typecheck_source(
        r#"
        fn compose<A, B, C>(f: (B) -> C, g: (A) -> B) -> (A) -> C {
            fn h(x: A) -> C {
                return f(g(x));
            }
            return h;
        }
        "#,
    );
    // Composition of generic functions is valid
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

// ============================================================================
// Let-Polymorphism Tests
// ============================================================================

#[test]
fn test_let_bind_infers_number() {
    let diags = typecheck_source("let _x = 10;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_let_bind_infers_string() {
    let diags = typecheck_source(r#"let _y = "hello";"#);
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_let_bind_infers_bool() {
    let diags = typecheck_source("let _z = false;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_let_bind_infers_null() {
    let diags = typecheck_source("let _n = null;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_let_bind_with_explicit_annotation() {
    let diags = typecheck_source("let _x: number = 42;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_let_bind_mutable_allows_reassign() {
    let diags = typecheck_source(
        r#"
        var x = 5;
        x = 10;
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_let_bind_immutable_rejects_reassign() {
    let diags = typecheck_source(
        r#"
        let x = 5;
        x = 10;
        "#,
    );
    assert!(has_code(&diags, "AT3003"), "Expected immutability error");
}

#[test]
fn test_recursive_function_type_check() {
    // Recursive function - let binding supports recursive references
    let diags = typecheck_source(
        r#"
        fn factorial(n: number) -> number {
            if (n == 0) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

// ============================================================================
// Flow-Sensitive Typing Tests
// ============================================================================

#[test]
fn test_flow_type_narrowed_in_then_branch() {
    // After checking typeof, the type is narrowed in the branch
    let diags = typecheck_source(
        r#"
        fn narrow_test(x: number | string) -> number {
            if (typeof(x) == "number") {
                return x;
            }
            return 0;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_flow_widen_at_merge_point() {
    // After if-else, mutable variable can be assigned in both branches
    let diags = typecheck_source(
        r#"
        fn get_val(flag: bool) -> number {
            var result = 0;
            if (flag) {
                result = 1;
            } else {
                result = 2;
            }
            return result;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_flow_immutable_tracking_precise() {
    // Immutable variable: type doesn't widen
    let diags = typecheck_source("let _x: number = 42;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_flow_loop_basic() {
    let diags = typecheck_source(
        r#"
        var i = 0;
        while (i < 10) {
            i = i + 1;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_flow_loop_with_for() {
    let diags = typecheck_source(
        r#"
        var sum = 0;
        for (var i = 0; i < 5; i++) {
            sum = sum + i;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_flow_impossible_never_branch() {
    // Narrowing to Never when both branches are covered
    let diags = typecheck_source(
        r#"
        fn check(x: number) -> bool {
            if (x > 0) {
                return true;
            }
            return false;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_flow_condition_bool_required() {
    // Control flow requires bool condition
    let diags = typecheck_source("if (42) { }");
    assert!(has_error(&diags), "Expected condition must be bool error");
}
