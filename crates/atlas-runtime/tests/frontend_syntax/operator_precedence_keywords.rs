//! Operator precedence and keyword policy tests (lines 973-1263 from original frontend_syntax.rs)

use super::*;

// ============================================================================
// Operator Precedence Snapshots
// ============================================================================

#[rstest]
// Multiplication/Division over Addition/Subtraction
#[case("mul_over_add", "1 + 2 * 3;")]
#[case("div_over_sub", "10 - 6 / 2;")]
#[case("mul_over_add_complex", "1 + 2 * 3 + 4;")]
#[case("div_over_sub_complex", "20 - 10 / 2 - 3;")]
// Unary operators
#[case("unary_minus_before_mul", "-2 * 3;")]
#[case("unary_not_before_and", "!false && true;")]
// Comparison operators
#[case("comparison_over_and", "1 < 2 && 3 > 2;")]
#[case("comparison_over_or", "1 == 1 || 2 != 2;")]
// Logical operators
#[case("and_before_or", "false || true && false;")]
// Parentheses override
#[case("parens_override_mul", "(1 + 2) * 3;")]
#[case("parens_override_div", "(10 - 2) / 4;")]
// Complex expressions
#[case("complex_arithmetic", "1 + 2 * 3 - 4 / 2;")]
#[case("complex_logical", "true && false || !true;")]
#[case("complex_comparison", "1 + 2 < 5 && 10 / 2 == 5;")]
// Function calls (highest precedence)
#[case("func_call_in_arithmetic", "foo() + 2 * 3;")]
#[case("func_call_in_comparison", "bar() < 5 && baz() > 0;")]
// Array indexing (highest precedence)
#[case("array_index_in_arithmetic", "arr[0] + 2 * 3;")]
#[case("array_index_in_comparison", "arr[i] < 10;")]
fn test_operator_precedence(#[case] name: &str, #[case] source: &str) {
    let program = parse_valid(source);

    // Snapshot the first statement's expression
    assert_eq!(program.items.len(), 1, "Should have one statement");
    insta::assert_yaml_snapshot!(format!("precedence_{}", name), program.items[0]);
}

// ============================================================================
// Keyword Policy Tests (from keyword_policy_tests.rs)
// ============================================================================

// ============================================================================
// Test Helpers
// ============================================================================

fn assert_parse_error_present(diagnostics: &[atlas_runtime::diagnostic::Diagnostic]) {
    assert!(!diagnostics.is_empty(), "Expected at least one diagnostic");
    let found = diagnostics.iter().any(|d| is_parser_error_code(&d.code));
    assert!(
        found,
        "Expected parser diagnostic, got: {:?}",
        diagnostics
            .iter()
            .map(|d| (&d.code, &d.message))
            .collect::<Vec<_>>()
    );
}

