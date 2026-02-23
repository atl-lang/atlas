use super::*;
use pretty_assertions::assert_eq;

// VM stdlib tests (co-located to eliminate duplicate binary pairs)

// ============================================================================
// From vm_stdlib_string_tests.rs
// ============================================================================

// String stdlib tests (VM engine)
//
// Tests all 18 string functions via VM execution for parity verification
//
// Note: These tests use the same common::* helpers which test through the full pipeline,
// ensuring both interpreter and VM produce identical results.

// All tests are identical to stdlib_string_tests.rs to verify parity
// The common test helpers automatically test through both interpreter and VM

// ============================================================================
// Core Operations Tests
// ============================================================================

#[test]
fn test_split_basic() {
    let code = r#"
    let result: string[] = split("a,b,c", ",");
    len(result)
"#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_split_empty_separator() {
    let code = r#"
    let result: string[] = split("abc", "");
    len(result)
"#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_split_no_match() {
    let code = r#"
    let result: string[] = split("hello", ",");
    len(result)
"#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_split_unicode() {
    let code = r#"
    let result: string[] = split("🎉,🔥,✨", ",");
    len(result)
"#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_join_basic() {
    let code = r#"join(["a", "b", "c"], ",")"#;
    assert_eval_string(code, "a,b,c");
}

#[test]
fn test_join_empty_array() {
    let code = r#"join([], ",")"#;
    assert_eval_string(code, "");
}

#[test]
fn test_join_empty_separator() {
    let code = r#"join(["a", "b", "c"], "")"#;
    assert_eval_string(code, "abc");
}

#[test]
fn test_trim_basic() {
    let code = r#"trim("  hello  ")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_trim_unicode_whitespace() {
    let code = "trim(\"\u{00A0}hello\u{00A0}\")";
    assert_eval_string(code, "hello");
}

#[test]
fn test_trim_start() {
    let code = r#"trimStart("  hello")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_trim_end() {
    let code = r#"trimEnd("hello  ")"#;
    assert_eval_string(code, "hello");
}

// ============================================================================
// Search Operations Tests
// ============================================================================

#[test]
fn test_index_of_found() {
    let code = r#"indexOf("hello", "ll")"#;
    assert_eval_number(code, 2.0);
}

#[test]
fn test_index_of_not_found() {
    let code = r#"indexOf("hello", "x")"#;
    assert_eval_number(code, -1.0);
}

#[test]
fn test_index_of_empty_needle() {
    let code = r#"indexOf("hello", "")"#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_last_index_of_found() {
    let code = r#"lastIndexOf("hello", "l")"#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_last_index_of_not_found() {
    let code = r#"lastIndexOf("hello", "x")"#;
    assert_eval_number(code, -1.0);
}

#[test]
fn test_includes_found() {
    let code = r#"includes("hello", "ll")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_includes_not_found() {
    let code = r#"includes("hello", "x")"#;
    assert_eval_bool(code, false);
}

// ============================================================================
// Transformation Tests
// ============================================================================

#[test]
fn test_to_upper_case() {
    let code = r#"toUpperCase("hello")"#;
    assert_eval_string(code, "HELLO");
}

#[test]
fn test_to_upper_case_unicode() {
    let code = r#"toUpperCase("café")"#;
    assert_eval_string(code, "CAFÉ");
}

#[test]
fn test_to_lower_case() {
    let code = r#"toLowerCase("HELLO")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_to_lower_case_unicode() {
    let code = r#"toLowerCase("CAFÉ")"#;
    assert_eval_string(code, "café");
}

#[test]
fn test_substring_basic() {
    let code = r#"substring("hello", 1, 4)"#;
    assert_eval_string(code, "ell");
}

#[test]
fn test_substring_empty() {
    let code = r#"substring("hello", 2, 2)"#;
    assert_eval_string(code, "");
}

#[test]
fn test_substring_out_of_bounds() {
    let code = r#"substring("hello", 0, 100)"#;
    assert_has_error(code);
}

#[test]
fn test_char_at_basic() {
    let code = r#"charAt("hello", 0)"#;
    assert_eval_string(code, "h");
}

#[test]
fn test_char_at_unicode() {
    let code = r#"charAt("🎉🔥✨", 1)"#;
    assert_eval_string(code, "🔥");
}

#[test]
fn test_char_at_out_of_bounds() {
    let code = r#"charAt("hello", 10)"#;
    assert_has_error(code);
}

#[test]
fn test_repeat_basic() {
    let code = r#"repeat("ha", 3)"#;
    assert_eval_string(code, "hahaha");
}

#[test]
fn test_repeat_zero() {
    let code = r#"repeat("ha", 0)"#;
    assert_eval_string(code, "");
}

#[test]
fn test_repeat_negative() {
    let code = r#"repeat("ha", -1)"#;
    assert_has_error(code);
}

#[test]
fn test_replace_basic() {
    let code = r#"replace("hello", "l", "L")"#;
    assert_eval_string(code, "heLlo");
}

#[test]
fn test_replace_not_found() {
    let code = r#"replace("hello", "x", "y")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_replace_empty_search() {
    let code = r#"replace("hello", "", "x")"#;
    assert_eval_string(code, "hello");
}

// ============================================================================
// Formatting Tests
// ============================================================================

#[test]
fn test_pad_start_basic() {
    let code = r#"padStart("5", 3, "0")"#;
    assert_eval_string(code, "005");
}

#[test]
fn test_pad_start_already_long() {
    let code = r#"padStart("hello", 3, "0")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_pad_start_multichar_fill() {
    let code = r#"padStart("x", 5, "ab")"#;
    assert_eval_string(code, "ababx");
}

#[test]
fn test_pad_end_basic() {
    let code = r#"padEnd("5", 3, "0")"#;
    assert_eval_string(code, "500");
}

#[test]
fn test_pad_end_already_long() {
    let code = r#"padEnd("hello", 3, "0")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_starts_with_true() {
    let code = r#"startsWith("hello", "he")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_starts_with_false() {
    let code = r#"startsWith("hello", "x")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_starts_with_empty() {
    let code = r#"startsWith("hello", "")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_ends_with_true() {
    let code = r#"endsWith("hello", "lo")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_ends_with_false() {
    let code = r#"endsWith("hello", "x")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_ends_with_empty() {
    let code = r#"endsWith("hello", "")"#;
    assert_eval_bool(code, true);
}

// ============================================================================
// From vm_stdlib_json_tests.rs
// ============================================================================

// JSON stdlib tests (VM engine)
//
// Tests all 5 JSON functions via VM execution for parity verification
//
// Note: These tests use the same common::* helpers which test through the full pipeline,
// ensuring both interpreter and VM produce identical results.

// ============================================================================
// parseJSON Tests
// ============================================================================

