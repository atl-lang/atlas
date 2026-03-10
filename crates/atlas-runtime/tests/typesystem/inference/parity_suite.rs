//! Comprehensive parity test suite (Block 5 Phase 7)

use super::super::*;
#[allow(unused_imports)]
use super::helpers::*;

// ============================================================================
// Comprehensive parity test suite (Block 5 Phase 7)
// ============================================================================

// --- Return type inference parity (6 tests) ---

#[test]
fn parity_return_infer_arithmetic() {
    assert_parity_num(
        "fn double(borrow x: number): number { return x * 2; } double(5);",
        10.0,
    );
}

#[test]
fn parity_return_infer_string_literal() {
    assert_parity_str(
        r#"fn greet(): string { return "hello"; } greet();"#,
        "hello",
    );
}

#[test]
fn parity_return_infer_void_no_return() {
    // Function with no return — returns null/void, both engines return Null
    let interp = interp_eval("fn noop(): void { } noop();");
    let vm = vm_eval("fn noop(): void { } noop();");
    assert_eq!(
        interp,
        Value::Null,
        "Interpreter: void fn should return Null"
    );
    assert_eq!(vm, Some(Value::Null), "VM: void fn should return Null");
}

#[test]
fn parity_return_infer_both_branches() {
    assert_parity_num(
        "fn clamp2(borrow x: number): number { if (x > 0) { return 1; } return 0; } clamp2(5);",
        1.0,
    );
}

#[test]
fn parity_return_infer_with_explicit_params() {
    // Params annotated, explicit return type — parity between engines
    assert_parity_num(
        "fn add(borrow a: number, borrow b: number): number { return a + b; } add(3, 4);",
        7.0,
    );
}

#[test]
fn parity_return_infer_bool_comparison() {
    assert_parity_bool(
        "fn is_positive(borrow x: number): bool { return x > 0; } is_positive(5);",
        true,
    );
}

// --- Local variable inference parity (6 tests) ---

#[test]
fn parity_local_number_arithmetic() {
    assert_parity_num("let x = 42; x + 1;", 43.0);
}

#[test]
fn parity_local_string_stdlib() {
    assert_parity_num(r#"let s = "hello"; len(s);"#, 5.0);
}

#[test]
fn parity_local_array_index() {
    assert_parity_num("let arr = [1, 2, 3]; arr[0];", 1.0);
}

#[test]
fn parity_local_bool_not() {
    assert_parity_bool("let b = true; !b;", false);
}

#[test]
fn parity_local_var_reassignment() {
    assert_parity_num("let mut x = 10; x = 20; x;", 20.0);
}

#[test]
fn parity_local_chained_inference() {
    assert_parity_num("let x = 1; let y = x + 1; y;", 2.0);
}

// --- Anonymous struct literal parity ---

#[test]
fn parity_anonymous_struct_literal_fields() {
    assert_parity_num(
        r#"
        let x = 5;
        let p = { x, y: 2 };
        p.x + p.y;
        "#,
        7.0,
    );
}

// --- Generic call-site inference parity (4 tests) ---

#[test]
fn parity_generic_identity_number() {
    assert_parity_num(
        "fn identity<T>(own x: T): T { return x; } identity(42);",
        42.0,
    );
}

#[test]
fn parity_generic_identity_string() {
    assert_parity_str(
        r#"fn identity<T>(own x: T): T { return x; } identity("hello");"#,
        "hello",
    );
}

#[test]
fn parity_generic_first_element() {
    assert_parity_num(
        "fn first<T>(borrow arr: []T): T { return arr[0]; } first([10, 20, 30]);",
        10.0,
    );
}

#[test]
fn parity_generic_multi_type_params() {
    assert_parity_num(
        "fn pair<T, U>(own x: T, own y: U): T { return x; } pair(99, \"ignored\");",
        99.0,
    );
}

// --- Edge cases (4 tests) ---

#[test]
fn parity_edge_anon_fn_inferred() {
    // Anonymous fn with inferred return: fn(x) { x + 1 }
    assert_parity_num("let f = fn(borrow x: number) { x + 1 }; f(5);", 6.0);
}

#[test]
fn parity_edge_hof_with_inferred_return() {
    // HOF: map with fn-expr having inferred return type
    assert_parity_num(
        "map([1,2,3], fn(borrow x: number) { return x * 2; })[0];",
        2.0,
    );
}

#[test]
fn parity_edge_nested_inferred_functions() {
    // Nested functions with explicit return types
    assert_parity_num(
        r#"
fn outer(borrow x: number): number {
    fn inner(borrow y: number): number { return y * y; }
    return inner(x);
}
outer(4);
"#,
        16.0,
    );
}

#[test]
fn parity_edge_inferred_return_with_own_param() {
    // own ownership annotation on param, explicit return type
    assert_parity_num("fn take(own x: number): number { return x; } take(7);", 7.0);
}