fn assert_error_mentions(diagnostics: &[atlas_runtime::diagnostic::Diagnostic], keywords: &[&str]) {
    assert!(
        diagnostics.iter().any(|d| {
            let msg_lower = d.message.to_lowercase();
            keywords.iter().any(|kw| msg_lower.contains(kw))
        }),
        "Expected error message to mention one of {:?}, got: {:?}",
        keywords,
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ============================================================================
// Reserved Future Keywords - Cannot be used as identifiers
// ============================================================================

#[rstest]
#[case("let import = 1;", &["variable", "identifier"])]
#[case("let match = 1;", &["variable", "identifier"])]
#[case("var import = 1;", &["variable", "identifier"])]
#[case("var match = 1;", &["variable", "identifier"])]
fn test_future_keywords_as_variables(#[case] source: &str, #[case] expected_mentions: &[&str]) {
    let (_program, diagnostics) = parse_source(source);
    assert_parse_error_present(&diagnostics);
    assert_error_mentions(&diagnostics, expected_mentions);
}

#[rstest]
#[case("fn import() { }", &["function", "identifier"])]
#[case("fn match() { }", &["function", "identifier"])]
fn test_future_keywords_as_function_names(
    #[case] source: &str,
    #[case] expected_mentions: &[&str],
) {
    let (_program, diagnostics) = parse_source(source);
    assert_parse_error_present(&diagnostics);
    assert_error_mentions(&diagnostics, expected_mentions);
}

#[rstest]
#[case("fn foo(import: number) { }", &["parameter", "identifier"])]
#[case("fn foo(match: number) { }", &["parameter", "identifier"])]
fn test_future_keywords_as_parameters(#[case] source: &str, #[case] expected_mentions: &[&str]) {
    let (_program, diagnostics) = parse_source(source);
    assert_parse_error_present(&diagnostics);
    assert_error_mentions(&diagnostics, expected_mentions);
}

// ============================================================================
// Active Keywords - Cannot be used as identifiers
// ============================================================================

#[rstest]
#[case("var let = 1;")]
#[case("let fn = 1;")]
#[case("let if = 1;")]
#[case("let while = 1;")]
#[case("let return = 1;")]
#[case("let true = 1;")]
#[case("let false = 1;")]
#[case("let null = 1;")]
fn test_active_keywords_as_identifiers(#[case] source: &str) {
    let (_program, diagnostics) = parse_source(source);
    assert!(
        !diagnostics.is_empty(),
        "Expected error for using active keyword as identifier"
    );
    assert_parse_error_present(&diagnostics);
}

// ============================================================================
// Future Feature Keywords - Statements not supported (v0.1)
// Note: Imports ARE supported as of v0.2 (BLOCKER 04-A)
// ============================================================================

// Import statements now supported - removed outdated tests
// See module_syntax_tests.rs for valid import syntax tests

#[rstest]
#[case("match x { 1 => 2 }", "match")]
fn test_match_expressions_not_supported(#[case] source: &str, #[case] keyword: &str) {
    let (_program, diagnostics) = parse_source(source);
    assert!(
        !diagnostics.is_empty(),
        "Expected error for '{}' expression",
        keyword
    );
    // Should have some error since match is not supported
}

// ============================================================================
// Valid Keyword Usage
// ============================================================================

#[rstest]
#[case("let x = 1;")]
#[case("fn foo() { }")]
#[case("if (true) { }")]
#[case("while (false) { }")]
#[case("return 42;")]
fn test_valid_keyword_usage(#[case] source: &str) {
    let (_program, diagnostics) = parse_source(source);
    // These should parse without errors (though return outside function might have semantic errors)
    // At parser level, these are valid
    let has_parser_error = diagnostics.iter().any(|d| is_parser_error_code(&d.code));
    assert!(
        !has_parser_error,
        "Should not have parser errors for valid keyword usage: {:?}",
        diagnostics
    );
}

// ============================================================================
// Edge Cases - Keywords in valid contexts
// ============================================================================

#[test]
fn test_keywords_in_strings_allowed() {
    let source = r#"let x = "import match let fn";"#;
    let (_program, diagnostics) = parse_source(source);

    // Keywords in strings are fine
    let has_parser_error = diagnostics.iter().any(|d| is_parser_error_code(&d.code));
    assert!(!has_parser_error, "Keywords in strings should be allowed");
}

#[test]
fn test_keywords_in_comments_allowed() {
    let source = "// import match let\nlet x = 1;";
    let (_program, diagnostics) = parse_source(source);

    // Keywords in comments are fine
    let has_parser_error = diagnostics.iter().any(|d| is_parser_error_code(&d.code));
    assert!(!has_parser_error, "Keywords in comments should be allowed");
}

// ============================================================================
// Error Message Quality Tests
// ============================================================================

#[test]
fn test_error_message_mentions_keyword_and_reserved() {
    let (_program, diagnostics) = parse_source("let import = 1;");

    assert!(!diagnostics.is_empty(), "Expected error");
    assert_parse_error_present(&diagnostics);

    // Error message should mention 'import' keyword and that it's reserved
    assert!(
        diagnostics
            .iter()
            .any(|d| d.message.contains("import") && d.message.contains("reserved")),
        "Expected error message to mention 'import' as reserved keyword, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_error_message_for_future_keyword_mentions_future() {
    let (_program, diagnostics) = parse_source("fn match() {}");

    assert!(!diagnostics.is_empty(), "Expected error");
    assert_parse_error_present(&diagnostics);

    // Error message should mention it's reserved for future use
    assert!(
        diagnostics
            .iter()
            .any(|d| d.message.contains("match") && d.message.contains("future")),
        "Expected error message to mention 'match' is reserved for future use, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// Import syntax tests moved to module_syntax_tests.rs (BLOCKER 04-A)
// Imports are now fully supported as of v0.2

// ============================================================================
// Contextual Tests
// ============================================================================

#[test]
fn test_keyword_as_identifier_in_expression() {
    // Trying to reference 'import' as if it were a variable
    let (_program, diagnostics) = parse_source("let x = import;");

    assert!(
        !diagnostics.is_empty(),
        "Expected error for using keyword as expression"
    );
    assert_parse_error_present(&diagnostics);
}

#[test]
fn test_multiple_keyword_errors() {
    let (_program, diagnostics) = parse_source(
        r#"
        let import = 1;
        let match = 2;
    "#,
    );

    // Should have at least 2 errors (one for each invalid use)
    assert!(diagnostics.len() >= 2, "Expected at least 2 errors");

    // All should be reserved keyword errors
    let reserved_count = diagnostics.iter().filter(|d| d.code == "AT1005").count();
    assert!(
        reserved_count >= 2,
        "Expected at least 2 AT1005 errors, got {}",
        reserved_count
    );
}

// ============================================================================
// Additional Valid Uses
// ============================================================================

#[test]
fn test_valid_use_of_boolean_and_null_literals() {
    let source = "let x = true; let y = false; let z = null;";
    let (_program, diagnostics) = parse_source(source);

    assert_eq!(
        diagnostics.len(),
        0,
        "Expected no errors for valid use of boolean/null literals"
    );
}

// ============================================================================
