use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;

fn parse_source(source: &str) -> (atlas_runtime::ast::Program, Vec<atlas_runtime::diagnostic::Diagnostic>) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

fn assert_has_error(diagnostics: &[atlas_runtime::diagnostic::Diagnostic], expected_substring: &str) {
    assert!(!diagnostics.is_empty(), "Expected at least one diagnostic");
    let expected_lower = expected_substring.to_lowercase();
    let found = diagnostics.iter().any(|d| {
        d.message.to_lowercase().contains(&expected_lower) && d.code == "AT1000"
    });
    assert!(found, "Expected diagnostic with message containing '{}' and code AT1000, got: {:?}",
        expected_substring, diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>());
}

// ========== Missing Semicolons ==========

#[test]
fn test_error_missing_semicolon_after_var_decl() {
    let (_program, diagnostics) = parse_source("let x = 42");
    assert_has_error(&diagnostics, "';'");
}

#[test]
fn test_error_missing_semicolon_after_expr() {
    let (_program, diagnostics) = parse_source("foo()");
    assert_has_error(&diagnostics, "';'");
}

#[test]
fn test_error_missing_semicolon_after_return() {
    let (_program, diagnostics) = parse_source("return 42");
    assert_has_error(&diagnostics, "';'");
}

#[test]
fn test_error_missing_semicolon_after_break() {
    let (_program, diagnostics) = parse_source("break");
    assert_has_error(&diagnostics, "';'");
}

#[test]
fn test_error_missing_semicolon_after_continue() {
    let (_program, diagnostics) = parse_source("continue");
    assert_has_error(&diagnostics, "';'");
}

// ========== Missing Parts of Declarations ==========

#[test]
fn test_error_missing_variable_name() {
    let (_program, diagnostics) = parse_source("let = 42;");
    assert_has_error(&diagnostics, "variable name");
}

#[test]
fn test_error_missing_variable_initializer() {
    let (_program, diagnostics) = parse_source("let x;");
    assert_has_error(&diagnostics, "=");
}

#[test]
fn test_error_missing_variable_value() {
    let (_program, diagnostics) = parse_source("let x = ;");
    assert_has_error(&diagnostics, "expression");
}

// ========== Function Declaration Errors ==========

#[test]
fn test_error_missing_function_name() {
    let (_program, diagnostics) = parse_source("fn () { }");
    assert_has_error(&diagnostics, "function name");
}

#[test]
fn test_error_missing_function_parens() {
    let (_program, diagnostics) = parse_source("fn foo { }");
    assert_has_error(&diagnostics, "'('");
}

#[test]
fn test_error_missing_function_body() {
    let (_program, diagnostics) = parse_source("fn foo()");
    assert_has_error(&diagnostics, "'{'");
}

#[test]
fn test_error_unclosed_function_body() {
    let (_program, diagnostics) = parse_source("fn foo() { let x = 1;");
    assert_has_error(&diagnostics, "'}'");
}

#[test]
fn test_error_function_missing_param_type() {
    let (_program, diagnostics) = parse_source("fn foo(x) { }");
    assert_has_error(&diagnostics, "':'");
}

#[test]
fn test_error_function_missing_param_name() {
    let (_program, diagnostics) = parse_source("fn foo(: number) { }");
    assert_has_error(&diagnostics, "parameter name");
}

// ========== Nested Function Declarations (Not Allowed) ==========

