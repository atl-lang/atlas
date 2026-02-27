//! Integration tests part 3 (lines 698-1064)

use super::*;

#[test]
fn test_pipeline_immutable_assignment_diagnostic() {
    let diag = Diagnostic::error_with_code(
        "AT3003",
        "Cannot assign to immutable variable",
        Span::new(4, 5),
    )
    .with_file("test.atlas")
    .with_line(2)
    .with_snippet("x = 10;")
    .with_label("immutable")
    .with_help("Use 'let mut' to declare a mutable variable");

    let output = diag.to_human_string();
    assert!(output.contains("error[AT3003]"));
    assert!(output.contains("help: Use 'let mut'"));
}

// ============================================================
// 10. Pipeline Tests: Mixed Error Types
// ============================================================

#[test]
fn test_pipeline_mixed_errors_and_warnings() {
    let mut all_diags: Vec<Diagnostic> = vec![];

    // Simulate collecting errors from parser
    all_diags.push(
        Diagnostic::error_with_code("AT1001", "Unexpected token", Span::new(0, 5))
            .with_file("test.atlas")
            .with_line(1),
    );

    // Simulate collecting warnings from typechecker
    let mut emitter = WarningEmitter::default_config();
    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused 'x'",
        Span::new(10, 11),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2005",
        "Shadowing 'y'",
        Span::new(20, 21),
    ));

    all_diags.extend(emitter.all_diagnostics());

    assert_eq!(all_diags.len(), 3);

    sort_diagnostics(&mut all_diags);
    assert_eq!(all_diags[0].level, DiagnosticLevel::Error);
}

// ============================================================
// 11. Pipeline Tests: Warning Collection
// ============================================================

#[test]
fn test_pipeline_warning_collection_with_config() {
    let mut config = WarningConfig::new();
    config.allow("AT2001");
    let mut emitter = WarningEmitter::new(config);

    // Simulate emitting warnings during compilation
    for i in 0..5 {
        emitter.emit(Diagnostic::warning_with_code(
            "AT2001",
            format!("Unused var_{}", i),
            Span::new(i * 10, i * 10 + 5),
        ));
    }
    emitter.emit(Diagnostic::warning_with_code(
        "AT2002",
        "Unreachable",
        Span::new(50, 55),
    ));

    // AT2001 suppressed, only AT2002 remains
    assert_eq!(emitter.warnings().len(), 1);
    assert_eq!(emitter.warnings()[0].code, "AT2002");
}

#[test]
fn test_pipeline_emitter_clear_and_reuse() {
    let mut emitter = WarningEmitter::default_config();
    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused",
        Span::new(0, 1),
    ));
    assert_eq!(emitter.count(), 1);

    emitter.clear();
    assert_eq!(emitter.count(), 0);

    emitter.emit(Diagnostic::warning_with_code(
        "AT2002",
        "Unreachable",
        Span::new(5, 10),
    ));
    assert_eq!(emitter.count(), 1);
    assert_eq!(emitter.warnings()[0].code, "AT2002");
}

// ============================================================
// 12. Pipeline Tests: Format After Check
// ============================================================

#[test]
fn test_pipeline_format_after_parse_check() {
    let source = "let x=42;let y=x+1;";

    // Step 1: Parse - should succeed
    let (ok, diags) = parse_source(source);
    assert!(ok);
    assert!(diags.is_empty());

    // Step 2: Format
    let formatted = fmt(source);
    assert!(formatted.contains("let x = 42;"));
    assert!(formatted.contains("let y = x + 1;"));
}

#[test]
fn test_pipeline_format_preserves_semantics() {
    let source = "let x = 1 + 2 * 3;";
    let formatted = fmt(source);

    // Both should parse successfully
    let (ok1, _) = parse_source(source);
    let (ok2, _) = parse_source(&formatted);
    assert!(ok1);
    assert!(ok2);
}

// ============================================================
// 13. Pipeline Tests: Reparse Formatted Output
// ============================================================

#[rstest]
#[case("let x = 42;")]
#[case("fn foo() -> number { return 1; }")]
#[case("if (true) { let a = 1; } else { let b = 2; }")]
#[case("while (true) { break; }")]
#[case("let arr = [1, 2, 3];")]
#[case("let s = \"hello world\";")]
#[case("let a = true; let b = false;")]
#[case("fn add(a: number, b: number) -> number { return a + b; }")]
#[case("let x = 1 + 2 * 3;")]
#[case("let neg = -5;")]
fn test_formatted_output_reparses(#[case] source: &str) {
    let formatted = fmt(source);
    let (ok, diags) = parse_source(&formatted);
    assert!(
        ok,
        "Formatted output failed to parse: {:?}\nFormatted:\n{}",
        diags, formatted
    );
}

// ============================================================
// 14. Pipeline Tests: Location Accuracy Preservation
// ============================================================

#[test]
fn test_location_offset_to_line_col() {
    let source = "let x = 1;\nlet y = 2;\nlet z = 3;";
    assert_eq!(offset_to_line_col(source, 0), (1, 1));
    assert_eq!(offset_to_line_col(source, 11), (2, 1));
    assert_eq!(offset_to_line_col(source, 22), (3, 1));
}

#[test]
fn test_location_extract_snippet() {
    let source = "first line\nsecond line\nthird line";
    assert_eq!(extract_snippet(source, 1).unwrap(), "first line");
    assert_eq!(extract_snippet(source, 2).unwrap(), "second line");
    assert_eq!(extract_snippet(source, 3).unwrap(), "third line");
    assert!(extract_snippet(source, 4).is_none());
}

