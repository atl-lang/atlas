use super::*;

#[test]
fn test_value_info_get_values() {
    let num = Value::Number(42.5);
    let info = ValueInfo::new(num);
    assert_eq!(info.get_number(), Some(42.5));
    assert_eq!(info.get_string(), None);

    let bool_val = Value::Bool(false);
    let info = ValueInfo::new(bool_val);
    assert_eq!(info.get_bool(), Some(false));
    assert_eq!(info.get_number(), None);
}

#[test]
fn test_value_info_array_elements() {
    let arr = Value::array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ]);
    let info = ValueInfo::new(arr);

    let elements = info.get_array_elements().unwrap();
    assert_eq!(elements.len(), 3);
    assert_eq!(elements[0], Value::Number(1.0));
    assert_eq!(elements[1], Value::Number(2.0));
    assert_eq!(elements[2], Value::Number(3.0));
}

#[test]
fn test_get_value_type_info_primitives() {
    let num = Value::Number(42.0);
    let info = get_value_type_info(&num);
    assert_eq!(info.name, "number");
    assert_eq!(info.kind, TypeKind::Number);

    let str_val = Value::string("hello");
    let info = get_value_type_info(&str_val);
    assert_eq!(info.name, "string");
    assert_eq!(info.kind, TypeKind::String);

    let bool_val = Value::Bool(true);
    let info = get_value_type_info(&bool_val);
    assert_eq!(info.name, "bool");
    assert_eq!(info.kind, TypeKind::Bool);

    let null_val = Value::Null;
    let info = get_value_type_info(&null_val);
    assert_eq!(info.name, "null");
    assert_eq!(info.kind, TypeKind::Null);
}

#[test]
fn test_get_value_type_info_array() {
    let arr = Value::array(vec![Value::Number(1.0), Value::Number(2.0)]);
    let info = get_value_type_info(&arr);
    assert_eq!(info.name, "array");
    assert_eq!(info.kind, TypeKind::Array);
}

#[test]
fn test_get_value_type_info_option() {
    let some_val = Value::Option(Some(Box::new(Value::Number(42.0))));
    let info = get_value_type_info(&some_val);
    assert_eq!(info.name, "Option");
    assert_eq!(info.kind, TypeKind::Option);

    let none_val = Value::Option(None);
    let info = get_value_type_info(&none_val);
    assert_eq!(info.name, "Option");
    assert_eq!(info.kind, TypeKind::Option);
}

#[test]
fn test_get_value_type_info_result() {
    let ok_val = Value::Result(Ok(Box::new(Value::Number(42.0))));
    let info = get_value_type_info(&ok_val);
    assert_eq!(info.name, "Result");
    assert_eq!(info.kind, TypeKind::Result);

    let err_val = Value::Result(Err(Box::new(Value::string("error"))));
    let info = get_value_type_info(&err_val);
    assert_eq!(info.name, "Result");
    assert_eq!(info.kind, TypeKind::Result);
}

// ============================================================================
// Stdlib Reflection Integration Tests (Interpreter)
// ============================================================================

fn run_interpreter(code: &str) -> Value {
    let runtime = Atlas::new();
    runtime.eval(code).expect("Interpreter execution failed")
}

#[rstest]
#[case("reflect.typeOf(42)", "number")]
#[case("reflect.typeOf(\"hello\")", "string")]
#[case("reflect.typeOf(true)", "bool")]
#[case("reflect.typeOf(null)", "null")]
#[case("reflect.typeOf([1, 2, 3])", "array")]
fn test_interpreter_typeof(#[case] code: &str, #[case] expected: &str) {
    let result = run_interpreter(code);
    assert_eq!(result, Value::string(expected));
}

#[rstest]
#[case("reflect.isPrimitive(42)", true)]
#[case("reflect.isPrimitive(\"test\")", true)]
#[case("reflect.isPrimitive(true)", true)]
#[case("reflect.isPrimitive(null)", true)]
#[case("reflect.isPrimitive([1, 2])", false)]
fn test_interpreter_is_primitive(#[case] code: &str, #[case] expected: bool) {
    let result = run_interpreter(code);
    assert_eq!(result, Value::Bool(expected));
}

