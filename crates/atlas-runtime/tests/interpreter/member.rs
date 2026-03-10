use super::*;
use pretty_assertions::assert_eq;

// From interpreter_member_tests.rs
// ============================================================================

fn run_interpreter(source: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);
    let mut interpreter = Interpreter::new();
    match interpreter.eval(&program, &SecurityContext::allow_all()) {
        Ok(value) => Ok(format!("{:?}", value)),
        Err(e) => Err(format!("{:?}", e)),
    }
}

// JSON as_string() Tests
#[rstest]
#[case(
    r#"let data: json = unwrap(parse_json("{\"name\":\"Alice\"}")); data["name"].as_string();"#,
    r#"String("Alice")"#
)]
#[case(r#"let data: json = unwrap(parse_json("{\"user\":{\"name\":\"Bob\"}}")); data["user"]["name"].as_string();"#, r#"String("Bob")"#)]
fn test_json_as_string(#[case] source: &str, #[case] expected: &str) {
    let result = run_interpreter(source).expect("Should succeed");
    assert_eq!(result, expected);
}

#[test]
fn test_json_as_string_error() {
    let result = run_interpreter(
        r#"let data: json = unwrap(parse_json("{\"age\":30}")); data["age"].as_string();"#,
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot extract string"));
}

// JSON as_number() Tests
#[rstest]
#[case(
    r#"let data: json = unwrap(parse_json("{\"age\":30}")); data["age"].as_number();"#,
    "Number(30)"
)]
#[case(
    r#"let data: json = unwrap(parse_json("{\"price\":19.99}")); data["price"].as_number();"#,
    "Number(19.99)"
)]
fn test_json_as_number(#[case] source: &str, #[case] expected: &str) {
    let result = run_interpreter(source).expect("Should succeed");
    assert_eq!(result, expected);
}

#[test]
fn test_json_as_number_error() {
    let result = run_interpreter(
        r#"let data: json = unwrap(parse_json("{\"name\":\"Alice\"}")); data["name"].as_number();"#,
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot extract number"));
}

// JSON as_bool() Tests
#[rstest]
#[case(
    r#"let data: json = unwrap(parse_json("{\"active\":true}")); data["active"].as_bool();"#,
    "Bool(true)"
)]
#[case(
    r#"let data: json = unwrap(parse_json("{\"disabled\":false}")); data["disabled"].as_bool();"#,
    "Bool(false)"
)]
fn test_json_as_bool(#[case] source: &str, #[case] expected: &str) {
    let result = run_interpreter(source).expect("Should succeed");
    assert_eq!(result, expected);
}

#[test]
fn test_json_as_bool_error() {
    let result = run_interpreter(
        r#"let data: json = unwrap(parse_json("{\"count\":5}")); data["count"].as_bool();"#,
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot extract bool"));
}

// JSON is_null() Tests
#[rstest]
#[case(
    r#"let data: json = unwrap(parse_json("{\"value\":null}")); data["value"].is_null();"#,
    "Bool(true)"
)]
#[case(
    r#"let data: json = unwrap(parse_json("{\"value\":\"text\"}")); data["value"].is_null();"#,
    "Bool(false)"
)]
#[case(
    r#"let data: json = unwrap(parse_json("{\"value\":42}")); data["value"].is_null();"#,
    "Bool(false)"
)]
fn test_json_is_null(#[case] source: &str, #[case] expected: &str) {
    let result = run_interpreter(source).expect("Should succeed");
    assert_eq!(result, expected);
}

