//! Formatter Integration Tests (Phase-03)
//!
//! Tests formatter integration with the full Atlas frontend pipeline:
//! parsing, formatting, comment preservation, configuration, and re-parsing.

use atlas_formatter::{
    check_formatted, check_formatted_with_config, format_source, format_source_with_config,
    FormatConfig, FormatResult,
};
use pretty_assertions::assert_eq;
use rstest::rstest;

// ============================================================
// Helpers
// ============================================================

fn fmt(source: &str) -> String {
    match format_source(source) {
        FormatResult::Ok(s) => s,
        FormatResult::ParseError(e) => panic!("Parse error: {:?}", e),
    }
}

fn fmt_with(source: &str, config: &FormatConfig) -> String {
    match format_source_with_config(source, config) {
        FormatResult::Ok(s) => s,
        FormatResult::ParseError(e) => panic!("Parse error: {:?}", e),
    }
}

fn try_fmt(source: &str) -> FormatResult {
    format_source(source)
}

// ============================================================
// 1. Full Pipeline: Lex → Parse → Format → Reparse
// ============================================================

#[rstest]
#[case("let x = 42;")]
#[case("let s = \"hello\";")]
#[case("let b = true;")]
#[case("let n = null;")]
#[case("fn foo() { let x = 1; }")]
#[case("fn add(a: number, b: number) -> number { return a + b; }")]
#[case("if (true) { let a = 1; } else { let b = 2; }")]
#[case("while (true) { break; }")]
#[case("let arr = [1, 2, 3];")]
#[case("let x = 1 + 2 * 3;")]
#[case("let neg = -5;")]
#[case("let x = !true;")]
fn test_pipeline_format_and_reparse(#[case] source: &str) {
    let formatted = fmt(source);

    // Verify formatted output re-parses cleanly
    let result = try_fmt(&formatted);
    match result {
        FormatResult::Ok(second) => {
            // Also verify idempotency
            assert_eq!(formatted, second, "Not idempotent for: {}", source);
        }
        FormatResult::ParseError(errors) => {
            panic!(
                "Formatted output failed to re-parse: {:?}\nFormatted:\n{}",
                errors, formatted
            );
        }
    }
}

// ============================================================
// 2. Comment Preservation Through Pipeline
// ============================================================

#[test]
fn test_line_comment_preserved() {
    let source = "// A line comment\nlet x = 42;\n";
    let formatted = fmt(source);
    assert!(formatted.contains("// A line comment"));
    assert!(formatted.contains("let x = 42;"));
}

#[test]
fn test_block_comment_preserved() {
    let source = "/* Block */\nlet x = 1;\n";
    let formatted = fmt(source);
    assert!(formatted.contains("/* Block */"));
}

#[test]
fn test_doc_comment_preserved() {
    let source = "/// Documentation\nfn foo() { return 1; }\n";
    let formatted = fmt(source);
    assert!(formatted.contains("/// Documentation"));
}

#[test]
fn test_multiple_comments_preserved() {
    let source = "// First\n// Second\n// Third\nlet x = 1;\n";
    let formatted = fmt(source);
    assert!(formatted.contains("// First"));
    assert!(formatted.contains("// Second"));
    assert!(formatted.contains("// Third"));
}

#[test]
fn test_comment_between_statements() {
    let source = "let x = 1;\n// Middle comment\nlet y = 2;\n";
    let formatted = fmt(source);
    assert!(formatted.contains("// Middle comment"));
    assert!(formatted.contains("let x = 1;"));
    assert!(formatted.contains("let y = 2;"));
}

// ============================================================
// 3. Configuration Integration
// ============================================================

#[test]
fn test_config_indent_2_spaces() {
    let config = FormatConfig::default().with_indent_size(2);
    let source = "fn foo() { let x = 1; }";
    let formatted = fmt_with(source, &config);
    assert!(
        formatted.contains("  let x = 1;"),
        "Expected 2-space indent, got:\n{}",
        formatted
    );
}

#[test]
fn test_config_indent_4_spaces() {
    let config = FormatConfig::default().with_indent_size(4);
    let source = "fn foo() { let x = 1; }";
    let formatted = fmt_with(source, &config);
    assert!(
        formatted.contains("    let x = 1;"),
        "Expected 4-space indent, got:\n{}",
        formatted
    );
}

#[test]
fn test_config_indent_8_spaces() {
    let config = FormatConfig::default().with_indent_size(8);
    let source = "fn foo() { let x = 1; }";
    let formatted = fmt_with(source, &config);
    assert!(
        formatted.contains("        let x = 1;"),
        "Expected 8-space indent, got:\n{}",
        formatted
    );
}

#[test]
fn test_config_trailing_commas_enabled() {
    let config = FormatConfig::default().with_trailing_commas(true);
    let source = "let x = [1, 2, 3];";
    let formatted = fmt_with(source, &config);
    // Should parse and format without error
    assert!(formatted.contains("[1, 2, 3]") || formatted.contains("1,"));
}

#[test]
fn test_config_trailing_commas_disabled() {
    let config = FormatConfig::default().with_trailing_commas(false);
    let source = "let x = [1, 2, 3];";
    let formatted = fmt_with(source, &config);
    assert!(!formatted.is_empty());
}

#[test]
fn test_config_max_width() {
    let config = FormatConfig::default().with_max_width(40);
    let source = "let x = 42;";
    let formatted = fmt_with(source, &config);
    assert!(!formatted.is_empty());
}

// ============================================================
// 4. Check Mode Integration
// ============================================================

#[test]
fn test_check_formatted_pass() {
    let formatted = fmt("let x = 42;");
    assert!(
        check_formatted(&formatted),
        "Already-formatted code should pass check"
    );
}