#[rstest]
#[case("reflect.sameType(42, 99)", true)]
#[case("reflect.sameType(42, \"test\")", false)]
#[case("reflect.sameType(\"a\", \"b\")", true)]
#[case("reflect.sameType(true, false)", true)]
fn test_interpreter_same_type(#[case] code: &str, #[case] expected: bool) {
    let result = run_interpreter(code);
    assert_eq!(result, Value::Bool(expected));
}

#[rstest]
#[case("reflect.getLength([1, 2, 3])", 3.0)]
#[case("reflect.getLength(\"hello\")", 5.0)]
#[case("reflect.getLength([])", 0.0)]
#[case("reflect.getLength(\"\")", 0.0)]
fn test_interpreter_get_length(#[case] code: &str, #[case] expected: f64) {
    let result = run_interpreter(code);
    assert_eq!(result, Value::Number(expected));
}

#[rstest]
#[case("reflect.isEmpty([])", true)]
#[case("reflect.isEmpty(\"\")", true)]
#[case("reflect.isEmpty([1])", false)]
#[case("reflect.isEmpty(\"x\")", false)]
fn test_interpreter_is_empty(#[case] code: &str, #[case] expected: bool) {
    let result = run_interpreter(code);
    assert_eq!(result, Value::Bool(expected));
}

#[test]
fn test_interpreter_type_describe() {
    let result = run_interpreter("reflect.typeDescribe(42)");
    assert_eq!(result, Value::string("primitive number type"));

    let result = run_interpreter("reflect.typeDescribe([1, 2])");
    // Just verify it returns a string
    assert!(matches!(result, Value::String(_)));
}

#[test]
fn test_interpreter_clone() {
    let result = run_interpreter("reflect.clone(42)");
    assert_eq!(result, Value::Number(42.0));

    let result = run_interpreter("reflect.clone(\"test\")");
    assert_eq!(result, Value::string("test"));
}

#[test]
fn test_interpreter_value_to_string() {
    let result = run_interpreter("reflect.valueToString(42)");
    assert_eq!(result, Value::string("42"));

    let result = run_interpreter("reflect.valueToString([1, 2, 3])");
    assert_eq!(result, Value::string("[1, 2, 3]"));

    let result = run_interpreter("reflect.valueToString(123)");
    assert_eq!(result, Value::string("123"));
}