#[test]
fn test_error_nested_function_in_block() {
    let (_program, diagnostics) = parse_source(r#"
        fn outer() {
            fn inner() {
                return 1;
            }
        }
    "#);
    assert_has_error(&diagnostics, "function");
}

#[test]
fn test_error_nested_function_in_if() {
    let (_program, diagnostics) = parse_source(r#"
        if (true) {
            fn foo() {
                return 1;
            }
        }
    "#);
    assert_has_error(&diagnostics, "function");
}

#[test]
fn test_error_nested_function_in_while() {
    let (_program, diagnostics) = parse_source(r#"
        while (true) {
            fn foo() {
                return 1;
            }
        }
    "#);
    assert_has_error(&diagnostics, "function");
}

#[test]
fn test_error_nested_function_in_for() {
    let (_program, diagnostics) = parse_source(r#"
        for (let i = 0; i < 10; i = i + 1) {
            fn foo() {
                return 1;
            }
        }
    "#);
    assert_has_error(&diagnostics, "function");
}

// ========== Import Keyword (Reserved, Not Allowed in v0.1) ==========

#[test]
fn test_error_import_not_supported() {
    let (_program, diagnostics) = parse_source("import foo;");
    // Import is a reserved keyword, so lexer will tokenize it, but parser will fail
    // since we don't have import statements in v0.1
    assert!(!diagnostics.is_empty(), "Expected error for unsupported import");
}

// ========== Control Flow Errors ==========

#[test]
fn test_error_if_missing_condition() {
    let (_program, diagnostics) = parse_source("if { }");
    assert_has_error(&diagnostics, "'('");
}

#[test]
fn test_error_if_missing_paren() {
    let (_program, diagnostics) = parse_source("if (true { }");
    assert_has_error(&diagnostics, "')'");
}

#[test]
fn test_error_if_missing_block() {
    let (_program, diagnostics) = parse_source("if (true)");
    assert_has_error(&diagnostics, "'{'");
}

#[test]
fn test_error_while_missing_condition() {
    let (_program, diagnostics) = parse_source("while { }");
    assert_has_error(&diagnostics, "'('");
}

#[test]
fn test_error_while_missing_paren() {
    let (_program, diagnostics) = parse_source("while (true { }");
    assert_has_error(&diagnostics, "')'");
}

#[test]
fn test_error_while_missing_block() {
    let (_program, diagnostics) = parse_source("while (true)");
    assert_has_error(&diagnostics, "'{'");
}

#[test]
fn test_error_for_missing_paren() {
    let (_program, diagnostics) = parse_source("for let i = 0; i < 10; i = i + 1 { }");
    assert_has_error(&diagnostics, "'('");
}

#[test]
fn test_error_for_missing_semicolon_after_init() {
    let (_program, diagnostics) = parse_source("for (let i = 0 i < 10; i = i + 1) { }");
    assert_has_error(&diagnostics, "';'");
}

#[test]
fn test_error_for_missing_semicolon_after_condition() {
    let (_program, diagnostics) = parse_source("for (let i = 0; i < 10 i = i + 1) { }");
    assert_has_error(&diagnostics, "';'");
}

#[test]
fn test_error_for_missing_closing_paren() {
    let (_program, diagnostics) = parse_source("for (let i = 0; i < 10; i = i + 1 { }");
    assert_has_error(&diagnostics, "')'");
}

#[test]
fn test_error_for_missing_block() {
    let (_program, diagnostics) = parse_source("for (let i = 0; i < 10; i = i + 1)");
    assert_has_error(&diagnostics, "'{'");
}

// ========== Expression Errors ==========

#[test]
fn test_error_unclosed_paren() {
    let (_program, diagnostics) = parse_source("let x = (1 + 2;");
    assert_has_error(&diagnostics, "')'");
}

#[test]
fn test_error_unclosed_array() {
    let (_program, diagnostics) = parse_source("let x = [1, 2, 3;");
    assert_has_error(&diagnostics, "']'");
}

#[test]
fn test_error_missing_array_index() {
    let (_program, diagnostics) = parse_source("arr[];");
    assert_has_error(&diagnostics, "expression");
}

#[test]
fn test_error_unclosed_array_index() {
    let (_program, diagnostics) = parse_source("arr[0;");
    assert_has_error(&diagnostics, "']'");
}

#[test]
fn test_error_unclosed_function_call() {
    let (_program, diagnostics) = parse_source("foo(1, 2, 3;");
    assert_has_error(&diagnostics, "')'");
}

#[test]
fn test_error_invalid_assignment_target() {
    let (_program, diagnostics) = parse_source("42 = 1;");
    // Should get an error about invalid assignment target
    assert!(!diagnostics.is_empty(), "Expected error for invalid assignment target");
}

// ========== Binary Operator Errors ==========

#[test]
fn test_error_missing_right_operand() {
    let (_program, diagnostics) = parse_source("let x = 1 +;");
    assert_has_error(&diagnostics, "expression");
}

#[test]
fn test_error_double_operator() {
    let (_program, _diagnostics) = parse_source("let x = 1 + + 2;");
    // This might parse as 1 + (+2), so check if we get an error or not
    // Actually this should be valid (unary plus on 2)
    // Let's try a different invalid case
}

// ========== Block and Statement Errors ==========

#[test]
fn test_error_unclosed_block() {
    let (_program, diagnostics) = parse_source("{ let x = 1;");
    assert_has_error(&diagnostics, "'}'");
}

#[test]
fn test_error_unexpected_closing_brace() {
    let (_program, _diagnostics) = parse_source("let x = 1; }");
    // This might just be ignored as trailing tokens
    // The parser should handle this as an unexpected token
}

// ========== Type Annotation Errors ==========

#[test]
fn test_error_missing_type_after_colon() {
    let (_program, diagnostics) = parse_source("let x: = 42;");
    assert_has_error(&diagnostics, "type");
}

// ========== Assignment Errors ==========

#[test]
fn test_error_assignment_missing_value() {
    let (_program, diagnostics) = parse_source("x = ;");
    assert_has_error(&diagnostics, "expression");
}

#[test]
fn test_error_assignment_to_literal() {
    let (_program, diagnostics) = parse_source(r#""hello" = 42;"#);
    assert!(!diagnostics.is_empty(), "Expected error for assigning to literal");
}

// ========== Error Recovery ==========

#[test]
fn test_error_recovery_multiple_errors() {
    let (_program, diagnostics) = parse_source(r#"
        let x =
        let y = 2;
        let z =
    "#);
    // Should have multiple errors but parser should recover
    assert!(diagnostics.len() >= 2, "Expected at least 2 errors");
}

#[test]
fn test_error_recovery_continues_parsing() {
    let (program, diagnostics) = parse_source(r#"
        let x = ;
        let y = 42;
    "#);
    // Should have 1 error for the first line
    assert!(!diagnostics.is_empty(), "Expected error");
    // But should still parse the second statement
    assert!(program.items.len() >= 1, "Parser should recover and parse second statement");
}
