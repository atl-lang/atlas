use super::*;
use pretty_assertions::assert_eq;

// From vm_member_tests.rs
// ============================================================================

// VM tests for method call syntax (Phase 17) - mirrors interpreter tests for 100% parity

fn run_vm(source: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);
    let mut compiler = Compiler::new();
    match compiler.compile(&program) {
        Ok(bytecode) => {
            let mut vm = VM::new(bytecode);
            match vm.run(&SecurityContext::allow_all()) {
                Ok(opt_value) => match opt_value {
                    Some(value) => Ok(format!("{:?}", value)),
                    None => Ok("None".to_string()),
                },
                Err(e) => Err(format!("{:?}", e)),
            }
        }
        Err(e) => Err(format!("Compile error: {:?}", e)),
    }
}

// JSON as_string() Tests
#[rstest]
#[case(
    r#"let data: json = unwrap(Json.parse("{\"name\":\"Alice\"}")); data["name"].as_string();"#,
    r#"String("Alice")"#
)]
#[case(r#"let data: json = unwrap(Json.parse("{\"user\":{\"name\":\"Bob\"}}")); data["user"]["name"].as_string();"#, r#"String("Bob")"#)]
fn test_json_as_string(#[case] source: &str, #[case] expected: &str) {
    let result = run_vm(source).expect("Should succeed");
    assert_eq!(result, expected);
}

#[test]
fn test_json_as_string_error() {
    let result =
        run_vm(r#"let data: json = unwrap(Json.parse("{\"age\":30}")); data["age"].as_string();"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot extract string"));
}

// JSON as_number() Tests
#[rstest]
#[case(
    r#"let data: json = unwrap(Json.parse("{\"age\":30}")); data["age"].as_number();"#,
    "Number(30)"
)]
#[case(
    r#"let data: json = unwrap(Json.parse("{\"price\":19.99}")); data["price"].as_number();"#,
    "Number(19.99)"
)]
fn test_json_as_number(#[case] source: &str, #[case] expected: &str) {
    let result = run_vm(source).expect("Should succeed");
    assert_eq!(result, expected);
}

#[test]
fn test_json_as_number_error() {
    let result = run_vm(
        r#"let data: json = unwrap(Json.parse("{\"name\":\"Alice\"}")); data["name"].as_number();"#,
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot extract number"));
}

// JSON as_bool() Tests
#[rstest]
#[case(
    r#"let data: json = unwrap(Json.parse("{\"active\":true}")); data["active"].as_bool();"#,
    "Bool(true)"
)]
#[case(
    r#"let data: json = unwrap(Json.parse("{\"disabled\":false}")); data["disabled"].as_bool();"#,
    "Bool(false)"
)]
fn test_json_as_bool(#[case] source: &str, #[case] expected: &str) {
    let result = run_vm(source).expect("Should succeed");
    assert_eq!(result, expected);
}

#[test]
fn test_json_as_bool_error() {
    let result =
        run_vm(r#"let data: json = unwrap(Json.parse("{\"count\":5}")); data["count"].as_bool();"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot extract bool"));
}

// JSON is_null() Tests
#[rstest]
#[case(
    r#"let data: json = unwrap(Json.parse("{\"value\":null}")); data["value"].is_null();"#,
    "Bool(true)"
)]
#[case(
    r#"let data: json = unwrap(Json.parse("{\"value\":\"text\"}")); data["value"].is_null();"#,
    "Bool(false)"
)]
#[case(
    r#"let data: json = unwrap(Json.parse("{\"value\":42}")); data["value"].is_null();"#,
    "Bool(false)"
)]
fn test_json_is_null(#[case] source: &str, #[case] expected: &str) {
    let result = run_vm(source).expect("Should succeed");
    assert_eq!(result, expected);
}

// Complex Tests
#[test]
fn test_chained_methods() {
    let result = run_vm(
        r#"
        let data: json = unwrap(Json.parse("{\"user\":{\"name\":\"Charlie\"}}"));
        data["user"]["name"].as_string();
    "#,
    )
    .expect("Should succeed");
    assert_eq!(result, r#"String("Charlie")"#);
}

#[test]
fn test_method_in_expression() {
    let result = run_vm(
        r#"
        let data: json = unwrap(Json.parse("{\"count\":5}"));
        data["count"].as_number() + 10;
    "#,
    )
    .expect("Should succeed");
    assert_eq!(result, "Number(15)");
}

#[test]
fn test_method_in_conditional() {
    let result = run_vm(
        r#"
        let data: json = unwrap(Json.parse("{\"enabled\":true}"));
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
    let result = run_vm(
        r#"
        let data: json = unwrap(Json.parse("{\"a\":5,\"b\":10}"));
        data["a"].as_number() + data["b"].as_number();
    "#,
    )
    .expect("Should succeed");
    assert_eq!(result, "Number(15)");
}

// Error Cases
#[test]
fn test_as_string_on_null() {
    let result =
        run_vm(r#"let data: json = unwrap(Json.parse("{\"v\":null}")); data["v"].as_string();"#);
    assert!(result.is_err());
}

#[test]
fn test_as_number_on_null() {
    let result =
        run_vm(r#"let data: json = unwrap(Json.parse("{\"v\":null}")); data["v"].as_number();"#);
    assert!(result.is_err());
}

#[test]
fn test_as_bool_on_null() {
    let result =
        run_vm(r#"let data: json = unwrap(Json.parse("{\"v\":null}")); data["v"].as_bool();"#);
    assert!(result.is_err());
}

#[test]
fn test_as_string_on_object() {
    let result = run_vm(
        r#"let data: json = unwrap(Json.parse("{\"obj\":{\"a\":1}}")); data["obj"].as_string();"#,
    );
    assert!(result.is_err());
}

#[test]
fn test_as_number_on_array() {
    let result = run_vm(
        r#"let data: json = unwrap(Json.parse("{\"arr\":[1,2,3]}")); data["arr"].as_number();"#,
    );
    assert!(result.is_err());
}

// ============================================================================

// ============================================================================
// B15-P06: Tuple destructuring in VM
// ============================================================================

#[test]
fn test_vm_tuple_destructure_basic() {
    // Use TupleGet opcode directly to verify VM destructure works
    let src = r#"let t = (42, "hello"); let a = t.0; let b = t.1; a;"#;
    let result = run_vm(src);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert!(result.unwrap().contains("42"));
}

#[test]
fn test_vm_let_destructure_values() {
    let src = r#"let (a, b) = (10, 20); a + b;"#;
    let result = run_vm(src);
    // Number display may be integer or float depending on debug format
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    let s = result.unwrap();
    assert!(s.contains("30"), "Expected 30 in result, got {}", s);
}

#[test]
fn test_vm_let_destructure_three_elements() {
    let src = r#"let (x, y, z) = (1, 2, 3); x + y + z;"#;
    let result = run_vm(src);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    let s = result.unwrap();
    assert!(s.contains("6"), "Expected 6 in result, got {}", s);
}

#[test]
fn test_vm_tuple_match_pattern() {
    let src = r#"match (1, 42) { (1, y) => y, _ => 0 };"#;
    let result = run_vm(src);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert!(result.unwrap().contains("42"), "Expected 42 in result");
}

#[test]
fn test_vm_tuple_match_wildcard() {
    let src = r#"match (99, 7) { (0, y) => y, (_, n) => n };"#;
    let result = run_vm(src);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert!(result.unwrap().contains("7"), "Expected 7 in result");
}
