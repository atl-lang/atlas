use super::common::*;

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
