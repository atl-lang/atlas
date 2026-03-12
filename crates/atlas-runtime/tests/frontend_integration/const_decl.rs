//! Const declaration tests (B39-P04)

use super::*;
use atlas_runtime::Atlas;

// ============================================================================
// Lexer Tests
// ============================================================================

#[test]
fn test_const_keyword_lexes_correctly() {
    use atlas_runtime::token::TokenKind;

    let mut lexer = Lexer::new("const");
    let (tokens, errors) = lexer.tokenize();
    assert!(errors.is_empty(), "unexpected lex errors: {errors:?}");
    assert_eq!(tokens.len(), 2); // const + EOF
    assert_eq!(tokens[0].kind, TokenKind::Const);
    assert_eq!(tokens[0].lexeme, "const");
}

#[test]
fn test_const_keyword_in_declaration() {
    use atlas_runtime::token::TokenKind;

    let src = "const PI = 3.14;";
    let mut lexer = Lexer::new(src);
    let (tokens, errors) = lexer.tokenize();
    assert!(errors.is_empty(), "unexpected lex errors: {errors:?}");

    let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind).collect();
    assert!(
        kinds.contains(&TokenKind::Const),
        "expected Const token in: {kinds:?}"
    );
}

// ============================================================================
// Parser Tests
// ============================================================================

#[test]
fn test_const_decl_parses() {
    let src = "const PI = 3.14;";
    let (success, errors) = parse_source(src);
    assert!(success, "parse failed: {errors:?}");
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn test_const_decl_with_type_annotation() {
    let src = "const PI: number = 3.14159;";
    let (success, errors) = parse_source(src);
    assert!(success, "parse failed: {errors:?}");
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn test_const_decl_string() {
    let src = r#"const NAME: string = "Atlas";"#;
    let (success, errors) = parse_source(src);
    assert!(success, "parse failed: {errors:?}");
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn test_const_decl_boolean() {
    let src = "const DEBUG = true;";
    let (success, errors) = parse_source(src);
    assert!(success, "parse failed: {errors:?}");
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn test_export_const_parses() {
    let src = "export const PI = 3.14;";
    let (success, errors) = parse_source(src);
    assert!(success, "parse failed: {errors:?}");
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

// ============================================================================
// Execution Tests
// ============================================================================

#[test]
fn test_const_number_value() {
    let runtime = Atlas::new();
    let src = "const PI = 3.14; PI;";
    let result = runtime.eval(src);
    assert!(result.is_ok(), "eval failed: {:?}", result);
    assert_eq!(result.unwrap(), Value::Number(3.14));
}

#[test]
fn test_const_string_value() {
    let runtime = Atlas::new();
    let src = r#"const GREETING = "Hello"; GREETING;"#;
    let result = runtime.eval(src);
    assert!(result.is_ok(), "eval failed: {:?}", result);
    match result.unwrap() {
        Value::String(s) => assert_eq!(s.as_ref(), "Hello"),
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn test_const_bool_value() {
    let runtime = Atlas::new();
    let src = "const DEBUG = true; DEBUG;";
    let result = runtime.eval(src);
    assert!(result.is_ok(), "eval failed: {:?}", result);
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
fn test_const_arithmetic() {
    let runtime = Atlas::new();
    let src = "const PI = 3.14; const TAU = PI * 2; TAU;";
    let result = runtime.eval(src);
    assert!(result.is_ok(), "eval failed: {:?}", result);
    assert_eq!(result.unwrap(), Value::Number(6.28));
}

#[test]
fn test_const_reference_chain() {
    let runtime = Atlas::new();
    let src = "const A = 10; const B = A + 5; const C = B * 2; C;";
    let result = runtime.eval(src);
    assert!(result.is_ok(), "eval failed: {:?}", result);
    assert_eq!(result.unwrap(), Value::Number(30.0));
}

#[test]
fn test_const_in_expression() {
    let runtime = Atlas::new();
    let src = "const MULTIPLIER = 10; let x = 5; x * MULTIPLIER;";
    let result = runtime.eval(src);
    assert!(result.is_ok(), "eval failed: {:?}", result);
    assert_eq!(result.unwrap(), Value::Number(50.0));
}

// ============================================================================
// Error Tests
// ============================================================================

#[test]
fn test_const_non_literal_errors() {
    let runtime = Atlas::new();
    let src = r#"
fn get_value(): number {
    return 42;
}
const X = get_value();
X;
"#;
    let result = runtime.eval(src);
    assert!(result.is_err(), "expected error for non-const initializer");
    let err = result.unwrap_err();
    let err_str = format!("{:?}", err);
    assert!(
        err_str.contains("AT3060") || err_str.contains("compile-time"),
        "expected AT3060 const error, got: {}",
        err_str
    );
}

#[test]
fn test_const_duplicate_uses_second_value() {
    // Duplicate const is a warning, not an error — uses the second value
    let runtime = Atlas::new();
    let src = "const PI = 3.14; const PI = 3.14159; PI;";
    let result = runtime.eval(src);
    // Should succeed with the second value
    assert!(result.is_ok(), "eval failed: {:?}", result);
    assert_eq!(result.unwrap(), Value::Number(3.14159));
}

// ============================================================================
// Formatter Tests
// ============================================================================

#[test]
fn test_const_formats_correctly() {
    let src = "const   PI   =   3.14;";
    let formatted = fmt(src);
    assert_eq!(formatted.trim(), "const PI = 3.14;");
}

#[test]
fn test_const_with_type_formats_correctly() {
    let src = "const   PI:number   =   3.14;";
    let formatted = fmt(src);
    assert_eq!(formatted.trim(), "const PI: number = 3.14;");
}

#[test]
fn test_export_const_formats_correctly() {
    let src = "export const   PI   =   3.14;";
    let formatted = fmt(src);
    assert_eq!(formatted.trim(), "export const PI = 3.14;");
}
