use super::common::*;

// ============================================================================
// Struct Expressions (compile to HashMap at runtime)
// ============================================================================

#[test]
fn struct_expr_basic() {
    // Struct expressions are syntactic sugar for creating HashMap objects
    let code = r#"
        let user = User { name: "Alice", age: 30 };
        unwrap(hashMapGet(user, "name"))
    "#;
    assert_eval_string(code, "Alice");
}

#[test]
fn struct_expr_access_field() {
    let code = r#"
        let point = Point { x: 10, y: 20 };
        unwrap(hashMapGet(point, "y"))
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn struct_expr_empty() {
    let code = r#"
        let empty = Empty {};
        hashMapSize(empty)
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
        unwrap(hashMapGet(obj, "name"))
    "#;
    assert_eval_string(code, "Bob");
}

#[test]
fn object_literal_field_assignment() {
    let code = r#"
        let obj = record { name: "Bob", count: 1 };
        obj.count = 5;
        unwrap(hashMapGet(obj, "count"))
    "#;
    assert_eval_number(code, 5.0);
}

#[test]
fn object_literal_field_compound_assignment() {
    let code = r#"
        let obj = record { count: 2 };
        obj.count += 3;
        unwrap(hashMapGet(obj, "count"))
    "#;
    assert_eval_number(code, 5.0);
}

#[test]
fn object_literal_number_value() {
    let code = r#"
        let obj = record { x: 10, y: 20 };
        unwrap(hashMapGet(obj, "x"))
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn object_literal_trailing_comma() {
    let code = r#"
        let obj = record { a: 1, b: 2, };
        hashMapSize(obj)
    "#;
    assert_eval_number(code, 2.0);
}

#[test]
fn object_literal_empty() {
    let code = r#"
        let obj = record {};
        hashMapSize(obj)
    "#;
    assert_eval_number(code, 0.0);
}

// ============================================================================
// HashMap JSON Serialization (toJSON)
// ============================================================================

#[test]
fn hashmap_tojson_basic() {
    // Test that toJSON works with object literals (which create HashMaps)
    let code = r#"
        let obj = record { name: "test" };
        let json = toJSON(obj);
        len(json) > 0
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn hashmap_tojson_multiple_fields() {
    let code = r#"
        let obj = record { a: 1, b: 2 };
        let json = toJSON(obj);
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
        let json = toJSON(obj);
        // Should contain "items" and array
        len(json) > 15
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn struct_expr_tojson() {
    // Struct expressions should also serialize via toJSON
    let code = r#"
        let user = User { name: "Alice", active: true };
        let json = toJSON(user);
        // Should contain the data
        len(json) > 20
    "#;
    assert_eval_bool(code, true);
}