#[test]
fn test_parse_json_null() {
    let code = r#"
    let result: json = parseJSON("null");
    typeof(result)
"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_boolean_true() {
    // Should return JsonValue, test via typeof
    let code = r#"typeof(parseJSON("true"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_boolean_false() {
    let code = r#"typeof(parseJSON("false"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_number() {
    let code = r#"typeof(parseJSON("42"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_number_float() {
    let code = r#"typeof(parseJSON("3.14"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_number_negative() {
    let code = r#"typeof(parseJSON("-123"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_string() {
    let code = r#"typeof(parseJSON("\"hello\""))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_empty_string() {
    let code = r#"typeof(parseJSON("\"\""))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_array_empty() {
    let code = r#"typeof(parseJSON("[]"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_array_numbers() {
    let code = r#"typeof(parseJSON("[1,2,3]"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_array_mixed() {
    let code = r#"typeof(parseJSON("[1,\"two\",true,null]"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_array_nested() {
    let code = r#"typeof(parseJSON("[[1,2],[3,4]]"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_object_empty() {
    let code = r#"typeof(parseJSON("{}"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_object_simple() {
    let code = r#"typeof(parseJSON("{\"name\":\"Alice\",\"age\":30}"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_object_nested() {
    let code = r#"typeof(parseJSON("{\"user\":{\"name\":\"Bob\"}}"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_object_with_array() {
    let code = r#"typeof(parseJSON("{\"items\":[1,2,3]}"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_whitespace() {
    let code = r#"typeof(parseJSON("  { \"a\" : 1 }  "))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_unicode() {
    let code = r#"typeof(parseJSON("{\"emoji\":\"🎉\"}"))"#;
    assert_eval_string(code, "json");
}

// ============================================================================
// parseJSON Error Tests
// ============================================================================

#[test]
fn test_parse_json_invalid_syntax() {
    let code = r#"parseJSON("{invalid}")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_json_trailing_comma() {
    let code = r#"parseJSON("[1,2,]")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_json_single_quote() {
    let code = r#"parseJSON("{'key':'value'}")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_json_unquoted_keys() {
    let code = r#"parseJSON("{key:\"value\"}")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_json_wrong_type() {
    let code = r#"parseJSON(123)"#;
    assert_has_error(code);
}

// ============================================================================
// toJSON Tests
// ============================================================================

#[test]
fn test_to_json_null() {
    let code = r#"toJSON(null)"#;
    assert_eval_string(code, "null");
}

#[test]
fn test_to_json_bool_true() {
    let code = r#"toJSON(true)"#;
    assert_eval_string(code, "true");
}

#[test]
fn test_to_json_bool_false() {
    let code = r#"toJSON(false)"#;
    assert_eval_string(code, "false");
}

#[test]
fn test_to_json_number_int() {
    let code = r#"toJSON(42)"#;
    assert_eval_string(code, "42");
}

#[test]
fn test_to_json_number_float() {
    let code = r#"toJSON(3.14)"#;
    assert_eval_string(code, "3.14");
}

#[test]
fn test_to_json_number_negative() {
    let code = r#"toJSON(-10)"#;
    assert_eval_string(code, "-10");
}

#[test]
fn test_to_json_number_zero() {
    let code = r#"toJSON(0)"#;
    assert_eval_string(code, "0");
}