// Complex Tests
#[test]
fn test_chained_methods() {
    let result = run_interpreter(
        r#"
        let data: json = unwrap(parse_json("{\"user\":{\"name\":\"Charlie\"}}"));
        data["user"]["name"].as_string();
    "#,
    )
    .expect("Should succeed");
    assert_eq!(result, r#"String("Charlie")"#);
}

#[test]
fn test_method_in_expression() {
    let result = run_interpreter(
        r#"
        let data: json = unwrap(parse_json("{\"count\":5}"));
        data["count"].as_number() + 10;
    "#,
    )
    .expect("Should succeed");
    assert_eq!(result, "Number(15)");
}

#[test]
fn test_method_in_conditional() {
    let result = run_interpreter(
        r#"
        let data: json = unwrap(parse_json("{\"enabled\":true}"));
        let mut result: string = "no";
        if (data["enabled"].as_bool()) {
            result = "yes";
        };
        result;
    "#,
    )
    .expect("Should succeed");
    assert_eq!(result, r#"String("yes")"#);
}

#[test]
fn test_multiple_methods() {
    let result = run_interpreter(
        r#"
        let data: json = unwrap(parse_json("{\"a\":5,\"b\":10}"));
        data["a"].as_number() + data["b"].as_number();
    "#,
    )
    .expect("Should succeed");
    assert_eq!(result, "Number(15)");
}

// Error Cases
#[test]
fn test_as_string_on_null() {
    let result = run_interpreter(
        r#"let data: json = unwrap(parse_json("{\"v\":null}")); data["v"].as_string();"#,
    );
    assert!(result.is_err());
}

#[test]
fn test_as_number_on_null() {
    let result = run_interpreter(
        r#"let data: json = unwrap(parse_json("{\"v\":null}")); data["v"].as_number();"#,
    );
    assert!(result.is_err());
}

#[test]
fn test_as_bool_on_null() {
    let result = run_interpreter(
        r#"let data: json = unwrap(parse_json("{\"v\":null}")); data["v"].as_bool();"#,
    );
    assert!(result.is_err());
}

#[test]
fn test_as_string_on_object() {
    let result = run_interpreter(
        r#"let data: json = unwrap(parse_json("{\"obj\":{\"a\":1}}")); data["obj"].as_string();"#,
    );
    assert!(result.is_err());
}

#[test]
fn test_as_number_on_array() {
    let result = run_interpreter(
        r#"let data: json = unwrap(parse_json("{\"arr\":[1,2,3]}")); data["arr"].as_number();"#,
    );
    assert!(result.is_err());
}

// ============================================================================

// ============================================================================
// B15: Tuple tests (interpreter parity)
// ============================================================================

#[test]
fn test_interp_tuple_literal() {
    let result = run_interpreter(r#"(1, "hello", true);"#);
    let s = result.expect("tuple literal should work");
    assert!(s.contains("Tuple"), "Expected Tuple, got {}", s);
}

#[test]
fn test_interp_tuple_index_access() {
    let result = run_interpreter(r#"let t = (42, "world"); t.0;"#);
    assert!(result.unwrap().contains("42"));
}

#[test]
fn test_interp_tuple_string_access() {
    let result = run_interpreter(r#"let t = (1, "atlas"); t.1;"#);
    assert!(result.unwrap().contains("atlas"));
}

#[test]
fn test_interp_let_destructure() {
    let result = run_interpreter(r#"let (a, b) = (10, 20); a + b;"#);
    let s = result.expect("destructure should succeed");
    assert!(s.contains("30"), "Expected 30 in {}", s);
}

#[test]
fn test_interp_let_destructure_mut() {
    let result = run_interpreter(r#"let mut (x, y) = (5, 6); x = 99; x + y;"#);
    let s = result.expect("mutable destructure should succeed");
    assert!(s.contains("105"), "Expected 105 in {}", s);
}

#[test]
fn test_interp_let_destructure_three() {
    let result = run_interpreter(r#"let (a, b, c) = (1, 2, 3); a + b + c;"#);
    let s = result.expect("three-element destructure should succeed");
    assert!(s.contains("6"), "Expected 6 in {}", s);
}

#[test]
fn test_interp_tuple_match_literal() {
    let result = run_interpreter(r#"match (1, 42) { (1, y) => y, _ => 0 };"#);
    let s = result.expect("tuple match should succeed");
    assert!(s.contains("42"), "Expected 42 in {}", s);
}

#[test]
fn test_interp_tuple_match_wildcard() {
    let result = run_interpreter(r#"match (99, 7) { (0, y) => y, (_, n) => n };"#);
    let s = result.expect("tuple match wildcard should succeed");
    assert!(s.contains("7"), "Expected 7 in {}", s);
}

#[test]
fn test_interp_tuple_display_unit() {
    let result = run_interpreter(r#"let u = (); u;"#);
    assert!(result.is_ok());
}

#[test]
fn test_interp_tuple_oob_error() {
    let result = run_interpreter(r#"let t = (1, 2); t.5;"#);
    assert!(result.is_err(), "Out-of-bounds access should fail");
}

#[test]
fn test_interp_destructure_count_mismatch() {
    let result = run_interpreter(r#"let (a, b, c) = (1, 2);"#);
    assert!(result.is_err(), "Count mismatch should fail");
}

#[test]
fn test_interp_destructure_non_tuple_error() {
    let result = run_interpreter(r#"let (a, b) = 42;"#);
    assert!(result.is_err(), "Non-tuple destructure should fail");
}
