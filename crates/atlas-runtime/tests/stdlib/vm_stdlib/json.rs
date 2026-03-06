use super::*;

// JSON stdlib tests (VM engine)
//
// Tests all 5 JSON functions via VM execution for parity verification
//
// Note: These tests use the same common::* helpers which test through the full pipeline,
// ensuring both interpreter and VM produce identical results.

// ============================================================================
// parse_json Tests
// ============================================================================

#[test]
fn test_parse_json_null() {
    let code = r#"
        let result: json = unwrap(parse_json("null"));
        typeof(result)
    "#;
    assert_eval_string(code, "null");
}

#[test]
fn test_parse_json_boolean_true() {
    // Should return JsonValue, test via typeof
    let code = r#"typeof(unwrap(parse_json("true")))"#;
    assert_eval_string(code, "boolean");
}

#[test]
fn test_parse_json_boolean_false() {
    let code = r#"typeof(unwrap(parse_json("false")))"#;
    assert_eval_string(code, "boolean");
}

#[test]
fn test_parse_json_number() {
    let code = r#"typeof(unwrap(parse_json("42")))"#;
    assert_eval_string(code, "number");
}

#[test]
fn test_parse_json_number_float() {
    let code = r#"typeof(unwrap(parse_json("3.14")))"#;
    assert_eval_string(code, "number");
}

#[test]
fn test_parse_json_number_negative() {
    let code = r#"typeof(unwrap(parse_json("-123")))"#;
    assert_eval_string(code, "number");
}

#[test]
fn test_parse_json_string() {
    let code = r#"typeof(unwrap(parse_json("\"hello\"")))"#;
    assert_eval_string(code, "string");
}

#[test]
fn test_parse_json_empty_string() {
    let code = r#"typeof(unwrap(parse_json("\"\"")))"#;
    assert_eval_string(code, "string");
}

#[test]
fn test_parse_json_array_empty() {
    let code = r#"typeof(unwrap(parse_json("[]")))"#;
    assert_eval_string(code, "array");
}

#[test]
fn test_parse_json_array_numbers() {
    let code = r#"typeof(unwrap(parse_json("[1,2,3]")))"#;
    assert_eval_string(code, "array");
}

#[test]
fn test_parse_json_array_mixed() {
    let code = r#"typeof(unwrap(parse_json("[1,\"two\",true,null]")))"#;
    assert_eval_string(code, "array");
}

#[test]
fn test_parse_json_array_nested() {
    let code = r#"typeof(unwrap(parse_json("[[1,2],[3,4]]")))"#;
    assert_eval_string(code, "array");
}

#[test]
fn test_parse_json_object_empty() {
    let code = r#"typeof(unwrap(parse_json("{}"  )))"#;
    assert_eval_string(code, "record");
}

#[test]
fn test_parse_json_object_simple() {
    let code = r#"typeof(unwrap(parse_json("{\"name\":\"Alice\",\"age\":30}")))"#;
    assert_eval_string(code, "record");
}

#[test]
fn test_parse_json_object_nested() {
    let code = r#"typeof(unwrap(parse_json("{\"user\":{\"name\":\"Bob\"}}")))"#;
    assert_eval_string(code, "record");
}

#[test]
fn test_parse_json_object_with_array() {
    let code = r#"typeof(unwrap(parse_json("{\"items\":[1,2,3]}")))"#;
    assert_eval_string(code, "record");
}

#[test]
fn test_parse_json_whitespace() {
    let code = r#"typeof(unwrap(parse_json("  { \"a\" : 1 }  ")))"#;
    assert_eval_string(code, "record");
}

#[test]
fn test_parse_json_unicode() {
    let code = r#"typeof(unwrap(parse_json("{\"emoji\":\"🎉\"}")))"#;
    assert_eval_string(code, "record");
}

// ============================================================================
// parse_json Error Tests
// ============================================================================

#[test]
fn test_parse_json_invalid_syntax() {
    let code = r#"parse_json("{invalid}")"#;
    assert_eval_result_err(code);
}