#[test]
fn test_to_json_string_simple() {
    let code = r#"toJSON("hello")"#;
    assert_eval_string(code, r#""hello""#);
}

#[test]
fn test_to_json_string_empty() {
    let code = r#"toJSON("")"#;
    assert_eval_string(code, r#""""#);
}

#[test]
fn test_to_json_string_with_quotes() {
    let code = r#"toJSON("say \"hi\"")"#;
    assert_eval_string(code, r#""say \"hi\"""#);
}

#[test]
fn test_to_json_array_empty() {
    let code = r#"toJSON([])"#;
    assert_eval_string(code, "[]");
}

#[test]
fn test_to_json_array_numbers() {
    let code = r#"toJSON([1,2,3])"#;
    assert_eval_string(code, "[1,2,3]");
}

// Note: Mixed-type array test removed - Atlas enforces homogeneous arrays.
// For heterogeneous JSON arrays, use parseJSON to create json values.

#[test]
fn test_to_json_array_nested() {
    let code = r#"toJSON([[1,2],[3,4]])"#;
    assert_eval_string(code, "[[1,2],[3,4]]");
}

// ============================================================================
// toJSON Error Tests
// ============================================================================

#[test]
fn test_to_json_nan_error() {
    let code = r#"toJSON(0.0 / 0.0)"#;
    assert_has_error(code);
}

#[test]
fn test_to_json_infinity_error() {
    let code = r#"toJSON(1.0 / 0.0)"#;
    assert_has_error(code);
}

#[test]
fn test_to_json_function_error() {
    let code = r#"
    fn test(): number { return 42; }
    toJSON(test)
"#;
    assert_has_error(code);
}

// ============================================================================
// isValidJSON Tests
// ============================================================================

#[test]
fn test_is_valid_json_true_null() {
    let code = r#"isValidJSON("null")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_bool() {
    let code = r#"isValidJSON("true")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_number() {
    let code = r#"isValidJSON("42")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_string() {
    let code = r#"isValidJSON("\"hello\"")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_array() {
    let code = r#"isValidJSON("[1,2,3]")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_object() {
    let code = r#"isValidJSON("{\"key\":\"value\"}")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_false_invalid() {
    let code = r#"isValidJSON("{invalid}")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_valid_json_false_trailing_comma() {
    let code = r#"isValidJSON("[1,2,]")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_valid_json_false_empty() {
    let code = r#"isValidJSON("")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_valid_json_false_single_quote() {
    let code = r#"isValidJSON("{'a':1}")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_valid_json_wrong_type() {
    let code = r#"isValidJSON(123)"#;
    assert_has_error(code);
}

// ============================================================================
// prettifyJSON Tests
// ============================================================================

#[test]
fn test_prettify_json_object() {
    let code = r#"
    let compact: string = "{\"name\":\"Alice\",\"age\":30}";
    let pretty: string = prettifyJSON(compact, 2);
    includes(pretty, "  ")
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_prettify_json_array() {
    let code = r#"
    let compact: string = "[1,2,3]";
    let pretty: string = prettifyJSON(compact, 2);
    len(pretty) > len(compact)
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_prettify_json_indent_zero() {
    let code = r#"
    let compact: string = "{\"a\":1}";
    let pretty: string = prettifyJSON(compact, 0);
    typeof(pretty)
"#;
    assert_eval_string(code, "string");
}

#[test]
fn test_prettify_json_indent_four() {
    let code = r#"
    let compact: string = "{\"a\":1}";
    let pretty: string = prettifyJSON(compact, 4);
    includes(pretty, "    ")
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_prettify_json_nested() {
    let code = r#"
    let compact: string = "{\"user\":{\"name\":\"Bob\"}}";
    let pretty: string = prettifyJSON(compact, 2);
    len(pretty) > len(compact)
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_prettify_json_invalid() {
    let code = r#"prettifyJSON("{invalid}", 2)"#;
    assert_has_error(code);
}

#[test]
fn test_prettify_json_negative_indent() {
    let code = r#"prettifyJSON("{}", -1)"#;
    assert_has_error(code);
}

#[test]
fn test_prettify_json_float_indent() {
    let code = r#"prettifyJSON("{}", 2.5)"#;
    assert_has_error(code);
}

#[test]
fn test_prettify_json_wrong_type_first_arg() {
    let code = r#"prettifyJSON(123, 2)"#;
    assert_has_error(code);
}

#[test]
fn test_prettify_json_wrong_type_second_arg() {
    let code = r#"prettifyJSON("{}", "2")"#;
    assert_has_error(code);
}

// ============================================================================
// minifyJSON Tests
// ============================================================================

#[test]
fn test_minify_json_object() {
    let code = r#"
    let pretty: string = "{\n  \"name\": \"Alice\",\n  \"age\": 30\n}";
    let minified: string = minifyJSON(pretty);
    len(minified) < len(pretty)
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_minify_json_array() {
    let code = r#"
    let pretty: string = "[\n  1,\n  2,\n  3\n]";
    let minified: string = minifyJSON(pretty);
    len(minified) < len(pretty)
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_minify_json_no_whitespace() {
    let code = r#"
    let compact: string = "{\"a\":1}";
    let minified: string = minifyJSON(compact);
    typeof(minified)
"#;
    assert_eval_string(code, "string");
}

#[test]
fn test_minify_json_nested() {
    let code = r#"
    let pretty: string = "{\n  \"user\": {\n    \"name\": \"Bob\"\n  }\n}";
    let minified: string = minifyJSON(pretty);
    len(minified) < len(pretty)
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_minify_json_invalid() {
    let code = r#"minifyJSON("{invalid}")"#;
    assert_has_error(code);
}

#[test]
fn test_minify_json_wrong_type() {
    let code = r#"minifyJSON(123)"#;
    assert_has_error(code);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_parse_then_serialize() {
    let code = r#"
    let original: string = "{\"name\":\"Alice\",\"age\":30}";
    let parsed: json = parseJSON(original);
    let serialized: string = toJSON(parsed);
    typeof(serialized)
"#;
    assert_eval_string(code, "string");
}

#[test]
fn test_prettify_then_minify() {
    let code = r#"
    let compact: string = "{\"a\":1,\"b\":2}";
    let pretty: string = prettifyJSON(compact, 2);
    let minified: string = minifyJSON(pretty);
    len(minified) < len(pretty)
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_validate_before_parse() {
    let code = r#"
    let json_str: string = "{\"valid\":true}";
    let valid: bool = isValidJSON(json_str);
    let parsed: json = parseJSON(json_str);
    valid && typeof(parsed) == "json"
"#;
    assert_eval_bool(code, true);
}

// ============================================================================
// From vm_stdlib_io_tests.rs
// ============================================================================

// Standard library file I/O tests (VM/Bytecode)
//
// Tests file and directory operations via bytecode execution for VM parity.

// Helper to execute Atlas source via bytecode
fn execute_with_io(source: &str, temp_dir: &TempDir) -> Result<atlas_runtime::Value, String> {
    use atlas_runtime::{Binder, Compiler, Lexer, Parser, TypeChecker, VM};

    // Parse and compile
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    // Execute with security context
    let mut security = SecurityContext::new();
    security.grant_filesystem_read(temp_dir.path(), true);
    security.grant_filesystem_write(temp_dir.path(), true);

    let mut vm = VM::new(bytecode);
    vm.run(&security)
        .map(|opt| opt.unwrap_or(atlas_runtime::Value::Null))
        .map_err(|e| format!("{:?}", e))
}

// ============================================================================
// VM parity tests - all use pattern: let result = func(); result;
// ============================================================================

#[test]
fn vm_test_read_file_basic() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Hello, VM!").unwrap();

    let code = format!(r#"let x = readFile("{}"); x;"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::String(_)));
}

#[test]
fn vm_test_write_file_basic() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("output.txt");

    let code = format!(
        r#"writeFile("{}", "VM content");"#,
        path_for_atlas(&test_file)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "VM content");
}

#[test]
fn vm_test_append_file_basic() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("append.txt");
    fs::write(&test_file, "line1\n").unwrap();

    let code = format!(
        r#"appendFile("{}", "line2\n");"#,
        path_for_atlas(&test_file)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "line1\nline2\n");
}

#[test]
fn vm_test_file_exists_true() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("exists.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(
        r#"let result = fileExists("{}"); result;"#,
        path_for_atlas(&test_file)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(matches!(value, atlas_runtime::Value::Bool(true)));
}

#[test]
fn vm_test_file_exists_false() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("does_not_exist.txt");

    let code = format!(
        r#"let result = fileExists("{}"); result;"#,
        path_for_atlas(&nonexistent)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Bool(false)));
}

#[test]
fn vm_test_read_dir_basic() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("file1.txt"), "").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "").unwrap();

    let code = format!(
        r#"let result = readDir("{}"); result;"#,
        path_for_atlas(temp_dir.path())
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Array(_)));
}

#[test]
fn vm_test_create_dir_basic() {
    let temp_dir = TempDir::new().unwrap();
    let new_dir = temp_dir.path().join("newdir");

    let code = format!(r#"createDir("{}");"#, path_for_atlas(&new_dir));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(new_dir.exists());
    assert!(new_dir.is_dir());
}

#[test]
fn vm_test_create_dir_nested() {
    let temp_dir = TempDir::new().unwrap();
    let nested_dir = temp_dir.path().join("a/b/c");

    let code = format!(r#"createDir("{}");"#, path_for_atlas(&nested_dir));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(nested_dir.exists());
}

#[test]
fn vm_test_remove_file_basic() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("remove.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(r#"removeFile("{}");"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(!test_file.exists());
}

#[test]
fn vm_test_remove_dir_basic() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("rmdir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"removeDir("{}");"#, path_for_atlas(&test_dir));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(!test_dir.exists());
}

#[test]
fn vm_test_file_info_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("info.txt");
    fs::write(&test_file, "test content").unwrap();

    let code = format!(
        r#"let result = fileInfo("{}"); result;"#,
        path_for_atlas(&test_file)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        atlas_runtime::Value::JsonValue(_)
    ));
}

#[test]
fn vm_test_path_join_basic() {
    let temp_dir = TempDir::new().unwrap();
    let code = r#"let result = pathJoin("a", "b", "c"); result;"#;
    let result = execute_with_io(code, &temp_dir);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::String(_)));
}

// ============================================================================
// Additional VM parity tests to match interpreter coverage
// ============================================================================

#[test]
fn vm_test_read_file_utf8() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("utf8.txt");
    fs::write(&test_file, "Hello 你好 🎉").unwrap();

    let code = format!(r#"let x = readFile("{}"); x;"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
}

#[test]
fn vm_test_read_file_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("does_not_exist.txt");

    let code = format!(r#"readFile("{}");"#, path_for_atlas(&nonexistent));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to resolve path"));
}

#[test]
fn vm_test_read_file_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("secret.txt");
    fs::write(&test_file, "secret").unwrap();

    // Execute without granting permissions
    let mut lexer =
        atlas_runtime::Lexer::new(format!(r#"readFile("{}");"#, path_for_atlas(&test_file)));
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = atlas_runtime::Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = atlas_runtime::TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = atlas_runtime::Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let security = SecurityContext::new(); // No permissions
    let mut vm = atlas_runtime::VM::new(bytecode);
    let result = vm.run(&security);

    assert!(result.is_err());
}

#[test]
fn vm_test_write_file_overwrite() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("overwrite.txt");
    fs::write(&test_file, "original").unwrap();

    let code = format!(
        r#"writeFile("{}", "new content");"#,
        path_for_atlas(&test_file)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "new content");
}

#[test]
fn vm_test_write_file_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("output.txt");

    // Execute without granting permissions
    let mut lexer = atlas_runtime::Lexer::new(format!(
        r#"writeFile("{}", "content");"#,
        path_for_atlas(&test_file)
    ));
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = atlas_runtime::Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = atlas_runtime::TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = atlas_runtime::Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let security = SecurityContext::new(); // No permissions
    let mut vm = atlas_runtime::VM::new(bytecode);
    let result = vm.run(&security);

    assert!(result.is_err());
}

#[test]
fn vm_test_append_file_create_if_not_exists() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("new.txt");

    let code = format!(
        r#"appendFile("{}", "content");"#,
        path_for_atlas(&test_file)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "content");
}