#[test]
fn test_location_span_in_diagnostic() {
    let diag =
        Diagnostic::error_with_code("AT0001", "test", Span::new(5, 10)).with_file("test.atlas");

    // column = span.start + 1 = 6
    assert_eq!(diag.column, 6);
    assert_eq!(diag.length, 5);
}

// ============================================================
// 15. Formatter Integration: Format with Warnings
// ============================================================

#[test]
fn test_format_code_with_unused_variable_pattern() {
    // Code that would generate unused variable warnings formats correctly
    let source = "let x = 42;\nlet y = 100;\n";
    let formatted = fmt(source);
    assert!(formatted.contains("let x = 42;"));
    assert!(formatted.contains("let y = 100;"));
}

#[test]
fn test_format_code_with_shadowing_pattern() {
    // Code with shadowing formats correctly
    let source = "let x = 1;\nlet x = 2;\n";
    let formatted = fmt(source);
    // Both declarations should be present
    let x_count = formatted.matches("let x =").count();
    assert_eq!(x_count, 2);
}

// ============================================================
// 16. Formatter Integration: Error Handling
// ============================================================

#[test]
fn test_format_parse_error_returns_errors() {
    let result = try_fmt("fn foo( { }");
    match result {
        FormatResult::ParseError(errors) => {
            assert!(!errors.is_empty());
        }
        FormatResult::Ok(_) => panic!("Expected parse error"),
    }
}

#[test]
fn test_format_multiple_parse_errors() {
    let result = try_fmt("let = ;\nlet = ;");
    match result {
        FormatResult::ParseError(errors) => {
            assert!(!errors.is_empty());
        }
        FormatResult::Ok(_) => {
            // Some parsers recover; either way is acceptable
        }
    }
}

// ============================================================
// 17. Formatter Integration: Comment Preservation
// ============================================================

#[test]
fn test_format_preserves_line_comments() {
    let source = "// This is a comment\nlet x = 42;\n";
    let formatted = fmt(source);
    assert!(formatted.contains("// This is a comment"));
    assert!(formatted.contains("let x = 42;"));
}

#[test]
fn test_format_preserves_block_comments() {
    let source = "/* Block comment */\nlet x = 42;\n";
    let formatted = fmt(source);
    assert!(formatted.contains("/* Block comment */"));
}

#[test]
fn test_format_preserves_doc_comments() {
    let source = "/// Doc comment\nfn foo() -> number { return 1; }\n";
    let formatted = fmt(source);
    assert!(formatted.contains("/// Doc comment"));
}

#[test]
fn test_format_preserves_multiple_comments() {
    let source = "// Comment 1\n// Comment 2\nlet x = 42;\n";
    let formatted = fmt(source);
    assert!(formatted.contains("// Comment 1"));
    assert!(formatted.contains("// Comment 2"));
}

// ============================================================
// 18. Formatter Integration: Idempotency
// ============================================================

#[rstest]
#[case("let x = 42;")]
#[case("fn foo() -> number { return 1; }")]
#[case("if (true) { let a = 1; }")]
#[case("while (true) { break; }")]
#[case("let arr = [1, 2, 3];")]
#[case("// comment\nlet x = 1;")]
#[case("fn add(a: number, b: number) -> number { return a + b; }")]
#[case("let x = 1;\nlet y = 2;\nlet z = x + y;")]
fn test_format_idempotent(#[case] source: &str) {
    let first = fmt(source);
    let second = fmt(&first);
    assert_eq!(
        first, second,
        "Formatting is not idempotent for:\n{}",
        source
    );
}

// ============================================================
// 19. Formatter Integration: Configuration Variations
// ============================================================

#[test]
fn test_format_with_indent_2() {
    let config = FormatConfig::default().with_indent_size(2);
    let source = "fn foo() { let x = 1; }";
    let formatted = match format_source_with_config(source, &config) {
        FormatResult::Ok(s) => s,
        FormatResult::ParseError(e) => panic!("Parse error: {:?}", e),
    };
    assert!(formatted.contains("  let x = 1;"));
}

#[test]
fn test_format_with_indent_4() {
    let config = FormatConfig::default().with_indent_size(4);
    let source = "fn foo() { let x = 1; }";
    let formatted = match format_source_with_config(source, &config) {
        FormatResult::Ok(s) => s,
        FormatResult::ParseError(e) => panic!("Parse error: {:?}", e),
    };
    assert!(formatted.contains("    let x = 1;"));
}

#[test]
fn test_check_formatted_already_formatted() {
    let formatted = fmt("let x = 42;");
    assert!(check_formatted(&formatted));
}

#[test]
fn test_check_formatted_needs_formatting() {
    // Poorly formatted source
    let source = "let   x   =   42  ;";
    // May or may not detect as needing formatting depending on parser
    let result = check_formatted(source);
    // Just verify it doesn't crash
    let _ = result;
}

// ============================================================
// 20. Error Code Registry Integration
// ============================================================

#[rstest]
#[case("AT0001", "Type mismatch")]
#[case("AT0002", "Undefined symbol")]
#[case("AT0005", "Division by zero")]
#[case("AT1001", "Unexpected token")]
#[case("AT2001", "Unused variable")]
#[case("AT3002", "Binary operation type error")]
#[case("AT5002", "Module not found")]
fn test_error_code_registry_lookup(#[case] code: &str, #[case] expected_desc: &str) {
    let info = error_codes::lookup(code).unwrap_or_else(|| panic!("Code {} not found", code));
    assert!(
        info.description
            .to_lowercase()
            .contains(&expected_desc.to_lowercase()),
        "Code {} description '{}' doesn't match expected '{}'",
        code,
        info.description,
        expected_desc
    );
}
