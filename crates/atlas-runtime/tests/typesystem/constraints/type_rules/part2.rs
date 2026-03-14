use super::super::super::*;

// ========== Complex Context Tests ==========

#[rstest]
#[case::function_arithmetic(
    r#"fn add(borrow a: number, borrow b: number): number { return a + b; }"#
)]
#[case::conditional_operators(
    r#"let x: number = 5; let y: number = 10; if (x < y && y > 0) { let z = x + y; }"#
)]
#[case::loop_operators(r#"let i: number = 0; while (i < 10) { let x = i + 1; }"#)]
fn test_operators_in_context(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

// ========== Method Call Type Checking ==========

#[rstest]
#[case::json_as_string(r#"let data: json = unwrap(Json.parse("{\"name\":\"Alice\"}")); let name: string = data["name"].asString();"#)]
#[case::json_as_number(
    r#"let data: json = unwrap(Json.parse("{\"age\":30}")); let age: number = data["age"].asNumber();"#
)]
#[case::json_as_bool(r#"let data: json = unwrap(Json.parse("{\"active\":true}")); let active: bool = data["active"].asBool();"#)]
#[case::jsonIsNull(r#"let data: json = unwrap(Json.parse("{\"value\":null}")); let null_check: bool = data["value"].isNull();"#)]
#[case::chained_json_access(r#"let data: json = unwrap(Json.parse("{\"user\":{\"name\":\"Bob\"}}")); let name: string = data["user"]["name"].asString();"#)]
#[case::method_in_expression(r#"let data: json = unwrap(Json.parse("{\"count\":5}")); let x: number = data["count"].asNumber() + 10;"#)]
#[case::method_as_arg(
    r#"let data: json = unwrap(Json.parse("{\"msg\":\"hi\"}")); console.log(data["msg"].asString());"#
)]
fn test_valid_method_calls(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::invalid_method_name(
    r#"let data: json = unwrap(Json.parse("{}")); data.invalid_method();"#,
    "AT3010"
)]
#[case::method_on_wrong_type("let x: number = 42; x.asString();", "AT3010")]
#[case::method_on_string_type(r#"let s: string = "hello"; s.asNumber();"#, "AT3010")]
#[case::method_on_bool_type("let b: bool = true; b.asString();", "AT3010")]
fn test_invalid_method_calls(#[case] source: &str, #[case] error_code: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, error_code);
}

#[rstest]
#[case::too_many_args(
    r#"let data: json = unwrap(Json.parse("{}")); data.asString(42);"#,
    "AT3005"
)]
#[case::too_many_multiple(
    r#"let data: json = unwrap(Json.parse("{}")); data.asString(1, 2);"#,
    "AT3005"
)]
fn test_method_argument_count_errors(#[case] source: &str, #[case] error_code: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, error_code);
}

#[rstest]
#[case::wrong_return_type_string(
    r#"let data: json = unwrap(Json.parse("{\"x\":1}")); let x: string = data["x"].asNumber();"#
)]
#[case::wrong_return_type_number(
    r#"let data: json = unwrap(Json.parse("{\"x\":\"y\"}")); let x: number = data["x"].asString();"#
)]
#[case::wrong_return_type_bool(
    r#"let data: json = unwrap(Json.parse("{\"x\":1}")); let x: bool = data["x"].asNumber();"#
)]
fn test_method_return_type_mismatch(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

#[test]
fn test_chained_method_calls_type_correctly() {
    let source = r#"
        let data: json = unwrap(Json.parse("{\"a\":{\"b\":{\"c\":\"value\"}}}"));
        let result: string = data["a"]["b"]["c"].asString();
    "#;
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[test]
fn test_method_call_in_conditional() {
    let source = r#"
        let data: json = unwrap(Json.parse("{\"enabled\":true}"));
        if (data["enabled"].asBool()) {
            console.log("Enabled");
        }
    "#;
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[test]
fn test_multiple_method_calls_in_expression() {
    let source = r#"
        let data: json = unwrap(Json.parse("{\"a\":5,\"b\":10}"));
        let sum: number = data["a"].asNumber() + data["b"].asNumber();
    "#;
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

// ============================================================================