#[test]
fn vm_test_read_dir_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent_dir");

    let code = format!(r#"readDir("{}");"#, path_for_atlas(&nonexistent));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_err());
}

#[test]
fn vm_test_remove_file_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("does_not_exist.txt");

    let code = format!(r#"removeFile("{}");"#, path_for_atlas(&nonexistent));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_err());
}

#[test]
fn vm_test_remove_dir_not_empty() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("notempty");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("file.txt"), "").unwrap();

    let code = format!(r#"removeDir("{}");"#, path_for_atlas(&test_dir));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to remove directory"));
}

#[test]
fn vm_test_file_info_directory() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("infodir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(
        r#"let result = fileInfo("{}"); result;"#,
        path_for_atlas(&test_dir)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
}

#[test]
fn vm_test_path_join_single() {
    let temp_dir = TempDir::new().unwrap();
    let code = r#"let result = pathJoin("single"); result;"#;
    let result = execute_with_io(code, &temp_dir);

    assert!(result.is_ok());
}

#[test]
fn vm_test_path_join_no_args() {
    let temp_dir = TempDir::new().unwrap();
    let code = r#"pathJoin();"#;
    let result = execute_with_io(code, &temp_dir);

    assert!(result.is_err());
}

// ============================================================================
// VM - Additional readFile tests
// ============================================================================

#[test]
fn vm_test_read_file_empty() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("empty.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(r#"let x = readFile("{}"); x;"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    if let atlas_runtime::Value::String(s) = result.unwrap() {
        assert_eq!(s.as_str(), "");
    } else {
        panic!("Expected string");
    }
}

#[test]
fn vm_test_read_file_invalid_utf8() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("binary.bin");
    fs::write(&test_file, [0xFF, 0xFE, 0xFD]).unwrap();

    let code = format!(r#"readFile("{}");"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_err());
}

#[test]
fn vm_test_read_file_multiline() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("multiline.txt");
    let content = "line1\nline2\nline3\n";
    fs::write(&test_file, content).unwrap();

    let code = format!(r#"let x = readFile("{}"); x;"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    if let atlas_runtime::Value::String(s) = result.unwrap() {
        assert_eq!(s.as_str(), content);
    } else {
        panic!("Expected string");
    }
}

#[test]
fn vm_test_read_file_large() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("large.txt");
    let content = "x".repeat(10000);
    fs::write(&test_file, &content).unwrap();

    let code = format!(r#"let x = readFile("{}"); x;"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    if let atlas_runtime::Value::String(s) = result.unwrap() {
        assert_eq!(s.len(), 10000);
    } else {
        panic!("Expected string");
    }
}

#[test]
fn vm_test_read_file_with_bom() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("bom.txt");
    let mut content = vec![0xEF, 0xBB, 0xBF];
    content.extend_from_slice(b"Hello");
    fs::write(&test_file, content).unwrap();

    let code = format!(r#"let x = readFile("{}"); x;"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
}

// ============================================================================
// VM - Additional writeFile tests
// ============================================================================

#[test]
fn vm_test_write_file_empty() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("empty_write.txt");

    let code = format!(r#"writeFile("{}", "");"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "");
}

#[test]
fn vm_test_write_file_unicode() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("unicode.txt");
    let content = "Hello 世界 🌍";

    let code = format!(
        r#"writeFile("{}", "{}");"#,
        path_for_atlas(&test_file),
        content
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, content);
}

#[test]
fn vm_test_write_file_newlines() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("newlines.txt");

    let code = format!(
        r#"writeFile("{}", "line1\nline2\n");"#,
        path_for_atlas(&test_file)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "line1\nline2\n");
}

#[test]
fn vm_test_write_file_creates_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("new_file.txt");
    assert!(!test_file.exists());

    let code = format!(r#"writeFile("{}", "content");"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(test_file.exists());
}

// ============================================================================
// VM - Additional appendFile tests
// ============================================================================

#[test]
fn vm_test_append_file_multiple() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("multi_append.txt");
    fs::write(&test_file, "start\n").unwrap();

    let code = format!(
        r#"appendFile("{}", "line1\n"); appendFile("{}", "line2\n");"#,
        path_for_atlas(&test_file),
        path_for_atlas(&test_file)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "start\nline1\nline2\n");
}

#[test]
fn vm_test_append_file_empty_content() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("append_empty.txt");
    fs::write(&test_file, "base").unwrap();

    let code = format!(r#"appendFile("{}", "");"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "base");
}

#[test]
fn vm_test_append_file_permission_denied() {
    use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("append_denied.txt");

    let code = format!(
        r#"appendFile("{}", "content");"#,
        path_for_atlas(&test_file)
    );

    let mut lexer = Lexer::new(code);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let security = SecurityContext::new();
    let mut vm = VM::new(bytecode);
    let result = vm.run(&security);

    assert!(result.is_err());
}

// ============================================================================
// VM - Additional fileExists tests
// ============================================================================

#[test]
fn vm_test_file_exists_directory() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("exists_dir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(
        r#"let result = fileExists("{}"); result;"#,
        path_for_atlas(&test_dir)
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Bool(true)));
}

#[test]
fn vm_test_file_exists_no_permission_check() {
    use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("exists_test.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(
        r#"let x = fileExists("{}"); x;"#,
        path_for_atlas(&test_file)
    );

    let mut lexer = Lexer::new(code);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let security = SecurityContext::new();
    let mut vm = VM::new(bytecode);
    let result = vm.run(&security);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        Some(atlas_runtime::Value::Bool(true))
    ));
}

// ============================================================================
// VM - Additional readDir tests
// ============================================================================