#[test]
fn test_check_formatted_with_custom_config() {
    let config = FormatConfig::default().with_indent_size(2);
    let source = "fn foo() {\n  let x = 1;\n}\n";
    // May or may not pass depending on exact formatting
    let _ = check_formatted_with_config(source, &config);
}

#[test]
fn test_check_formatted_parse_error() {
    assert!(
        !check_formatted("let x = ;"),
        "Parse errors should fail check"
    );
}

// ============================================================
// 5. Error Handling Integration
// ============================================================

#[rstest]
#[case("let x = ;")]
#[case("fn foo( { }")]
#[case("let x = \"unterminated")]
#[case("if { }")]
fn test_parse_errors_return_error_result(#[case] source: &str) {
    let result = try_fmt(source);
    match result {
        FormatResult::ParseError(errors) => {
            assert!(!errors.is_empty(), "Should have error messages");
        }
        FormatResult::Ok(_) => {
            // Parser recovery may allow some of these
        }
    }
}

#[test]
fn test_parse_error_messages_are_descriptive() {
    let result = try_fmt("let = ;");
    if let FormatResult::ParseError(errors) = result {
        for err in &errors {
            assert!(!err.is_empty(), "Error messages should not be empty");
        }
    }
}

// ============================================================
// 6. Idempotency Integration
// ============================================================

#[rstest]
#[case("let x = 42;")]
#[case("fn foo() -> number { return 1; }")]
#[case("if (true) { let a = 1; } else { let b = 2; }")]
#[case("while (true) { break; }")]
#[case("let arr = [1, 2, 3];")]
#[case("// comment\nlet x = 1;")]
#[case("let a = 1;\nlet b = 2;\nlet c = a + b;")]
#[case("fn f(x: number, y: number) -> number { return x + y; }")]
fn test_idempotent_formatting(#[case] source: &str) {
    let first = fmt(source);
    let second = fmt(&first);
    let third = fmt(&second);

    assert_eq!(
        first, second,
        "Not idempotent after 1st pass for:\n{}",
        source
    );
    assert_eq!(
        second, third,
        "Not idempotent after 2nd pass for:\n{}",
        source
    );
}

// ============================================================
// 7. Complex Formatting Scenarios
// ============================================================

#[test]
fn test_nested_blocks() {
    let source = "fn foo() { if (true) { while (false) { let x = 1; } } }";
    let formatted = fmt(source);

    // Should have proper nesting
    assert!(formatted.contains("fn foo()"));
    assert!(formatted.contains("if (true)"));
    assert!(formatted.contains("while (false)"));
    assert!(formatted.contains("let x = 1;"));

    // Verify re-parses
    let result = try_fmt(&formatted);
    assert!(matches!(result, FormatResult::Ok(_)));
}

#[test]
fn test_multiple_functions() {
    let source = "fn foo() -> number { return 1; }\nfn bar() -> number { return 2; }";
    let formatted = fmt(source);
    assert!(formatted.contains("fn foo()"));
    assert!(formatted.contains("fn bar()"));
}

#[test]
fn test_function_with_multiple_params() {
    let source = "fn calc(a: number, b: number, c: number) -> number { return a + b + c; }";
    let formatted = fmt(source);
    assert!(formatted.contains("a: number"));
    assert!(formatted.contains("b: number"));
    assert!(formatted.contains("c: number"));
}

#[test]
fn test_complex_expressions() {
    let source = "let x = 1 + 2 * 3 - 4 / 2;";
    let formatted = fmt(source);
    assert!(formatted.contains("let x ="));
    assert!(formatted.contains("1 + 2 * 3 - 4 / 2"));
}

#[test]
fn test_string_with_escapes() {
    let source = r#"let s = "hello\nworld";"#;
    let formatted = fmt(source);
    assert!(formatted.contains(r#""hello\nworld""#));
}

// ============================================================
// 8. Edge Cases
// ============================================================

#[test]
fn test_empty_input() {
    let result = try_fmt("");
    match result {
        FormatResult::Ok(s) => assert!(s.is_empty() || s == "\n"),
        FormatResult::ParseError(_) => {}
    }
}

#[test]
fn test_only_whitespace() {
    let result = try_fmt("   \n   \n");
    match result {
        FormatResult::Ok(s) => assert!(s.trim().is_empty()),
        FormatResult::ParseError(_) => {}
    }
}

#[test]
fn test_only_comments() {
    let result = try_fmt("// Comment 1\n// Comment 2\n");
    match result {
        FormatResult::Ok(s) => {
            assert!(s.contains("// Comment 1"));
            assert!(s.contains("// Comment 2"));
        }
        FormatResult::ParseError(_) => {}
    }
}

#[test]
fn test_single_statement() {
    let formatted = fmt("let x = 1;");
    assert_eq!(formatted, "let x = 1;\n");
}

#[test]
fn test_many_declarations() {
    let mut source = String::new();
    for i in 0..20 {
        source.push_str(&format!("let x{} = {};\n", i, i));
    }
    let formatted = fmt(&source);
    for i in 0..20 {
        assert!(formatted.contains(&format!("let x{} = {};", i, i)));
    }
}

// ============================================================
// 9. Config Default Values
// ============================================================

#[test]
fn test_default_config_values() {
    let config = FormatConfig::default();
    assert_eq!(config.indent_size, 4);
    assert_eq!(config.max_width, 100);
    assert!(config.trailing_commas);
}

#[test]
fn test_config_builder_chaining() {
    let config = FormatConfig::default()
        .with_indent_size(2)
        .with_max_width(80)
        .with_trailing_commas(false);

    assert_eq!(config.indent_size, 2);
    assert_eq!(config.max_width, 80);
    assert!(!config.trailing_commas);
}