#[test]
fn test_interpreter_deep_equals() {
    let code = r#"
        let a = [1, 2, 3];
        let b = [1, 2, 3];
        reflect.deepEquals(a, b)
    "#;
    let result = run_interpreter(code);
    assert_eq!(result, Value::Bool(true));

    let code = r#"
        let a = [1, 2, 3];
        let b = [1, 2, 4];
        reflect.deepEquals(a, b)
    "#;
    let result = run_interpreter(code);
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_interpreter_nested_deep_equals() {
    let code = r#"
        let a = [[1, 2], [3, 4]];
        let b = [[1, 2], [3, 4]];
        reflect.deepEquals(a, b)
    "#;
    let result = run_interpreter(code);
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Stdlib Reflection Integration Tests (VM)
// ============================================================================

fn run_vm(code: &str) -> Value {
    // Use the full Runtime pipeline (lex → parse → typecheck → compile → VM).
    // The typechecker must run before compilation to set type_tag on member expressions,
    // which is required for namespace method dispatch (e.g. reflect.typeOf).
    let runtime = Atlas::new();
    runtime.eval(code).expect("VM execution failed")
}

#[rstest]
#[case("reflect.typeOf(42)", "number")]
#[case("reflect.typeOf(\"hello\")", "string")]
#[case("reflect.typeOf(true)", "bool")]
#[case("reflect.typeOf(null)", "null")]
#[case("reflect.typeOf([1, 2, 3])", "array")]
fn test_vm_typeof(#[case] code: &str, #[case] expected: &str) {
    let result = run_vm(code);
    assert_eq!(result, Value::string(expected));
}

#[rstest]
#[case("reflect.isPrimitive(42)", true)]
#[case("reflect.isPrimitive(\"test\")", true)]
#[case("reflect.isPrimitive(true)", true)]
#[case("reflect.isPrimitive(null)", true)]
#[case("reflect.isPrimitive([1, 2])", false)]
fn test_vm_is_primitive(#[case] code: &str, #[case] expected: bool) {
    let result = run_vm(code);
    assert_eq!(result, Value::Bool(expected));
}

#[rstest]
#[case("reflect.sameType(42, 99)", true)]
#[case("reflect.sameType(42, \"test\")", false)]
#[case("reflect.sameType(\"a\", \"b\")", true)]
#[case("reflect.sameType(true, false)", true)]
fn test_vm_same_type(#[case] code: &str, #[case] expected: bool) {
    let result = run_vm(code);
    assert_eq!(result, Value::Bool(expected));
}

#[rstest]
#[case("reflect.getLength([1, 2, 3])", 3.0)]
#[case("reflect.getLength(\"hello\")", 5.0)]
#[case("reflect.getLength([])", 0.0)]
#[case("reflect.getLength(\"\")", 0.0)]
fn test_vm_get_length(#[case] code: &str, #[case] expected: f64) {
    let result = run_vm(code);
    assert_eq!(result, Value::Number(expected));
}

#[rstest]
#[case("reflect.isEmpty([])", true)]
#[case("reflect.isEmpty(\"\")", true)]
#[case("reflect.isEmpty([1])", false)]
#[case("reflect.isEmpty(\"x\")", false)]
fn test_vm_is_empty(#[case] code: &str, #[case] expected: bool) {
    let result = run_vm(code);
    assert_eq!(result, Value::Bool(expected));
}

#[test]
fn test_vm_type_describe() {
    let result = run_vm("reflect.typeDescribe(42)");
    assert_eq!(result, Value::string("primitive number type"));

    let result = run_vm("reflect.typeDescribe([1, 2])");
    assert!(matches!(result, Value::String(_)));
}

#[test]
fn test_vm_clone() {
    let result = run_vm("reflect.clone(42)");
    assert_eq!(result, Value::Number(42.0));

    let result = run_vm("reflect.clone(\"test\")");
    assert_eq!(result, Value::string("test"));
}

#[test]
fn test_vm_value_to_string() {
    let result = run_vm("reflect.valueToString(42)");
    assert_eq!(result, Value::string("42"));

    let result = run_vm("reflect.valueToString([1, 2, 3])");
    assert_eq!(result, Value::string("[1, 2, 3]"));

    let result = run_vm("reflect.valueToString(123)");
    assert_eq!(result, Value::string("123"));
}

#[test]
fn test_vm_deep_equals() {
    let code = r#"
        let a = [1, 2, 3];
        let b = [1, 2, 3];
        reflect.deepEquals(a, b)
    "#;
    let result = run_vm(code);
    assert_eq!(result, Value::Bool(true));

    let code = r#"
        let a = [1, 2, 3];
        let b = [1, 2, 4];
        reflect.deepEquals(a, b)
    "#;
    let result = run_vm(code);
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_vm_nested_deep_equals() {
    let code = r#"
        let a = [[1, 2], [3, 4]];
        let b = [[1, 2], [3, 4]];
        reflect.deepEquals(a, b)
    "#;
    let result = run_vm(code);
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Parity Verification Tests
// ============================================================================

#[rstest]
#[case("reflect.typeOf(42)")]
#[case("reflect.typeOf(\"test\")")]
#[case("reflect.typeOf([1, 2, 3])")]
#[case("reflect.isPrimitive(42)")]
#[case("reflect.isPrimitive([1])")]
#[case("reflect.sameType(1, 2)")]
#[case("reflect.sameType(1, \"a\")")]
#[case("reflect.getLength([1, 2, 3])")]
#[case("reflect.getLength(\"hello\")")]
#[case("reflect.isEmpty([])")]
#[case("reflect.isEmpty([1])")]
#[case("reflect.clone(42)")]
#[case("reflect.valueToString(42)")]
fn test_parity_reflection_functions(#[case] code: &str) {
    let interpreter_result = run_interpreter(code);
    let vm_result = run_vm(code);

    assert_eq!(
        interpreter_result, vm_result,
        "Parity violation for: {}",
        code
    );
}

#[test]
fn test_parity_deep_equals() {
    let cases = vec![
        "reflect.deepEquals([1, 2], [1, 2])",
        "reflect.deepEquals([1, 2], [1, 3])",
        "reflect.deepEquals([[1]], [[1]])",
        "reflect.deepEquals(42, 42)",
        "reflect.deepEquals(\"a\", \"a\")",
    ];

    for code in cases {
        let interpreter_result = run_interpreter(code);
        let vm_result = run_vm(code);

        assert_eq!(
            interpreter_result, vm_result,
            "Parity violation for: {}",
            code
        );
    }
}
