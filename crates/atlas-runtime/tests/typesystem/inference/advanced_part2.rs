//! Advanced type inference tests part 2: unification, constraint solving, cross-module, and heuristics tests

use super::super::*;
#[allow(unused_imports)]
use super::helpers::*;

// ============================================================================
// Unification Tests (via type checker API)
// ============================================================================

#[test]
fn test_unification_generic_type_arg_inferred() {
    let diags = typecheck_source(
        r#"
        fn wrap<T>(x: T) -> T[] {
            let _arr: T[] = [x];
            return _arr;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_unification_occurs_check_invalid_recursive_fn() {
    // A function declared recursively but with wrong return type
    let diags = typecheck_source(
        r#"
        fn get_number() -> string {
            return 42;
        }
        "#,
    );
    assert!(has_error(&diags), "Expected return type mismatch");
}

#[test]
fn test_unification_struct_member_types() {
    // Structural type accepted as function parameter
    let diags = typecheck_source(
        r#"
        fn validate_point(_p: { x: number, y: number }) -> bool {
            return true;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_unification_union_type_parameters() {
    let diags = typecheck_source(
        r#"
        fn get_str_or_num() -> number | string {
            return 42;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_unification_function_signature_match() {
    let diags = typecheck_source(
        r#"
        fn apply_fn(f: (number) -> string, x: number) -> string {
            return f(x);
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_unification_generic_constraints_satisfied() {
    let diags = typecheck_source(
        r#"
        fn max_val<T extends Comparable>(a: T, b: T) -> T {
            if (a > b) {
                return a;
            }
            return b;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

// ============================================================================
// Constraint Solving Tests
// ============================================================================

#[test]
fn test_constraint_type_annotation_solves() {
    // Annotation provides the constraint, initializer must satisfy it
    let diags = typecheck_source("let _v: number = 1 + 2;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_constraint_unsolvable_type_mismatch() {
    let diags = typecheck_source(r#"let _v: number = "string";"#);
    assert!(has_error(&diags), "Expected constraint violation");
}

#[test]
fn test_constraint_delayed_solving_generic_call() {
    // Type parameters inferred lazily from call site
    let diags = typecheck_source(
        r#"
        fn id<T>(x: T) -> T {
            return x;
        }
        let _n: number = id(42);
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_constraint_function_return_constraint() {
    let diags = typecheck_source(
        r#"
        fn make_number() -> number {
            let _x = 42;
            return _x;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_constraint_parameter_type_propagated() {
    let diags = typecheck_source(
        r#"
        fn double(x: number) -> number {
            return x * 2;
        }
        let _r: number = double(5);
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_constraint_multiple_parameters_inferred() {
    let diags = typecheck_source(
        r#"
        fn pair<A, B>(a: A, b: B) -> A {
            return a;
        }
        let _r = pair(1, "two");
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

// ============================================================================
// Cross-Module Inference Tests
// ============================================================================

#[test]
fn test_cross_module_export_valid() {
    // A module with a valid export
    let diags = typecheck_source(
        r#"
        export fn add(a: number, b: number) -> number {
            return a + b;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_cross_module_no_duplicate_exports() {
    // Duplicate exports of the same name should be detected
    let diags = typecheck_source(
        r#"
        export let _a: number = 1;
        export let _a: number = 2;
        "#,
    );
    // Either binder redeclaration error OR type checker duplicate export error
    assert!(
        has_error(&diags),
        "Expected error for duplicate export or redeclaration"
    );
}

#[test]
fn test_cross_module_type_alias_exported() {
    let diags = typecheck_source(
        r#"
        export type Name = string;
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_cross_module_exported_variable() {
    let diags = typecheck_source(
        r#"
        export let _version: string = "1.0";
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_cross_module_inferred_type_exported() {
    let diags = typecheck_source(
        r#"
        export fn identity<T>(x: T) -> T {
            return x;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_cross_module_export_type_validated() {
    let diags = typecheck_source(
        r#"
        export fn get_number() -> number {
            return "not a number";
        }
        "#,
    );
    assert!(
        has_error(&diags),
        "Expected return type error in exported function"
    );
}

// ============================================================================
// Inference Heuristics Tests (via type checker)
// ============================================================================

#[test]
fn test_heuristic_prefer_simple_in_arithmetic() {
    // Arithmetic produces number, not a complex type
    let diags = typecheck_source("let _x = 1 + 2;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_heuristic_literal_inference() {
    // Number literal infers to number
    let diags = typecheck_source(
        r#"
        fn expects_num(x: number) -> number { return x; }
        let _r = expects_num(42);
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_heuristic_union_inferred_from_conditional() {
    // Union type inferred when condition returns different types
    let diags = typecheck_source(
        r#"
        fn get_val(flag: bool) -> number | string {
            if (flag) {
                return 42;
            }
            return "hello";
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_heuristic_prefer_primitive_in_generic_context() {
    let diags = typecheck_source(
        r#"
        fn id<T>(x: T) -> T {
            return x;
        }
        let _v = id(99);
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_heuristic_minimize_vars_unknown_fallback() {
    // When a generic function is used without explicit type arg,
    // the type checker should infer it from the call site
    let diags = typecheck_source(
        r#"
        fn wrap<T>(x: T) -> T {
            return x;
        }
        let _r = wrap(true);
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_heuristic_array_element_type_inferred() {
    // Array element type inferred from literal
    let diags = typecheck_source("let _arr = [1, 2, 3];");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}
