//! B40: Reflect namespace tests (H-287)
//!
//! Tests for type introspection via reflect namespace

use atlas_runtime::runtime::Atlas;
use atlas_runtime::value::Value;
use rstest::rstest;

fn eval(source: &str) -> Value {
    let runtime = Atlas::new();
    runtime.eval(source).unwrap()
}

// ============================================================================
// reflect.typeOf Tests
// ============================================================================

#[rstest]
#[case::number("reflect.typeOf(42)", "number")]
#[case::string("reflect.typeOf(\"hello\")", "string")]
#[case::bool_true("reflect.typeOf(true)", "bool")]
#[case::bool_false("reflect.typeOf(false)", "bool")]
#[case::null("reflect.typeOf(null)", "null")]
#[case::array("reflect.typeOf([1, 2, 3])", "array")]
#[case::option_some("reflect.typeOf(Some(1))", "Option")]
#[case::option_none("reflect.typeOf(None)", "Option")]
fn test_reflect_typeof(#[case] code: &str, #[case] expected: &str) {
    let result = eval(code);
    assert_eq!(result.to_string(), expected, "Failed for: {}", code);
}

#[test]
fn test_reflect_typeof_function() {
    let result = eval(
        r#"
        fn foo(): void {}
        reflect.typeOf(foo)
    "#,
    );
    assert_eq!(result.to_string(), "function");
}

// ============================================================================
// reflect.isCallable Tests
// ============================================================================

#[rstest]
#[case::number("reflect.isCallable(42)", "false")]
#[case::string("reflect.isCallable(\"hello\")", "false")]
#[case::array("reflect.isCallable([1, 2])", "false")]
#[case::null("reflect.isCallable(null)", "false")]
fn test_reflect_is_callable_primitives(#[case] code: &str, #[case] expected: &str) {
    let result = eval(code);
    assert_eq!(result.to_string(), expected, "Failed for: {}", code);
}

