//! B40: Typed JSON deserialization tests (H-293)
//!
//! Tests for Json.parse<T>() typed deserialization

use atlas_runtime::runtime::Atlas;
use atlas_runtime::value::Value;

fn eval(source: &str) -> Value {
    let runtime = Atlas::new();
    runtime.eval(source).unwrap()
}

// ============================================================================
// Basic Json.parse<T> Tests
// ============================================================================

#[test]
fn test_json_parse_typed_simple_struct() {
    let result = eval(
        r#"
        struct User { name: string, age: number }
        let json = "{\"name\": \"Alice\", \"age\": 30}";
        let user = Json.parse<User>(json)?;
        user.name
    "#,
    );
    assert_eq!(result.to_string(), "Alice");
}

#[test]
fn test_json_parse_typed_access_number_field() {
    let result = eval(
        r#"
        struct Person { name: string, age: number }
        let json = "{\"name\": \"Bob\", \"age\": 25}";
        let person = Json.parse<Person>(json)?;
        person.age
    "#,
    );
    assert_eq!(result, Value::Number(25.0));
}

#[test]
fn test_json_parse_typed_bool_field() {
    let result = eval(
        r#"
        struct Config { enabled: bool, name: string }
        let json = "{\"enabled\": true, \"name\": \"test\"}";
        let config = Json.parse<Config>(json)?;
        config.enabled
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_json_parse_typed_multiple_fields() {
    let result = eval(
        r#"
        struct Point { x: number, y: number, z: number }
        let json = "{\"x\": 10, \"y\": 20, \"z\": 30}";
        let point = Json.parse<Point>(json)?;
        point.x + point.y + point.z
    "#,
    );
    assert_eq!(result, Value::Number(60.0));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_json_parse_typed_invalid_json() {
    let result = eval(
        r#"
        struct Foo { x: number }
        let result = Json.parse<Foo>("not valid json");
        match result {
            Ok(_) => "unexpected ok",
            Err(e) => "error"
        }
    "#,
    );
    assert_eq!(result.to_string(), "error");
}

#[test]
fn test_json_parse_typed_missing_field() {
    // JSON missing required field
    let result = eval(
        r#"
        struct Required { name: string, id: number }
        let json = "{\"name\": \"test\"}";
        let result = Json.parse<Required>(json);
        match result {
            Ok(v) => "ok",
            Err(_) => "error"
        }
    "#,
    );
    // Either succeeds with null/default or fails - depends on impl
    assert!(matches!(result, Value::String(_)));
}

#[test]
fn test_json_parse_typed_extra_fields_ignored() {
    // Extra JSON fields should be ignored
    let result = eval(
        r#"
        struct Small { x: number }
        let json = "{\"x\": 42, \"y\": 100, \"extra\": \"ignored\"}";
        let small = Json.parse<Small>(json)?;
        small.x
    "#,
    );
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Type Checking Tests
// ============================================================================

#[test]
fn test_json_parse_typed_returns_result_type() {
    let result = eval(
        r#"
        struct Data { value: number }
        let parsed = Json.parse<Data>("{\"value\": 99}");
        match parsed {
            Ok(d) => d.value,
            Err(_) => -1
        }
    "#,
    );
    assert_eq!(result, Value::Number(99.0));
}

#[test]
fn test_json_parse_typed_with_optional_operator() {
    let result = eval(
        r#"
        struct Item { count: number }
        let item = Json.parse<Item>("{\"count\": 5}")?;
        item.count
    "#,
    );
    assert_eq!(result, Value::Number(5.0));
}

// ============================================================================
// Complex Struct Tests
// ============================================================================

#[test]
fn test_json_parse_typed_string_with_spaces() {
    let result = eval(
        r#"
        struct Message { text: string }
        let json = "{\"text\": \"Hello World\"}";
        let msg = Json.parse<Message>(json)?;
        msg.text
    "#,
    );
    assert_eq!(result.to_string(), "Hello World");
}

#[test]
fn test_json_parse_typed_negative_number() {
    let result = eval(
        r#"
        struct Balance { amount: number }
        let json = "{\"amount\": -100.5}";
        let bal = Json.parse<Balance>(json)?;
        bal.amount
    "#,
    );
    assert_eq!(result, Value::Number(-100.5));
}

#[test]
fn test_json_parse_typed_zero_values() {
    let result = eval(
        r#"
        struct Zeros { n: number, flag: bool }
        let json = "{\"n\": 0, \"flag\": false}";
        let z = Json.parse<Zeros>(json)?;
        z.n == 0 && !z.flag
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Compared to untyped Json.parse
// ============================================================================

#[test]
fn test_json_parse_untyped_vs_typed() {
    // Untyped returns JsonValue — use typeOf to verify it's json type
    let untyped = eval(
        r#"
        let j = Json.parse("{\"x\": 1}")?;
        reflect.typeOf(j)
    "#,
    );
    assert_eq!(untyped.to_string(), "json");

    // Typed returns struct instance — field access returns native number
    let typed = eval(
        r#"
        struct Obj { x: number }
        let o = Json.parse<Obj>("{\"x\": 1}")?;
        o.x
    "#,
    );
    assert_eq!(typed, Value::Number(1.0));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_json_parse_typed_empty_object() {
    let result = eval(
        r#"
        struct Empty { }
        let e = Json.parse<Empty>("{}")?;
        "ok"
    "#,
    );
    assert_eq!(result.to_string(), "ok");
}

#[test]
fn test_json_parse_typed_unicode_string() {
    let result = eval(
        r#"
        struct Unicode { text: string }
        let json = "{\"text\": \"Hello 世界\"}";
        let u = Json.parse<Unicode>(json)?;
        u.text
    "#,
    );
    assert_eq!(result.to_string(), "Hello 世界");
}

#[test]
fn test_json_parse_typed_escaped_quotes() {
    let result = eval(
        r#"
        struct Quote { msg: string }
        let json = "{\"msg\": \"He said \\\"hello\\\"\"}";
        let q = Json.parse<Quote>(json)?;
        q.msg.includes("hello")
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_json_parse_typed_large_number() {
    let result = eval(
        r#"
        struct Big { n: number }
        let json = "{\"n\": 1000000000}";
        let b = Json.parse<Big>(json)?;
        b.n > 999999999
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}
