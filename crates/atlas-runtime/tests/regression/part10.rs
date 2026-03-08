use super::common::*;

// ============================================================================
// H-088: Loop variable reassignment must not collapse type to never
// ============================================================================

#[test]
fn test_h088_while_loop_mutable_var_not_never() {
    // Previously: `bb != 0` narrowed bb to Never inside the loop body
    // (exclude_from(Number, Number) = Never), breaking arithmetic.
    let code = r#"
        fn gcd(borrow a: number, borrow b: number) -> number {
            let mut aa = a;
            let mut bb = b;
            while bb != 0 {
                let temp = bb;
                bb = aa % bb;
                aa = temp;
            }
            return aa;
        }
        gcd(48, 18)
    "#;
    assert_eval_number(code, 6.0);
}

#[test]
fn test_h088_while_loop_counter() {
    // Simple counter — reassigning a loop variable must not produce Never
    let code = r#"
        let mut i = 0;
        let mut sum = 0;
        while i != 5 {
            sum = sum + i;
            i = i + 1;
        }
        sum
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_h088_while_string_condition_not_narrowed() {
    // String variable used in != condition must stay string inside loop
    let code = r#"
        let mut s = "hello";
        let mut count = 0;
        while s != "" {
            count = count + 1;
            s = "";
        }
        count
    "#;
    assert_eval_number(code, 1.0);
}

// ============================================================================
// Struct Expressions (compile to HashMap at runtime)
// ============================================================================

#[test]
fn struct_expr_basic() {
    // Struct expressions are syntactic sugar for creating HashMap objects
    let code = r#"
        let user = User { name: "Alice", age: 30 };
        unwrap(hash_map_get(user, "name"))
    "#;
    assert_eval_string(code, "Alice");
}

#[test]
fn struct_expr_access_field() {
    let code = r#"
        let point = Point { x: 10, y: 20 };
        unwrap(hash_map_get(point, "y"))
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn struct_expr_empty() {
    let code = r#"
        let empty = Empty {};
        hash_map_size(empty)
    "#;
    assert_eval_number(code, 0.0);
}

// ============================================================================
// Object Literals (compile to HashMap at runtime)
// ============================================================================

#[test]
fn object_literal_basic() {
    let code = r#"
        let obj = record { name: "Bob", count: 42 };
        unwrap(hash_map_get(obj, "name"))
    "#;
    assert_eval_string(code, "Bob");
}

#[test]
fn object_literal_field_assignment() {
    let code = r#"
        let mut obj = record { name: "Bob", count: 1 };
        obj.count = 5;
        unwrap(hash_map_get(obj, "count"))
    "#;
    assert_eval_number(code, 5.0);
}

#[test]
fn object_literal_field_compound_assignment() {
    let code = r#"
        let mut obj = record { count: 2 };
        obj.count += 3;
        unwrap(hash_map_get(obj, "count"))
    "#;
    assert_eval_number(code, 5.0);
}

#[test]
fn object_literal_number_value() {
    let code = r#"
        let obj = record { x: 10, y: 20 };
        unwrap(hash_map_get(obj, "x"))
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn object_literal_trailing_comma() {
    let code = r#"
        let obj = record { a: 1, b: 2, };
        hash_map_size(obj)
    "#;
    assert_eval_number(code, 2.0);
}

#[test]
fn object_literal_empty() {
    let code = r#"
        let obj = record {};
        hash_map_size(obj)
    "#;
    assert_eval_number(code, 0.0);
}

// ============================================================================
// HashMap JSON Serialization (to_json)
// ============================================================================

#[test]
fn hashmap_tojson_basic() {
    // Test that to_json works with object literals (which create HashMaps)
    let code = r#"
        let obj = record { name: "test" };
        let json = to_json(obj);
        len(json) > 0
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn hashmap_tojson_multiple_fields() {
    let code = r#"
        let obj = record { a: 1, b: 2 };
        let json = to_json(obj);
        // JSON should contain both keys - check length
        len(json) > 10
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn hashmap_tojson_nested() {
    // Object literals with nested arrays should serialize
    let code = r#"
        let obj = record { items: [1, 2, 3] };
        let json = to_json(obj);
        // Should contain "items" and array
        len(json) > 15
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn struct_expr_tojson() {
    // Struct expressions should also serialize via to_json
    let code = r#"
        let user = User { name: "Alice", active: true };
        let json = to_json(user);
        // Should contain the data
        len(json) > 20
    "#;
    assert_eval_bool(code, true);
}

// ============================================================================
// H-078: Option/Result method dispatch (unwrap returns T, not any)
// ============================================================================

#[test]
fn h078_option_unwrap_returns_value() {
    let code = r#"
        let opt: Option<number> = Some(42);
        opt.unwrap()
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn h078_option_unwrap_or_returns_inner() {
    let code = r#"
        let opt: Option<number> = Some(10);
        opt.unwrapOr(0)
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn h078_option_unwrap_or_returns_default_on_none() {
    let code = r#"
        let opt: Option<number> = None;
        opt.unwrapOr(99)
    "#;
    assert_eval_number(code, 99.0);
}

#[test]
fn h078_option_is_some() {
    let code = r#"
        let opt: Option<number> = Some(1);
        opt.isSome()
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn h078_option_is_none() {
    let code = r#"
        let opt: Option<number> = None;
        opt.isNone()
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn h078_result_unwrap_ok() {
    let code = r#"
        let r: Result<number, string> = Ok(7);
        r.unwrap()
    "#;
    assert_eval_number(code, 7.0);
}

#[test]
fn h078_result_is_ok() {
    let code = r#"
        let r: Result<number, string> = Ok(7);
        r.isOk()
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn h078_result_is_err() {
    let code = r#"
        let r: Result<number, string> = Err("oops");
        r.isErr()
    "#;
    assert_eval_bool(code, true);
}

// ============================================================================
// H-089: mut function parameters — params declared `mut` can be reassigned
// ============================================================================

#[test]
fn test_h089_mut_param_gcd() {
    // Natural GCD algorithm — Haiku writes params as mutable, not via let copies
    let code = r#"
        fn gcd(mut a: number, mut b: number) -> number {
            while b != 0 {
                let tmp = b;
                b = a % b;
                a = tmp;
            }
            return a;
        }
        gcd(48, 18)
    "#;
    assert_eval_number(code, 6.0);
}

#[test]
fn test_h089_immutable_param_still_rejected() {
    // Without mut, params must remain immutable
    let code = r#"
        fn bad(borrow a: number) -> number {
            a = 5;
            return a;
        }
        bad(3)
    "#;
    assert_error_code(code, "AT3003");
}

#[test]
fn test_h089_mut_param_single() {
    // Single mut param — simple mutation
    let code = r#"
        fn double_it(mut x: number) -> number {
            x = x * 2;
            return x;
        }
        double_it(7)
    "#;
    assert_eval_number(code, 14.0);
}

// ============================================================================
// H-092: Local let mut must not collide with outer let of same name
// ============================================================================

#[test]
fn test_h092_local_mut_shadows_outer_immutable() {
    // A local `let mut x` inside a function must shadow an outer `let x`
    // and be independently mutable. Previously typechecker resolved lookup
    // to the outer symbol and rejected assignment as AT3003.
    let code = r#"
        let primes = 0;
        fn compute(borrow n: number) -> number {
            let mut primes = n;
            primes = primes * 2;
            return primes;
        }
        compute(5)
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_h092_outer_binding_unchanged() {
    // The outer binding must remain unchanged after the function runs
    let code = r#"
        let x = 99;
        fn f() -> number {
            let mut x = 1;
            x = x + 1;
            return x;
        }
        f() + x
    "#;
    assert_eval_number(code, 101.0);
}