#[test]
fn test_parse_json_trailing_comma() {
    let code = r#"parse_json("[1,2,]")"#;
    assert_eval_result_err(code);
}

#[test]
fn test_parse_json_single_quote() {
    let code = r#"parse_json("{'key':'value'}")"#;
    assert_eval_result_err(code);
}

#[test]
fn test_parse_json_unquoted_keys() {
    let code = r#"parse_json("{key:\"value\"}")"#;
    assert_eval_result_err(code);
}

#[test]
fn test_parse_json_wrong_type() {
    let code = r#"parse_json(123)"#;
    assert_has_error(code);
}

// ============================================================================
// to_json Tests
// ============================================================================

#[test]
fn test_to_json_null() {
    let code = r#"to_json(null)"#;
    assert_eval_string(code, "null");
}

#[test]
fn test_to_json_bool_true() {
    let code = r#"to_json(true)"#;
    assert_eval_string(code, "true");
}

#[test]
fn test_to_json_bool_false() {
    let code = r#"to_json(false)"#;
    assert_eval_string(code, "false");
}

#[test]
fn test_to_json_number_int() {
    let code = r#"to_json(42)"#;
    assert_eval_string(code, "42");
}

#[test]
fn test_to_json_number_float() {
    let code = r#"to_json(3.14)"#;
    assert_eval_string(code, "3.14");
}

#[test]
fn test_to_json_number_negative() {
    let code = r#"to_json(-10)"#;
    assert_eval_string(code, "-10");
}

#[test]
fn test_to_json_number_zero() {
    let code = r#"to_json(0)"#;
    assert_eval_string(code, "0");
}

