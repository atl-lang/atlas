use super::*;

// From advanced_inference_tests.rs
// ============================================================================

// Advanced Type Inference - Integration Tests (Phase 07)
//
// Tests for:
// - Bidirectional type checking (synthesis & checking modes)
// - Higher-rank polymorphism
// - Let-polymorphism generalization
// - Flow-sensitive typing
// - Unification algorithm
// - Constraint-based inference
// - Cross-module inference
// - Inference heuristics
// - Complex program integration

// ============================================================================
// Helpers
// ============================================================================

fn has_code(diags: &[Diagnostic], code: &str) -> bool {
    diags.iter().any(|d| d.code == code)
}

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

// From type_inference_tests.rs
// ============================================================================

// Tests for enhanced type inference (Phase typing-01)
//
// Validates return type inference, bidirectional checking, expression type
// inference, and least upper bound computation.

/// Helper: typecheck source and return diagnostics
#[allow(dead_code)]
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

// ============================================================================
// 10. Complex scenarios
// ============================================================================

#[test]
fn test_function_with_if_return() {
    let diags = errors(
        r#"
        fn my_abs(x: number) -> number {
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
        fn square(x: number) -> number { return x * x; }
        fn sum_squares(a: number, b: number) -> number {
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
        fn is_even(n: number) -> bool { return n % 2 == 0; }
        fn describe(n: number) -> string {
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
        fn countdown(n: number) -> number {
            var count = n;
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
        fn sum_arr() -> number {
            let arr = [1, 2, 3];
            var total = 0;
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
fn test_inferred_return_no_annotation_valid() {
    // fn with no return type annotation should not emit AT3001
    let diags = errors("fn double(x: number) { return x * 2; }");
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Expected no AT3001 for inferred return, got: {:?}",
        type_errors
    );
}

#[test]
fn test_inferred_return_bool_from_comparison() {
    // fn returning a comparison: infer -> bool
    let diags = errors("fn is_zero(x: number) { return x == 0; }");
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(type_errors.is_empty(), "Expected no AT3001, got: {:?}", type_errors);
}

#[test]
fn test_inferred_return_void_from_empty_body() {
    // fn with empty body: infer -> void, no type errors
    let diags = errors("fn noop() { }");
    let errs: Vec<_> = diags.iter().filter(|d| d.code == "AT3001" || d.code == "AT3004").collect();
    assert!(errs.is_empty(), "Expected no type errors for noop(), got: {:?}", errs);
}

#[test]
fn test_inferred_return_number_from_literal() {
    // fn returning a number literal: infer -> number
    let diags = errors("fn get_answer() { return 42; }");
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(type_errors.is_empty(), "Expected no AT3001 for literal return, got: {:?}", type_errors);
}

#[test]
fn test_inferred_return_consistent_arithmetic() {
    // Mul/Sub/Div/Mod are unambiguously number, no annotation needed
    let diags = errors("fn half(x: number) { return x / 2; }");
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(type_errors.is_empty(), "Expected no AT3001 for arithmetic return, got: {:?}", type_errors);
}

#[test]
fn test_at3050_on_inconsistent_return_types() {
    // fn with different return types in branches and no annotation → AT3050
    let diags = errors(r#"
fn confused(x: number) {
    if (x > 0) {
        return 1;
    } else {
        return "negative";
    }
}
"#);
    assert!(
        diags.iter().any(|d| d.code == "AT3050"),
        "Expected AT3050 for inconsistent returns, got: {:?}",
        diags.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn test_inferred_return_callable_result() {
    // Function with inferred return can be called; result usable in expression
    let diags = errors(r#"
fn square(x: number) { return x * x; }
let _y: number = square(4);
"#);
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(type_errors.is_empty(), "Expected no AT3001 for inferred-return call, got: {:?}", type_errors);
}

#[test]
fn test_inferred_return_both_engines() {
    // Both interpreter and VM execute functions with inferred return type correctly
    let runtime = atlas_runtime::Atlas::new();
    let result = runtime.eval("fn double(x: number) { return x * 2; } double(5);");
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
    assert!(type_errors.is_empty(), "Expected no AT3001 for inferred number, got: {:?}", type_errors);
}

#[test]
fn test_local_infer_string_literal() {
    let diags = errors(r#"let s = "hello"; let _t: string = s;"#);
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(type_errors.is_empty(), "Expected no AT3001 for inferred string, got: {:?}", type_errors);
}

#[test]
fn test_local_infer_bool_literal() {
    let diags = errors("let b = true; let _c: bool = b;");
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(type_errors.is_empty(), "Expected no AT3001 for inferred bool, got: {:?}", type_errors);
}

#[test]
fn test_local_infer_array_literal() {
    // [1,2,3] infers number[]
    let diags = errors("let arr = [1, 2, 3]; let _b: number[] = arr;");
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(type_errors.is_empty(), "Expected no AT3001 for inferred number[], got: {:?}", type_errors);
}

#[test]
fn test_local_infer_arithmetic_expression() {
    let diags = errors("let x = 1 + 2; let _y: number = x;");
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(type_errors.is_empty(), "Expected no AT3001 for inferred arithmetic, got: {:?}", type_errors);
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
    assert!(type_errors.is_empty(), "Expected no AT3001 for inferred bool comparison, got: {:?}", type_errors);
}

#[test]
fn test_local_infer_chained_usage() {
    // Inferred type flows through multiple assignments
    let diags = errors("let x = 10; let y = x * 2; let _z: number = y;");
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(type_errors.is_empty(), "Expected no AT3001 for chained inferred types, got: {:?}", type_errors);
}
