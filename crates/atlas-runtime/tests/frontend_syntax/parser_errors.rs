//! Parser error tests (lines 588-963 from original frontend_syntax.rs)

use super::*;

// Error Recovery
// ============================================================================

#[test]
fn test_parse_error_recovery() {
    let (program, diagnostics) = parse_source("let x = ; let y = 2;");
    assert!(!diagnostics.is_empty(), "Expected syntax error");
    // Parser should recover and parse the second statement
    assert!(
        !program.items.is_empty(),
        "Expected at least one item after recovery"
    );
}

#[test]
fn test_parse_missing_semicolon_error() {
    let (_program, diagnostics) = parse_source("let x = 1 let y = 2;");
    assert!(
        !diagnostics.is_empty(),
        "Expected syntax error for missing semicolon"
    );
}

// ============================================================================
// Nested Functions (Phase 1: Parser Support)
// ============================================================================

#[test]
fn test_parse_nested_function_in_function() {
    let source = r#"
        fn outer() -> number {
            fn helper(x: number) -> number {
                return x * 2;
            }
            return helper(21);
        }
    "#;
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_function_in_if_block() {
    let source = r#"
        fn outer() -> void {
            if (true) {
                fn helper() -> void {
                    print("hello");
                }
                helper();
            }
        }
    "#;
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_function_in_while_block() {
    let source = r#"
        fn outer() -> void {
            var i: number = 0;
            while (i < 5) {
                fn increment() -> void {
                    i++;
                }
                increment();
                i++;
            }
        }
    "#;
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_function_in_for_block() {
    let source = r#"
        fn outer() -> void {
            for (var i: number = 0; i < 5; i++) {
                fn log(x: number) -> void {
                    print(str(x));
                }
                log(i);
            }
        }
    "#;
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_multiple_nested_functions_same_scope() {
    let source = r#"
        fn outer() -> number {
            fn add(a: number, b: number) -> number {
                return a + b;
            }
            fn multiply(a: number, b: number) -> number {
                return a * b;
            }
            return add(2, multiply(3, 4));
        }
    "#;
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_deeply_nested_functions() {
    let source = r#"
        fn level1() -> number {
            fn level2() -> number {
                fn level3() -> number {
                    return 42;
                }
                return level3();
            }
            return level2();
        }
    "#;
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_function_with_type_params() {
    let source = r#"
        fn outer<T>() -> void {
            fn inner<E>(x: E) -> E {
                return x;
            }
        }
    "#;
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_function_no_params() {
    let source = r#"
        fn outer() -> number {
            fn get_value() -> number {
                return 42;
            }
            return get_value();
        }
    "#;
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_function_defaults_to_null_return_type() {
    let source = r#"
        fn outer() -> void {
            fn helper(x: number) {
                return x;
            }
        }
    "#;
    let (program, diagnostics) = parse_source(source);
    // Parser allows omitting return type arrow - defaults to null type
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_function_syntax_error_missing_body() {
    let source = r#"
        fn outer() -> void {
            fn helper() -> void;
        }
    "#;
    let (_program, diagnostics) = parse_source(source);
    // Parser should report syntax error for missing function body
    assert!(
        !diagnostics.is_empty(),
        "Expected parser error for missing function body"
    );
}

// ============================================================================
// Parser Error Tests (from parser_error_tests.rs)
// ============================================================================

fn parse_errors(source: &str) -> Vec<atlas_runtime::diagnostic::Diagnostic> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (_program, diagnostics) = parser.parse();
    diagnostics
}

fn is_parser_error_code(code: &str) -> bool {
    matches!(
        code,
        "AT1000" | "AT1001" | "AT1002" | "AT1003" | "AT1004" | "AT1005"
    )
}

fn assert_has_parser_error(
    diagnostics: &[atlas_runtime::diagnostic::Diagnostic],
    expected_substring: &str,
) {
    assert!(!diagnostics.is_empty(), "Expected at least one diagnostic");
    let expected_lower = expected_substring.to_lowercase();
    let found = diagnostics.iter().any(|d| {
        d.message.to_lowercase().contains(&expected_lower) && is_parser_error_code(&d.code)
    });
    assert!(
        found,
        "Expected parser error with '{}', got: {:?}",
        expected_substring,
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ============================================================================
// Missing Semicolons
// ============================================================================

#[rstest]
#[case("let x = 42", "';'")]
#[case("foo()", "';'")]
#[case("return 42", "';'")]
#[case("break", "';'")]
#[case("continue", "';'")]
fn test_missing_semicolons(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Variable Declaration Errors
// ============================================================================

#[rstest]
#[case("let = 42;", "variable name")]
#[case("let x;", "=")]
#[case("let x = ;", "expression")]
fn test_var_declaration_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Function Declaration Errors
// ============================================================================

#[rstest]
#[case("fn () { }", "function name")]
#[case("fn foo { }", "'('")]
#[case("fn foo()", "'{'")]
#[case("fn foo() { let x = 1;", "'}'")]
#[case("fn foo(x) { }", "':'")]
#[case("fn foo(: number) { }", "parameter name")]
fn test_function_declaration_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Nested Functions - Parser Support Added (Phase 1)
// ============================================================================
//
// NOTE: Nested function syntax is now allowed by the parser (Phase 1 complete).
// Semantic validation (binder/typechecker) will be added in Phases 3-4.
// The parser no longer rejects nested functions - it parses them as Stmt::FunctionDecl.
// Tests for semantic errors (AT1013) will be added in later phases.

// ============================================================================
// If Statement Errors
// ============================================================================

#[rstest]
#[case("if { }", "(")]
#[case("if (true { }", ")")]
#[case("if (true) }", "{")]
fn test_if_statement_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// While Loop Errors
// ============================================================================

#[rstest]
#[case("while { }", "(")]
#[case("while (true { }", ")")]
#[case("while (true) }", "{")]
fn test_while_loop_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// For Loop Errors
// ============================================================================

#[rstest]
#[case("for { }", "variable")] // for-in syntax: expects variable name, not '('
#[case("for (let i = 0 { }", ";")]
#[case("for (let i = 0; i < 10 { }", ";")]
#[case("for (let i = 0; i < 10; i++ { }", ")")]
#[case("for (let i = 0; i < 10; i++) }", "{")]
fn test_for_loop_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Expression Errors
// ============================================================================

#[rstest]
#[case("1 +", "expression")]
#[case("1 + + 2", "expression")]
#[case("let x = (1 + 2;", "')'")]
#[case("let x = [1, 2, 3;", "']'")]
#[case("arr[];", "expression")]
#[case("arr[0;", "']'")]
#[case("foo(1, 2, 3;", "')'")]
fn test_expression_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Block Errors
// ============================================================================

#[rstest]
#[case("{ let x = 1", "}")]
#[case("fn foo() -> number { return 1", "}")]
fn test_block_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Array Literal Errors
// ============================================================================

#[test]
fn test_array_literal_unclosed() {
    // Note: This might get consumed as expression start, so just check for error
    let diagnostics = parse_errors("[1, 2");
    assert!(!diagnostics.is_empty(), "Expected error for unclosed array");
}

// ============================================================================
// Unary Operator Errors
// ============================================================================

#[rstest]
#[case("-", "expression")]
#[case("!", "expression")]
fn test_unary_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Operator Precedence Tests (from operator_precedence_tests.rs)
// ============================================================================