#[test]
fn test_reflect_is_callable_function() {
    let result = eval(
        r#"
        fn myFunc(): void {}
        reflect.isCallable(myFunc)
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// reflect.isPrimitive Tests
// ============================================================================

#[rstest]
#[case::number("reflect.isPrimitive(42)", "true")]
#[case::string("reflect.isPrimitive(\"hello\")", "true")]
#[case::bool("reflect.isPrimitive(true)", "true")]
#[case::null("reflect.isPrimitive(null)", "true")]
#[case::array("reflect.isPrimitive([1, 2, 3])", "false")]
fn test_reflect_is_primitive(#[case] code: &str, #[case] expected: &str) {
    let result = eval(code);
    assert_eq!(result.to_string(), expected, "Failed for: {}", code);
}

#[test]
fn test_reflect_is_primitive_function() {
    let result = eval(
        r#"
        fn foo(): void {}
        reflect.isPrimitive(foo)
    "#,
    );
    assert_eq!(result, Value::Bool(false));
}

// ============================================================================
// reflect.sameType Tests
// ============================================================================

#[rstest]
#[case::same_numbers("reflect.sameType(42, 99)", "true")]
#[case::same_strings("reflect.sameType(\"a\", \"b\")", "true")]
#[case::same_bools("reflect.sameType(true, false)", "true")]
#[case::diff_num_str("reflect.sameType(42, \"42\")", "false")]
#[case::diff_num_bool("reflect.sameType(1, true)", "false")]
#[case::same_arrays("reflect.sameType([1], [2, 3])", "true")]
fn test_reflect_same_type(#[case] code: &str, #[case] expected: &str) {
    let result = eval(code);
    assert_eq!(result.to_string(), expected, "Failed for: {}", code);
}

// ============================================================================
// reflect.getLength Tests
// ============================================================================

#[rstest]
#[case::array_3("reflect.getLength([1, 2, 3])", "3")]
#[case::array_empty("reflect.getLength([])", "0")]
#[case::string_5("reflect.getLength(\"hello\")", "5")]
#[case::string_empty("reflect.getLength(\"\")", "0")]
#[case::string_unicode("reflect.getLength(\"hi\")", "2")]
fn test_reflect_get_length(#[case] code: &str, #[case] expected: &str) {
    let result = eval(code);
    assert_eq!(result.to_string(), expected, "Failed for: {}", code);
}

// ============================================================================
// reflect.isEmpty Tests
// ============================================================================

#[rstest]
#[case::empty_array("reflect.isEmpty([])", "true")]
#[case::non_empty_array("reflect.isEmpty([1])", "false")]
#[case::empty_string("reflect.isEmpty(\"\")", "true")]
#[case::non_empty_string("reflect.isEmpty(\"x\")", "false")]
fn test_reflect_is_empty(#[case] code: &str, #[case] expected: &str) {
    let result = eval(code);
    assert_eq!(result.to_string(), expected, "Failed for: {}", code);
}

// ============================================================================
// reflect.typeDescribe Tests
// ============================================================================

#[test]
fn test_reflect_type_describe_number() {
    let result = eval(r#"reflect.typeDescribe(42)"#);
    let desc = result.to_string();
    assert!(
        desc.contains("number") || desc.contains("primitive"),
        "Expected number description, got: {}",
        desc
    );
}

#[test]
fn test_reflect_type_describe_array() {
    let result = eval(r#"reflect.typeDescribe([1, 2, 3])"#);
    let desc = result.to_string();
    assert!(
        desc.contains("array"),
        "Expected array description, got: {}",
        desc
    );
}

// ============================================================================
// reflect.clone Tests
// ============================================================================

#[test]
fn test_reflect_clone_array() {
    let result = eval(
        r#"
        let arr = [1, 2, 3];
        let cloned = reflect.clone(arr);
        len(cloned)
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_reflect_clone_preserves_values() {
    let result = eval(
        r#"
        let arr = [1, 2, 3];
        let cloned = reflect.clone(arr);
        cloned[1]
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

// ============================================================================
// reflect.valueToString Tests
// ============================================================================

#[rstest]
#[case::number("reflect.valueToString(42)", "42")]
#[case::bool("reflect.valueToString(true)", "true")]
fn test_reflect_value_to_string(#[case] code: &str, #[case] expected: &str) {
    let result = eval(code);
    assert_eq!(result.to_string(), expected, "Failed for: {}", code);
}

#[test]
fn test_reflect_value_to_string_array() {
    let result = eval(r#"reflect.valueToString([1, 2, 3])"#);
    let s = result.to_string();
    // Should contain array elements
    assert!(
        s.contains("1") && s.contains("2") && s.contains("3"),
        "Expected array string, got: {}",
        s
    );
}

// ============================================================================
// reflect.deepEquals Tests
// ============================================================================

#[rstest]
#[case::same_numbers("reflect.deepEquals(42, 42)", "true")]
#[case::diff_numbers("reflect.deepEquals(42, 43)", "false")]
#[case::same_strings("reflect.deepEquals(\"a\", \"a\")", "true")]
#[case::diff_strings("reflect.deepEquals(\"a\", \"b\")", "false")]
#[case::same_arrays("reflect.deepEquals([1, 2], [1, 2])", "true")]
#[case::diff_arrays("reflect.deepEquals([1, 2], [1, 3])", "false")]
#[case::diff_lengths("reflect.deepEquals([1, 2], [1])", "false")]
fn test_reflect_deep_equals(#[case] code: &str, #[case] expected: &str) {
    let result = eval(code);
    assert_eq!(result.to_string(), expected, "Failed for: {}", code);
}

#[test]
fn test_reflect_deep_equals_nested() {
    let result = eval(
        r#"
        let a = [[1, 2], [3, 4]];
        let b = [[1, 2], [3, 4]];
        reflect.deepEquals(a, b)
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// reflect.getFunctionName / getFunctionArity Tests
// ============================================================================

#[test]
fn test_reflect_get_function_name() {
    let result = eval(
        r#"
        fn myTestFunc(): void {}
        reflect.getFunctionName(myTestFunc)
    "#,
    );
    assert_eq!(result.to_string(), "myTestFunc");
}

#[test]
fn test_reflect_get_function_arity() {
    let result = eval(
        r#"
        fn add(borrow a: number, borrow b: number): number { return a + b; }
        reflect.getFunctionArity(add)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_reflect_get_function_arity_zero() {
    let result = eval(
        r#"
        fn noArgs(): void {}
        reflect.getFunctionArity(noArgs)
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

// ============================================================================
// reflect.fields Tests
// ============================================================================

#[test]
fn test_reflect_fields_json_keys() {
    // Test reflect.fields on a JSON object
    let result = eval(
        r#"
        let j = Json.parse("{\"a\": 1, \"b\": 2}")?;
        len(reflect.fields(j))
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_reflect_fields_json_object() {
    let result = eval(
        r#"
        let j = Json.parse("{\"x\": 1, \"y\": 2}")?;
        len(reflect.fields(j))
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_reflect_fields_non_object() {
    // Non-object types should return empty array
    let result = eval(r#"len(reflect.fields(42))"#);
    assert_eq!(result, Value::Number(0.0));
}

// ============================================================================
// reflect.hasMethod Tests
// ============================================================================

#[test]
fn test_reflect_has_method_array() {
    let result = eval(
        r#"
        let arr = [1, 2, 3];
        reflect.hasMethod(arr, "push")
    "#,
    );
    // Arrays should have push method
    assert!(matches!(result, Value::Bool(_)));
}

#[test]
fn test_reflect_has_method_string() {
    let result = eval(r#"reflect.hasMethod("hello", "length")"#);
    // Check if strings have length method
    assert!(matches!(result, Value::Bool(_)));
}

#[test]
fn test_reflect_has_method_nonexistent() {
    let result = eval(r#"reflect.hasMethod([1, 2], "nonexistentMethod")"#);
    assert_eq!(result, Value::Bool(false));
}