#[test]
fn test_to_json_string_simple() {
    let code = r#"to_json("hello")"#;
    assert_eval_string(code, r#""hello""#);
}

#[test]
fn test_to_json_string_empty() {
    let code = r#"to_json("")"#;
    assert_eval_string(code, r#""""#);
}

#[test]
fn test_to_json_string_with_quotes() {
    let code = r#"to_json("say \"hi\"")"#;
    assert_eval_string(code, r#""say \"hi\"""#);
}

#[test]
fn test_to_json_array_empty() {
    let code = r#"to_json([])"#;
    assert_eval_string(code, "[]");
}

#[test]
fn test_to_json_array_numbers() {
    let code = r#"to_json([1,2,3])"#;
    assert_eval_string(code, "[1,2,3]");
}

// Note: Mixed-type array test removed - Atlas enforces homogeneous arrays.
// For heterogeneous JSON arrays, use parse_json to create json values.

#[test]
fn test_to_json_array_nested() {
    let code = r#"to_json([[1,2],[3,4]])"#;
    assert_eval_string(code, "[[1,2],[3,4]]");
}

// ============================================================================
// to_json Error Tests
// ============================================================================

#[test]
fn test_to_json_nan_error() {
    let code = r#"to_json(0.0 / 0.0)"#;
    assert_has_error(code);
}

#[test]
fn test_to_json_infinity_error() {
    let code = r#"to_json(1.0 / 0.0)"#;
    assert_has_error(code);
}

#[test]
fn test_to_json_function_error() {
    let code = r#"
    fn test(): number { return 42; }
    to_json(test)
"#;
    assert_has_error(code);
}

// ============================================================================
// is_valid_json Tests
// ============================================================================

#[test]
fn test_is_valid_json_true_null() {
    let code = r#"is_valid_json("null")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_bool() {
    let code = r#"is_valid_json("true")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_number() {
    let code = r#"is_valid_json("42")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_string() {
    let code = r#"is_valid_json("\"hello\"")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_array() {
    let code = r#"is_valid_json("[1,2,3]")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_object() {
    let code = r#"is_valid_json("{\"key\":\"value\"}")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_false_invalid() {
    let code = r#"is_valid_json("{invalid}")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_valid_json_false_trailing_comma() {
    let code = r#"is_valid_json("[1,2,]")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_valid_json_false_empty() {
    let code = r#"is_valid_json("")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_valid_json_false_single_quote() {
    let code = r#"is_valid_json("{'a':1}")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_valid_json_wrong_type() {
    let code = r#"is_valid_json(123)"#;
    assert_has_error(code);
}

// ============================================================================
// prettify_json Tests
// ============================================================================

#[test]
fn test_prettify_json_object() {
    let code = r#"
    let compact: string = "{\"name\":\"Alice\",\"age\":30}";
    let pretty: string = prettify_json(compact, 2);
    includes(pretty, "  ")
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_prettify_json_array() {
    let code = r#"
    let compact: string = "[1,2,3]";
    let pretty: string = prettify_json(compact, 2);
    len(pretty) > len(compact)
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_prettify_json_indent_zero() {
    let code = r#"
    let compact: string = "{\"a\":1}";
    let pretty: string = prettify_json(compact, 0);
    typeof(pretty)
"#;
    assert_eval_string(code, "string");
}

#[test]
fn test_prettify_json_indent_four() {
    let code = r#"
    let compact: string = "{\"a\":1}";
    let pretty: string = prettify_json(compact, 4);
    includes(pretty, "    ")
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_prettify_json_nested() {
    let code = r#"
    let compact: string = "{\"user\":{\"name\":\"Bob\"}}";
    let pretty: string = prettify_json(compact, 2);
    len(pretty) > len(compact)
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_prettify_json_invalid() {
    let code = r#"prettify_json("{invalid}", 2)"#;
    assert_has_error(code);
}

#[test]
fn test_prettify_json_negative_indent() {
    let code = r#"prettify_json("{}", -1)"#;
    assert_has_error(code);
}

#[test]
fn test_prettify_json_float_indent() {
    let code = r#"prettify_json("{}", 2.5)"#;
    assert_has_error(code);
}

#[test]
fn test_prettify_json_wrong_type_first_arg() {
    let code = r#"prettify_json(123, 2)"#;
    assert_has_error(code);
}

#[test]
fn test_prettify_json_wrong_type_second_arg() {
    let code = r#"prettify_json("{}", "2")"#;
    assert_has_error(code);
}

// ============================================================================
// minify_json Tests
// ============================================================================

#[test]
fn test_minify_json_object() {
    let code = r#"
    let pretty: string = "{\n  \"name\": \"Alice\",\n  \"age\": 30\n}";
    let minified: string = minify_json(pretty);
    len(minified) < len(pretty)
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_minify_json_array() {
    let code = r#"
    let pretty: string = "[\n  1,\n  2,\n  3\n]";
    let minified: string = minify_json(pretty);
    len(minified) < len(pretty)
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_minify_json_no_whitespace() {
    let code = r#"
    let compact: string = "{\"a\":1}";
    let minified: string = minify_json(compact);
    typeof(minified)
"#;
    assert_eval_string(code, "string");
}

#[test]
fn test_minify_json_nested() {
    let code = r#"
    let pretty: string = "{\n  \"user\": {\n    \"name\": \"Bob\"\n  }\n}";
    let minified: string = minify_json(pretty);
    len(minified) < len(pretty)
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_minify_json_invalid() {
    let code = r#"minify_json("{invalid}")"#;
    assert_has_error(code);
}

#[test]
fn test_minify_json_wrong_type() {
    let code = r#"minify_json(123)"#;
    assert_has_error(code);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_parse_then_serialize() {
    let code = r#"
    let original: string = "{\"name\":\"Alice\",\"age\":30}";
    let parsed: json = unwrap(parse_json(original));
    let serialized: string = to_json(parsed);
    typeof(serialized)
"#;
    assert_eval_string(code, "string");
}

#[test]
fn test_prettify_then_minify() {
    let code = r#"
    let compact: string = "{\"a\":1,\"b\":2}";
    let pretty: string = prettify_json(compact, 2);
    let minified: string = minify_json(pretty);
    len(minified) < len(pretty)
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_validate_before_parse() {
    let code = r#"
    let json_str: string = "{\"valid\":true}";
    let valid: bool = is_valid_json(json_str);
    let parsed: json = unwrap(parse_json(json_str));
        valid && typeof(parsed) == "record"
    "#;
    assert_eval_bool(code, true);
}