#[test]
fn vm_test_read_dir_empty() {
    let temp_dir = TempDir::new().unwrap();
    let empty_dir = temp_dir.path().join("empty");
    fs::create_dir(&empty_dir).unwrap();

    let code = format!(r#"let x = readDir("{}"); x;"#, path_for_atlas(&empty_dir));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    if let atlas_runtime::Value::Array(arr) = result.unwrap() {
        assert_eq!(arr.len(), 0);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn vm_test_read_dir_mixed_contents() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("file.txt"), "").unwrap();
    fs::create_dir(temp_dir.path().join("subdir")).unwrap();

    let code = format!(
        r#"let x = readDir("{}"); x;"#,
        path_for_atlas(temp_dir.path())
    );
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    if let atlas_runtime::Value::Array(arr) = result.unwrap() {
        assert_eq!(arr.len(), 2);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn vm_test_read_dir_permission_denied() {
    use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("dir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"readDir("{}");"#, path_for_atlas(&test_dir));

    let mut lexer = Lexer::new(code);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let security = SecurityContext::new();
    let mut vm = VM::new(bytecode);
    let result = vm.run(&security);

    assert!(result.is_err());
}

// ============================================================================
// VM - Additional createDir tests
// ============================================================================

#[test]
fn vm_test_create_dir_already_exists() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("already_exists");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"createDir("{}");"#, path_for_atlas(&test_dir));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
}

#[test]
fn vm_test_create_dir_permission_denied() {
    use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

    let temp_dir = TempDir::new().unwrap();
    let new_dir = temp_dir.path().join("denied");

    let code = format!(r#"createDir("{}");"#, path_for_atlas(&new_dir));

    let mut lexer = Lexer::new(code);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let security = SecurityContext::new();
    let mut vm = VM::new(bytecode);
    let result = vm.run(&security);

    assert!(result.is_err());
}

// ============================================================================
// VM - Additional removeFile tests
// ============================================================================

#[test]
fn vm_test_remove_file_is_directory() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("is_dir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"removeFile("{}");"#, path_for_atlas(&test_dir));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_err());
}

#[test]
fn vm_test_remove_file_permission_denied() {
    use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("remove_denied.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(r#"removeFile("{}");"#, path_for_atlas(&test_file));

    let mut lexer = Lexer::new(code);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let security = SecurityContext::new();
    let mut vm = VM::new(bytecode);
    let result = vm.run(&security);

    assert!(result.is_err());
}

// ============================================================================
// VM - Additional removeDir tests
// ============================================================================

#[test]
fn vm_test_remove_dir_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("not_found");

    let code = format!(r#"removeDir("{}");"#, path_for_atlas(&nonexistent));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_err());
}

#[test]
fn vm_test_remove_dir_is_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("is_file.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(r#"removeDir("{}");"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_err());
}

#[test]
fn vm_test_remove_dir_permission_denied() {
    use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("remove_denied");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"removeDir("{}");"#, path_for_atlas(&test_dir));

    let mut lexer = Lexer::new(code);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let security = SecurityContext::new();
    let mut vm = VM::new(bytecode);
    let result = vm.run(&security);

    assert!(result.is_err());
}

// ============================================================================
// VM - Additional fileInfo tests
// ============================================================================

#[test]
fn vm_test_file_info_size_check() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("info_fields.txt");
    fs::write(&test_file, "12345").unwrap();

    let code = format!(r#"let x = fileInfo("{}"); x;"#, path_for_atlas(&test_file));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        atlas_runtime::Value::JsonValue(_)
    ));
}

#[test]
fn vm_test_file_info_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("not_found.txt");

    let code = format!(r#"fileInfo("{}");"#, path_for_atlas(&nonexistent));
    let result = execute_with_io(&code, &temp_dir);

    assert!(result.is_err());
}

#[test]
fn vm_test_file_info_permission_denied() {
    use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("info_denied.txt");
    fs::write(&test_file, "test").unwrap();

    let code = format!(r#"fileInfo("{}");"#, path_for_atlas(&test_file));

    let mut lexer = Lexer::new(code);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let security = SecurityContext::new();
    let mut vm = VM::new(bytecode);
    let result = vm.run(&security);

    assert!(result.is_err());
}

// ============================================================================
// VM - Additional pathJoin tests
// ============================================================================

#[test]
fn vm_test_path_join_many_parts() {
    let temp_dir = TempDir::new().unwrap();
    let code = r#"let x = pathJoin("a", "b", "c", "d", "e"); x;"#;
    let result = execute_with_io(code, &temp_dir);

    assert!(result.is_ok());
    if let atlas_runtime::Value::String(path) = result.unwrap() {
        assert!(path.contains("a"));
        assert!(path.contains("e"));
    } else {
        panic!("Expected string");
    }
}

#[test]
fn vm_test_path_join_empty_parts() {
    let temp_dir = TempDir::new().unwrap();
    let code = r#"let x = pathJoin("", "a", ""); x;"#;
    let result = execute_with_io(code, &temp_dir);

    assert!(result.is_ok());
}

#[test]
fn vm_test_path_join_absolute_path() {
    let temp_dir = TempDir::new().unwrap();
    let code = r#"let x = pathJoin("/absolute", "path"); x;"#;
    let result = execute_with_io(code, &temp_dir);

    assert!(result.is_ok());
    if let atlas_runtime::Value::String(path) = result.unwrap() {
        assert!(path.starts_with("/") || path.starts_with("\\"));
    } else {
        panic!("Expected string");
    }
}

// ============================================================================
// From vm_stdlib_types_tests.rs
// ============================================================================

// Type checking and conversion stdlib tests (VM engine)
//
// Tests all 12 type utility functions via VM execution for parity verification
//
// Note: These tests use the same common::* helpers which test through the full pipeline,
// ensuring both interpreter and VM produce identical results.

// ============================================================================
// typeof Tests
// ============================================================================

#[test]
fn test_typeof_null() {
    let code = r#"typeof(null)"#;
    assert_eval_string(code, "null");
}

#[test]
fn test_typeof_bool_true() {
    let code = r#"typeof(true)"#;
    assert_eval_string(code, "bool");
}

#[test]
fn test_typeof_bool_false() {
    let code = r#"typeof(false)"#;
    assert_eval_string(code, "bool");
}

#[test]
fn test_typeof_number_positive() {
    let code = r#"typeof(42)"#;
    assert_eval_string(code, "number");
}

#[test]
fn test_typeof_number_negative() {
    let code = r#"typeof(-10)"#;
    assert_eval_string(code, "number");
}

#[test]
fn test_typeof_number_float() {
    let code = r#"typeof(3.5)"#;
    assert_eval_string(code, "number");
}

// NaN/Infinity tests removed: division by zero is a runtime error in Atlas

#[test]
fn test_typeof_string_nonempty() {
    let code = r#"typeof("hello")"#;
    assert_eval_string(code, "string");
}

#[test]
fn test_typeof_string_empty() {
    let code = r#"typeof("")"#;
    assert_eval_string(code, "string");
}

#[test]
fn test_typeof_array_nonempty() {
    let code = r#"typeof([1,2,3])"#;
    assert_eval_string(code, "array");
}

#[test]
fn test_typeof_array_empty() {
    let code = r#"typeof([])"#;
    assert_eval_string(code, "array");
}

// Function reference tests removed: not yet fully supported

#[test]
fn test_typeof_json() {
    let code = r#"typeof(parseJSON("null"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_typeof_option() {
    let code = r#"typeof(Some(42))"#;
    assert_eval_string(code, "option");
}

#[test]
fn test_typeof_result() {
    let code = r#"typeof(Ok(42))"#;
    assert_eval_string(code, "result");
}

// ============================================================================
// Type Guard Tests
// ============================================================================

#[test]
fn test_is_string_true() {
    let code = r#"isString("hello")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_string_false_number() {
    let code = r#"isString(42)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_string_false_null() {
    let code = r#"isString(null)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_number_true_int() {
    let code = r#"isNumber(42)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_number_true_float() {
    let code = r#"isNumber(3.5)"#;
    assert_eval_bool(code, true);
}

// Removed: NaN test (division by zero is error)

#[test]
fn test_is_number_false_string() {
    let code = r#"isNumber("42")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_bool_true() {
    let code = r#"isBool(true)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_bool_false() {
    let code = r#"isBool(false)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_bool_false_number() {
    let code = r#"isBool(1)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_null_true() {
    let code = r#"isNull(null)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_null_false() {
    let code = r#"isNull(0)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_array_true() {
    let code = r#"isArray([1,2,3])"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_array_true_empty() {
    let code = r#"isArray([])"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_array_false() {
    let code = r#"isArray("not array")"#;
    assert_eval_bool(code, false);
}

// Function reference tests removed: not yet fully supported

#[test]
fn test_is_function_false() {
    let code = r#"isFunction(42)"#;
    assert_eval_bool(code, false);
}

// ============================================================================
// toString Tests
// ============================================================================

#[test]
fn test_to_string_null() {
    let code = r#"toString(null)"#;
    assert_eval_string(code, "null");
}

#[test]
fn test_to_string_bool_true() {
    let code = r#"toString(true)"#;
    assert_eval_string(code, "true");
}

#[test]
fn test_to_string_bool_false() {
    let code = r#"toString(false)"#;
    assert_eval_string(code, "false");
}

#[test]
fn test_to_string_number_int() {
    let code = r#"toString(42)"#;
    assert_eval_string(code, "42");
}

#[test]
fn test_to_string_number_float() {
    let code = r#"toString(3.5)"#;
    assert_eval_string(code, "3.5");
}

#[test]
fn test_to_string_number_negative() {
    let code = r#"toString(-10)"#;
    assert_eval_string(code, "-10");
}

#[test]
fn test_to_string_number_zero() {
    let code = r#"toString(0)"#;
    assert_eval_string(code, "0");
}

// NaN/Infinity toString tests removed: division by zero is error

#[test]
fn test_to_string_string_identity() {
    let code = r#"toString("hello")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_to_string_string_empty() {
    let code = r#"toString("")"#;
    assert_eval_string(code, "");
}

#[test]
fn test_to_string_array() {
    let code = r#"toString([1,2,3])"#;
    assert_eval_string(code, "[Array]");
}

// Function toString test removed: not yet fully supported

#[test]
fn test_to_string_json() {
    let code = r#"toString(parseJSON("null"))"#;
    assert_eval_string(code, "[JSON]");
}

// ============================================================================
// toNumber Tests
// ============================================================================

#[test]
fn test_to_number_number_identity() {
    let code = r#"toNumber(42)"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_to_number_bool_true() {
    let code = r#"toNumber(true)"#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_to_number_bool_false() {
    let code = r#"toNumber(false)"#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_to_number_string_int() {
    let code = r#"toNumber("42")"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_to_number_string_float() {
    let code = r#"toNumber("3.5")"#;
    assert_eval_number(code, 3.5);
}

#[test]
fn test_to_number_string_negative() {
    let code = r#"toNumber("-10")"#;
    assert_eval_number(code, -10.0);
}

#[test]
fn test_to_number_string_whitespace() {
    let code = r#"toNumber("  42  ")"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_to_number_string_scientific() {
    let code = r#"toNumber("1e10")"#;
    assert_eval_number(code, 1e10);
}

#[test]
fn test_to_number_string_empty_error() {
    let code = r#"toNumber("")"#;
    assert_has_error(code);
}

#[test]
fn test_to_number_string_invalid_error() {
    let code = r#"toNumber("hello")"#;
    assert_has_error(code);
}

#[test]
fn test_to_number_null_error() {
    let code = r#"toNumber(null)"#;
    assert_has_error(code);
}

#[test]
fn test_to_number_array_error() {
    let code = r#"toNumber([1,2,3])"#;
    assert_has_error(code);
}

// ============================================================================
// toBool Tests
// ============================================================================

#[test]
fn test_to_bool_bool_identity_true() {
    let code = r#"toBool(true)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_bool_identity_false() {
    let code = r#"toBool(false)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_to_bool_number_zero_false() {
    let code = r#"toBool(0)"#;
    assert_eval_bool(code, false);
}

// NaN toBool test removed: division by zero is error

#[test]
fn test_to_bool_number_positive_true() {
    let code = r#"toBool(42)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_number_negative_true() {
    let code = r#"toBool(-10)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_string_empty_false() {
    let code = r#"toBool("")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_to_bool_string_nonempty_true() {
    let code = r#"toBool("hello")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_string_space_true() {
    let code = r#"toBool(" ")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_null_false() {
    let code = r#"toBool(null)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_to_bool_array_true() {
    let code = r#"toBool([1,2,3])"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_array_empty_true() {
    let code = r#"toBool([])"#;
    assert_eval_bool(code, true);
}

// Function toBool test removed: not yet fully supported

// ============================================================================
// parseInt Tests
// ============================================================================

#[test]
fn test_parse_int_decimal() {
    let code = r#"parseInt("42", 10)"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_int_decimal_negative() {
    let code = r#"parseInt("-10", 10)"#;
    assert_eval_number(code, -10.0);
}

#[test]
fn test_parse_int_binary() {
    let code = r#"parseInt("1010", 2)"#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_parse_int_octal() {
    let code = r#"parseInt("17", 8)"#;
    assert_eval_number(code, 15.0);
}

#[test]
fn test_parse_int_hex() {
    let code = r#"parseInt("FF", 16)"#;
    assert_eval_number(code, 255.0);
}

#[test]
fn test_parse_int_hex_lowercase() {
    let code = r#"parseInt("ff", 16)"#;
    assert_eval_number(code, 255.0);
}

#[test]
fn test_parse_int_radix_36() {
    let code = r#"parseInt("Z", 36)"#;
    assert_eval_number(code, 35.0);
}

#[test]
fn test_parse_int_plus_sign() {
    let code = r#"parseInt("+42", 10)"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_int_whitespace() {
    let code = r#"parseInt("  42  ", 10)"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_int_radix_too_low() {
    let code = r#"parseInt("42", 1)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_radix_too_high() {
    let code = r#"parseInt("42", 37)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_radix_float() {
    let code = r#"parseInt("42", 10.5)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_empty_string() {
    let code = r#"parseInt("", 10)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_invalid_digit() {
    let code = r#"parseInt("G", 16)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_invalid_for_radix() {
    let code = r#"parseInt("2", 2)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_wrong_type_first_arg() {
    let code = r#"parseInt(42, 10)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_wrong_type_second_arg() {
    let code = r#"parseInt("42", "10")"#;
    assert_has_error(code);
}

// ============================================================================
// parseFloat Tests
// ============================================================================

#[test]
fn test_parse_float_integer() {
    let code = r#"parseFloat("42")"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_float_decimal() {
    let code = r#"parseFloat("3.5")"#;
    assert_eval_number(code, 3.5);
}

#[test]
fn test_parse_float_negative() {
    let code = r#"parseFloat("-10.5")"#;
    assert_eval_number(code, -10.5);
}

#[test]
fn test_parse_float_scientific_lowercase() {
    let code = r#"parseFloat("1.5e3")"#;
    assert_eval_number(code, 1500.0);
}

#[test]
fn test_parse_float_scientific_uppercase() {
    let code = r#"parseFloat("1.5E3")"#;
    assert_eval_number(code, 1500.0);
}

#[test]
fn test_parse_float_scientific_negative_exp() {
    let code = r#"parseFloat("1.5e-3")"#;
    assert_eval_number(code, 0.0015);
}

#[test]
fn test_parse_float_scientific_positive_exp() {
    let code = r#"parseFloat("1.5e+3")"#;
    assert_eval_number(code, 1500.0);
}

#[test]
fn test_parse_float_whitespace() {
    let code = r#"parseFloat("  3.5  ")"#;
    assert_eval_number(code, 3.5);
}

#[test]
fn test_parse_float_plus_sign() {
    let code = r#"parseFloat("+42.5")"#;
    assert_eval_number(code, 42.5);
}

#[test]
fn test_parse_float_empty_string() {
    let code = r#"parseFloat("")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_float_invalid() {
    let code = r#"parseFloat("hello")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_float_wrong_type() {
    let code = r#"parseFloat(42)"#;
    assert_has_error(code);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_typeof_guards_match() {
    let code = r#"
    let val: string = "hello";
    typeof(val) == "string" && isString(val)
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_type_conversion_chain() {
    let code = r#"
    let num: number = 42;
    let numStr: string = toString(num);
    toNumber(numStr)
"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_int_then_to_string() {
    let code = r#"
    let parsed: number = parseInt("FF", 16);
    toString(parsed)
"#;
    assert_eval_string(code, "255");
}

#[test]
fn test_type_guards_all_false_for_null() {
    let code = r#"
    let val = null;
    !isString(val) && !isNumber(val) && !isBool(val) && !isArray(val) && !isFunction(val)
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_type_guards_only_null_true() {
    let code = r#"isNull(null)"#;
    assert_eval_bool(code, true);
}

// ============================================================================
// From vm_option_result_tests.rs
// ============================================================================

// VM tests for Option<T> and Result<T,E>
//
// BLOCKER 02-D: Built-in Generic Types
//
// These tests verify VM parity with interpreter for Option and Result support.
// Tests mirror option_result_tests.rs to ensure identical behavior.

// ============================================================================
// Option<T> Tests
// ============================================================================

#[test]
fn test_option_is_some() {
    assert_eval_bool("is_some(Some(42))", true);
    assert_eval_bool("is_some(None())", false);
}

#[test]
fn test_option_is_none() {
    assert_eval_bool("is_none(None())", true);
    assert_eval_bool("is_none(Some(42))", false);
}

#[test]
fn test_option_unwrap_number() {
    assert_eval_number("unwrap(Some(42))", 42.0);
}

#[test]
fn test_option_unwrap_string() {
    assert_eval_string(r#"unwrap(Some("hello"))"#, "hello");
}

#[test]
fn test_option_unwrap_bool() {
    assert_eval_bool("unwrap(Some(true))", true);
}

#[test]
fn test_option_unwrap_null() {
    assert_eval_null("unwrap(Some(null))");
}

#[test]
fn test_option_unwrap_or_some() {
    assert_eval_number("unwrap_or(Some(42), 0)", 42.0);
}

#[test]
fn test_option_unwrap_or_none() {
    assert_eval_number("unwrap_or(None(), 99)", 99.0);
}

#[test]
fn test_option_unwrap_or_string() {
    assert_eval_string(r#"unwrap_or(Some("hello"), "default")"#, "hello");
    assert_eval_string(r#"unwrap_or(None(), "default")"#, "default");
}

#[test]
fn test_option_nested() {
    assert_eval_number("unwrap(unwrap(Some(Some(42))))", 42.0);
}

// ============================================================================
// Result<T,E> Tests
// ============================================================================

#[test]
fn test_result_is_ok() {
    assert_eval_bool("is_ok(Ok(42))", true);
    assert_eval_bool(r#"is_ok(Err("failed"))"#, false);
}

#[test]
fn test_result_is_err() {
    assert_eval_bool(r#"is_err(Err("failed"))"#, true);
    assert_eval_bool("is_err(Ok(42))", false);
}

#[test]
fn test_result_unwrap_ok_number() {
    assert_eval_number("unwrap(Ok(42))", 42.0);
}

#[test]
fn test_result_unwrap_ok_string() {
    assert_eval_string(r#"unwrap(Ok("success"))"#, "success");
}

#[test]
fn test_result_unwrap_ok_null() {
    assert_eval_null("unwrap(Ok(null))");
}

#[test]
fn test_result_unwrap_or_ok() {
    assert_eval_number("unwrap_or(Ok(42), 0)", 42.0);
}

#[test]
fn test_result_unwrap_or_err() {
    assert_eval_number(r#"unwrap_or(Err("failed"), 99)"#, 99.0);
}

#[test]
fn test_result_unwrap_or_string() {
    assert_eval_string(r#"unwrap_or(Ok("success"), "default")"#, "success");
    assert_eval_string(r#"unwrap_or(Err(404), "default")"#, "default");
}

// ============================================================================
// Mixed Option/Result Tests
// ============================================================================

#[test]
fn test_option_and_result_together() {
    let code = r#"
    let opt = Some(42);
    let res = Ok(99);
    unwrap(opt) + unwrap(res)
"#;
    assert_eval_number(code, 141.0);
}

#[test]
fn test_option_in_conditional() {
    let code = r#"
    let opt = Some(42);
    if (is_some(opt)) {
        unwrap(opt);
    } else {
        0;
    }
"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_result_in_conditional() {
    let code = r#"
    let res = Ok(42);
    if (is_ok(res)) {
        unwrap(res);
    } else {
        0;
    }
"#;
    assert_eval_number(code, 42.0);
}

// ============================================================================
// Complex Tests
// ============================================================================

#[test]
fn test_option_chain() {
    let code = r#"
    let a = Some(10);
    let b = Some(20);
    let c = Some(30);
    unwrap(a) + unwrap(b) + unwrap(c)
"#;
    assert_eval_number(code, 60.0);
}

#[test]
fn test_result_chain() {
    let code = r#"
    let a = Ok(10);
    let b = Ok(20);
    let c = Ok(30);
    unwrap(a) + unwrap(b) + unwrap(c)
"#;
    assert_eval_number(code, 60.0);
}

#[test]
fn test_option_unwrap_or_with_none_chain() {
    let code = r#"
    let a = None();
    let b = None();
    unwrap_or(a, 5) + unwrap_or(b, 10)
"#;
    assert_eval_number(code, 15.0);
}

#[test]
fn test_result_unwrap_or_with_err_chain() {
    let code = r#"
    let a = Err("fail1");
    let b = Err("fail2");
    unwrap_or(a, 5) + unwrap_or(b, 10)
"#;
    assert_eval_number(code, 15.0);
}

// ============================================================================
// From vm_result_advanced_tests.rs
// ============================================================================

// VM tests for advanced Result<T,E> methods
//
// These tests verify VM parity with interpreter for advanced Result operations.
// Tests mirror result_advanced_tests.rs to ensure identical behavior (including ? operator).

// ============================================================================
// expect() Tests
// ============================================================================

#[test]
fn test_expect_ok() {
    assert_eval_number(r#"expect(Ok(42), "should have value")"#, 42.0);
}

#[test]
fn test_expect_with_string() {
    assert_eval_string(r#"expect(Ok("success"), "should work")"#, "success");
}

// ============================================================================
// result_ok() Tests - Convert Result to Option
// ============================================================================

#[test]
fn test_result_ok_from_ok() {
    let code = r#"
    let result = Ok(42);
    let opt = result_ok(result);
    unwrap(opt)
"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_result_ok_from_err() {
    let code = r#"
    let result = Err("failed");
    let opt = result_ok(result);
    is_none(opt)
"#;
    assert_eval_bool(code, true);
}

// ============================================================================
// result_err() Tests - Extract Err to Option
// ============================================================================

#[test]
fn test_result_err_from_ok() {
    let code = r#"
    let result = Ok(42);
    let opt = result_err(result);
    is_none(opt)
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_result_err_from_err() {
    let code = r#"
    let result = Err("failed");
    let opt = result_err(result);
    unwrap(opt)
"#;
    assert_eval_string(code, "failed");
}

// ============================================================================
// result_map() Tests - Transform Ok value
// ============================================================================

#[test]
fn test_result_map_ok() {
    let code = r#"
    fn double(x: number) -> number { return x * 2; }
    let result = Ok(21);
    let mapped = result_map(result, double);
    unwrap(mapped)
"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_result_map_err_preserves() {
    let code = r#"
    fn double(x: number) -> number { return x * 2; }
    let result = Err("failed");
    let mapped = result_map(result, double);
    is_err(mapped)
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_result_map_chain() {
    let code = r#"
    fn double(x: number) -> number { return x * 2; }
    fn triple(x: number) -> number { return x * 3; }
    let result = Ok(7);
    let mapped = result_map(result, double);
    let mapped2 = result_map(mapped, triple);
    unwrap(mapped2)
"#;
    assert_eval_number(code, 42.0); // 7 * 2 * 3 = 42
}

// ============================================================================
// result_map_err() Tests - Transform Err value
// ============================================================================

#[test]
fn test_result_map_err_transforms_error() {
    let code = r#"
    fn format_error(e: string) -> string { return "Error: " + e; }
    let result = Err("failed");
    let mapped = result_map_err(result, format_error);
    unwrap_or(mapped, "default")
"#;
    assert_eval_string(code, "default");
}

#[test]
fn test_result_map_err_preserves_ok() {
    let code = r#"
    fn format_error(e: string) -> string { return "Error: " + e; }
    let result = Ok(42);
    let mapped = result_map_err(result, format_error);
    unwrap(mapped)
"#;
    assert_eval_number(code, 42.0);
}

// ============================================================================
// result_and_then() Tests - Monadic chaining
// ============================================================================

#[test]
fn test_result_and_then_success_chain() {
    let code = r#"
    fn divide(x: number) -> Result<number, string> {
        if (x == 0) {
            return Err("division by zero");
        }
        return Ok(100 / x);
    }
    let result = Ok(10);
    let chained = result_and_then(result, divide);
    unwrap(chained)
"#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_result_and_then_error_propagates() {
    let code = r#"
    fn divide(x: number) -> Result<number, string> {
        if (x == 0) {
            return Err("division by zero");
        }
        return Ok(100 / x);
    }
    let result = Err("initial error");
    let chained = result_and_then(result, divide);
    is_err(chained)
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_result_and_then_returns_error() {
    let code = r#"
    fn divide(x: number) -> Result<number, string> {
        if (x == 0) {
            return Err("division by zero");
        }
        return Ok(100 / x);
    }
    let result = Ok(0);
    let chained = result_and_then(result, divide);
    is_err(chained)
"#;
    assert_eval_bool(code, true);
}

// ============================================================================
// result_or_else() Tests - Error recovery
// ============================================================================

#[test]
fn test_result_or_else_recovers_from_error() {
    let code = r#"
    fn recover(_e: string) -> Result<number, string> {
        return Ok(0);
    }
    let result = Err("failed");
    let recovered = result_or_else(result, recover);
    unwrap(recovered)
"#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_result_or_else_preserves_ok() {
    let code = r#"
    fn recover(_e: string) -> Result<number, string> {
        return Ok(0);
    }
    let result = Ok(42);
    let recovered = result_or_else(result, recover);
    unwrap(recovered)
"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_result_or_else_can_return_error() {
    let code = r#"
    fn retry(_e: string) -> Result<number, string> {
        return Err("retry failed");
    }
    let result = Err("initial");
    let recovered = result_or_else(result, retry);
    is_err(recovered)
"#;
    assert_eval_bool(code, true);
}

// ============================================================================
// Complex Combination Tests
// ============================================================================

#[test]
fn test_result_pipeline() {
    let code = r#"
    fn double(x: number) -> number { return x * 2; }
    fn safe_divide(x: number) -> Result<number, string> {
        if (x == 0) {
            return Err("division by zero");
        }
        return Ok(100 / x);
    }

    let result = Ok(10);
    let step1 = result_map(result, double);
    let step2 = result_and_then(step1, safe_divide);
    unwrap(step2)
"#;
    assert_eval_number(code, 5.0); // (10 * 2) = 20, then 100 / 20 = 5
}

#[test]
fn test_result_error_recovery_pipeline() {
    let code = r#"
    fn recover(_e: string) -> Result<number, string> {
        return Ok(99);
    }
    fn double(x: number) -> number { return x * 2; }

    let result = Err("initial");
    let recovered = result_or_else(result, recover);
    let mapped = result_map(recovered, double);
    unwrap(mapped)
"#;
    assert_eval_number(code, 198.0); // recover to 99, then * 2
}

// ============================================================================
// Error Propagation Operator (?) Tests
// ============================================================================

#[test]
fn test_try_operator_unwraps_ok() {
    let code = r#"
    fn get_value() -> Result<number, string> {
        let result = Ok(42);
        return Ok(result?);
    }
    unwrap(get_value())
"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_try_operator_propagates_error() {
    let code = r#"
    fn get_value() -> Result<number, string> {
        let result = Err("failed");
        return Ok(result?);
    }
    is_err(get_value())
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_try_operator_multiple_propagations() {
    let code = r#"
    fn divide(a: number, b: number) -> Result<number, string> {
        if (b == 0) {
            return Err("division by zero");
        }
        return Ok(a / b);
    }

    fn calculate() -> Result<number, string> {
        let x = divide(100, 10)?;
        let y = divide(x, 2)?;
        let z = divide(y, 5)?;
        return Ok(z);
    }

    unwrap(calculate())
"#;
    assert_eval_number(code, 1.0); // 100 / 10 = 10, 10 / 2 = 5, 5 / 5 = 1
}

#[test]
fn test_try_operator_early_return() {
    let code = r#"
    fn divide(a: number, b: number) -> Result<number, string> {
        if (b == 0) {
            return Err("division by zero");
        }
        return Ok(a / b);
    }

    fn calculate() -> Result<number, string> {
        let x = divide(100, 10)?;
        let y = divide(x, 0)?;  // This will error
        let z = divide(y, 5)?;  // This won't execute
        return Ok(z);
    }

    is_err(calculate())
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_try_operator_with_expressions() {
    let code = r#"
    fn get_number() -> Result<number, string> {
        return Ok(21);
    }

    fn double_it() -> Result<number, string> {
        return Ok(get_number()? * 2);
    }

    unwrap(double_it())
"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_try_operator_in_nested_calls() {
    let code = r#"
    fn inner() -> Result<number, string> {
        return Ok(42);
    }

    fn middle() -> Result<number, string> {
        return Ok(inner()?);
    }

    fn outer() -> Result<number, string> {
        return Ok(middle()?);
    }

    unwrap(outer())
"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_try_operator_with_error_in_nested_calls() {
    let code = r#"
    fn inner() -> Result<number, string> {
        return Err("inner failed");
    }

    fn middle() -> Result<number, string> {
        return Ok(inner()?);
    }

    fn outer() -> Result<number, string> {
        return Ok(middle()?);
    }

    is_err(outer())
"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_try_operator_combined_with_methods() {
    let code = r#"
    fn get_value() -> Result<number, string> {
        return Ok(10);
    }

    fn double(x: number) -> number {
        return x * 2;
    }

    fn process() -> Result<number, string> {
        let val = get_value()?;
        let mapped = Ok(double(val));
        return Ok(mapped?);
    }

    unwrap(process())
"#;
    assert_eval_number(code, 20.0);
}
