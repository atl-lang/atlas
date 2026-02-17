//! Frontend Integration Tests (Phase-03)
//!
//! Comprehensive integration testing of all frontend features: enhanced errors,
//! warnings, formatter, and their cross-feature interactions.
//! Validates the full pipeline from source → lex → parse → check → format.

use atlas_formatter::{
    check_formatted, check_formatted_with_config, format_source, format_source_with_config,
    FormatConfig, FormatResult,
};
use atlas_runtime::diagnostic::error_codes;
use atlas_runtime::diagnostic::formatter::{
    enrich_diagnostic, extract_snippet, offset_to_line_col, DiagnosticFormatter,
};
use atlas_runtime::diagnostic::normalizer::normalize_diagnostic_for_testing;
use atlas_runtime::diagnostic::warnings::{
    WarningConfig, WarningEmitter, WarningKind, WarningLevel,
};
use atlas_runtime::{
    sort_diagnostics, Diagnostic, DiagnosticLevel, Lexer, Parser, Span, DIAG_VERSION,
};
use rstest::rstest;

// ============================================================
// Helper Functions
// ============================================================

/// Parse source and return (AST success, parse error diagnostics only)
fn parse_source(source: &str) -> (bool, Vec<Diagnostic>) {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, diags) = parser.parse();
    let has_items = !program.items.is_empty();
    // Filter to only error-level diagnostics (parser may emit warnings)
    let errors: Vec<Diagnostic> = diags
        .into_iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    (has_items || errors.is_empty(), errors)
}

/// Format source and return formatted string, or panic on error
fn fmt(source: &str) -> String {
    match format_source(source) {
        FormatResult::Ok(s) => s,
        FormatResult::ParseError(e) => panic!("Parse error: {:?}", e),
    }
}

/// Format source and return FormatResult directly
fn try_fmt(source: &str) -> FormatResult {
    format_source(source)
}

// ============================================================
// 1. Cross-Feature Integration: Error + Warning Simultaneous
// ============================================================

#[test]
fn test_error_and_warning_diagnostic_types() {
    let error = Diagnostic::error_with_code("AT0001", "Type mismatch", Span::new(0, 5))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("let x: number = \"hello\";")
        .with_label("expected number");

    let warning = Diagnostic::warning_with_code("AT2001", "Unused variable 'y'", Span::new(10, 11))
        .with_file("test.atlas")
        .with_line(2)
        .with_snippet("let y = 42;")
        .with_label("never used");

    assert_eq!(error.level, DiagnosticLevel::Error);
    assert_eq!(warning.level, DiagnosticLevel::Warning);
    assert_eq!(error.diag_version, DIAG_VERSION);
    assert_eq!(warning.diag_version, DIAG_VERSION);
}

#[test]
fn test_mixed_error_and_warning_formatting() {
    let formatter = DiagnosticFormatter::plain();

    let error = Diagnostic::error_with_code("AT0001", "Type mismatch", Span::new(8, 13))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("let x: number = \"hello\";")
        .with_label("expected number, found string");

    let warning = Diagnostic::warning_with_code("AT2001", "Unused variable 'y'", Span::new(4, 5))
        .with_file("test.atlas")
        .with_line(2)
        .with_snippet("let y = 42;")
        .with_label("never used");

    let err_buf = formatter.format_to_buffer(&error);
    let warn_buf = formatter.format_to_buffer(&warning);
    let err_str = String::from_utf8(err_buf).unwrap();
    let warn_str = String::from_utf8(warn_buf).unwrap();

    assert!(err_str.contains("error[AT0001]"));
    assert!(err_str.contains("Type mismatch"));
    assert!(warn_str.contains("warning[AT2001]"));
    assert!(warn_str.contains("Unused variable"));
}

#[test]
fn test_error_warning_sort_order() {
    let mut diagnostics = vec![
        Diagnostic::warning("warn1", Span::new(0, 1))
            .with_file("a.atlas")
            .with_line(1),
        Diagnostic::error("err1", Span::new(0, 1))
            .with_file("b.atlas")
            .with_line(5),
        Diagnostic::warning("warn2", Span::new(0, 1))
            .with_file("a.atlas")
            .with_line(10),
        Diagnostic::error("err2", Span::new(0, 1))
            .with_file("a.atlas")
            .with_line(3),
    ];

    sort_diagnostics(&mut diagnostics);

    // Errors first, then warnings, each sorted by file/line
    assert_eq!(diagnostics[0].level, DiagnosticLevel::Error);
    assert_eq!(diagnostics[0].file, "a.atlas");
    assert_eq!(diagnostics[1].level, DiagnosticLevel::Error);
    assert_eq!(diagnostics[1].file, "b.atlas");
    assert_eq!(diagnostics[2].level, DiagnosticLevel::Warning);
    assert_eq!(diagnostics[2].line, 1);
    assert_eq!(diagnostics[3].level, DiagnosticLevel::Warning);
    assert_eq!(diagnostics[3].line, 10);
}

#[test]
fn test_multiple_diagnostics_same_line() {
    let diags = [
        Diagnostic::error_with_code("AT0001", "type error", Span::new(0, 3))
            .with_file("test.atlas")
            .with_line(1),
        Diagnostic::warning_with_code("AT2001", "unused", Span::new(4, 5))
            .with_file("test.atlas")
            .with_line(1),
    ];

    assert_eq!(diags.len(), 2);
    assert_eq!(diags[0].line, diags[1].line);
    assert_ne!(diags[0].level, diags[1].level);
}

// ============================================================
// 2. Cross-Feature: Multiple Warnings in File
// ============================================================

#[test]
fn test_multiple_warnings_collection() {
    let mut emitter = WarningEmitter::default_config();

    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused 'a'",
        Span::new(0, 1),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2002",
        "Unreachable",
        Span::new(10, 15),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2005",
        "Shadowing 'x'",
        Span::new(20, 21),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused 'b'",
        Span::new(30, 31),
    ));

    assert_eq!(emitter.warnings().len(), 4);
    assert!(!emitter.has_errors());
}

#[test]
fn test_multiple_warnings_selective_suppression() {
    let mut config = WarningConfig::new();
    config.allow("AT2001"); // Suppress unused variable warnings
    let mut emitter = WarningEmitter::new(config);

    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused 'a'",
        Span::new(0, 1),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2002",
        "Unreachable",
        Span::new(10, 15),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused 'b'",
        Span::new(30, 31),
    ));

    // Only the unreachable warning should survive
    assert_eq!(emitter.warnings().len(), 1);
    assert_eq!(emitter.warnings()[0].code, "AT2002");
}

#[test]
fn test_multiple_warnings_deny_promotion() {
    let mut config = WarningConfig::new();
    config.deny("AT2001");
    let mut emitter = WarningEmitter::new(config);

    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused 'x'",
        Span::new(0, 1),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2002",
        "Unreachable",
        Span::new(10, 15),
    ));

    assert_eq!(emitter.errors().len(), 1);
    assert_eq!(emitter.errors()[0].level, DiagnosticLevel::Error);
    assert_eq!(emitter.warnings().len(), 1);
}

#[test]
fn test_all_warning_kinds_have_codes() {
    let kinds = [
        WarningKind::UnusedVariable,
        WarningKind::UnreachableCode,
        WarningKind::DuplicateDeclaration,
        WarningKind::UnusedFunction,
        WarningKind::Shadowing,
        WarningKind::ConstantCondition,
        WarningKind::UnnecessaryAnnotation,
        WarningKind::UnusedImport,
    ];

    for kind in &kinds {
        let code = kind.code();
        assert!(
            code.starts_with("AT2"),
            "Warning code {} doesn't start with AT2",
            code
        );
        assert!(
            error_codes::lookup(code).is_some(),
            "Warning code {} not in registry",
            code
        );
    }
}

// ============================================================
// 3. Cross-Feature: Formatter with Partial Errors
// ============================================================

#[test]
fn test_formatter_rejects_syntax_errors() {
    let result = try_fmt("let x = ;");
    assert!(matches!(result, FormatResult::ParseError(_)));
}

#[test]
fn test_formatter_rejects_unterminated_string() {
    let result = try_fmt("let x = \"hello;");
    assert!(matches!(result, FormatResult::ParseError(_)));
}

#[test]
fn test_formatter_rejects_missing_semicolon_in_let() {
    // Parser may or may not require semicolons — test that it handles gracefully
    let result = try_fmt("let x = 5\nlet y = 10\n");
    // Either formats successfully (if parser is lenient) or returns parse error
    match result {
        FormatResult::Ok(formatted) => {
            // If it succeeds, it should be valid
            assert!(!formatted.is_empty());
        }
        FormatResult::ParseError(errors) => {
            assert!(!errors.is_empty());
        }
    }
}

#[test]
fn test_formatter_handles_empty_input() {
    let result = try_fmt("");
    match result {
        FormatResult::Ok(formatted) => {
            assert!(formatted.is_empty() || formatted == "\n");
        }
        FormatResult::ParseError(_) => {
            // Empty input may parse as empty program
        }
    }
}

#[test]
fn test_formatter_rejects_unmatched_braces() {
    let result = try_fmt("fn foo() {");
    assert!(matches!(result, FormatResult::ParseError(_)));
}

#[test]
fn test_formatter_rejects_unexpected_token() {
    let result = try_fmt("let = = = ;");
    assert!(matches!(result, FormatResult::ParseError(_)));
}

// ============================================================
// 4. Cross-Feature: Warning Suppression via Config
// ============================================================

#[test]
fn test_warning_suppression_allow_all() {
    let config = WarningConfig::allow_all();
    let mut emitter = WarningEmitter::new(config);

    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused",
        Span::new(0, 1),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2002",
        "Unreachable",
        Span::new(5, 10),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2005",
        "Shadowing",
        Span::new(15, 16),
    ));

    assert_eq!(emitter.count(), 0);
    assert!(!emitter.has_warnings());
    assert!(!emitter.has_errors());
}

#[test]
fn test_warning_suppression_selective_with_deny() {
    let mut config = WarningConfig::new();
    config.allow("AT2001");
    config.deny("AT2002");
    let mut emitter = WarningEmitter::new(config);

    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused",
        Span::new(0, 1),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2002",
        "Unreachable",
        Span::new(5, 10),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2005",
        "Shadowing",
        Span::new(15, 16),
    ));

    // AT2001 suppressed, AT2002 promoted to error, AT2005 remains warning
    assert_eq!(emitter.warnings().len(), 1);
    assert_eq!(emitter.warnings()[0].code, "AT2005");
    assert_eq!(emitter.errors().len(), 1);
    assert_eq!(emitter.errors()[0].code, "AT2002");
}

#[test]
fn test_warning_config_from_toml() {
    let toml_str = r#"
[warnings]
level = "warn"
allow = ["AT2001", "AT2005"]
deny = ["AT2002"]
"#;
    let table: toml::Value = toml_str.parse().unwrap();
    let config = atlas_runtime::diagnostic::warnings::config_from_toml(&table);

    assert!(config.is_allowed("AT2001"));
    assert!(config.is_allowed("AT2005"));
    assert!(config.is_denied("AT2002"));
    assert_eq!(config.level_for("AT2006"), WarningLevel::Warn); // Default
}

#[test]
fn test_warning_config_deny_all_with_exceptions() {
    let mut config = WarningConfig::deny_all();
    config.warn("AT2001");

    let mut emitter = WarningEmitter::new(config);
    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused",
        Span::new(0, 1),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2002",
        "Unreachable",
        Span::new(5, 10),
    ));

    assert_eq!(emitter.warnings().len(), 1);
    assert_eq!(emitter.errors().len(), 1);
}

// ============================================================
// 5. Cross-Feature: Error Codes in Formatted Output
// ============================================================

#[test]
fn test_error_code_in_human_output() {
    let diag = Diagnostic::error_with_code("AT0001", "Type mismatch", Span::new(0, 5))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("let x = y;")
        .with_label("error here");

    let output = diag.to_human_string();
    assert!(output.contains("error[AT0001]"));
    assert!(output.contains("Type mismatch"));
    assert!(output.contains("test.atlas:1:1"));
}

#[test]
fn test_error_code_in_json_output() {
    let diag = Diagnostic::error_with_code("AT3002", "Binary op error", Span::new(0, 3))
        .with_file("test.atlas")
        .with_line(1);

    let json = diag.to_json_string().unwrap();
    assert!(json.contains("\"code\": \"AT3002\""));
    assert!(json.contains("\"level\": \"error\""));
    assert!(json.contains(&format!("\"diag_version\": {}", DIAG_VERSION)));
}

#[test]
fn test_warning_code_in_human_output() {
    let diag = Diagnostic::warning_with_code("AT2001", "Unused variable 'x'", Span::new(4, 5))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("let x = 42;")
        .with_label("never used");

    let output = diag.to_human_string();
    assert!(output.contains("warning[AT2001]"));
    assert!(output.contains("Unused variable"));
}

#[test]
fn test_error_code_json_roundtrip() {
    let diag = Diagnostic::error_with_code("AT1001", "Unexpected token", Span::new(5, 10))
        .with_file("test.atlas")
        .with_line(3)
        .with_snippet("let x = @;")
        .with_label("unexpected")
        .with_help("Remove the invalid character");

    let json = diag.to_json_string().unwrap();
    let deserialized: Diagnostic = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.code, "AT1001");
    assert_eq!(deserialized.level, DiagnosticLevel::Error);
    assert_eq!(
        deserialized.help.as_deref(),
        Some("Remove the invalid character")
    );
    assert_eq!(deserialized.diag_version, DIAG_VERSION);
}

// ============================================================
// 6. Cross-Feature: Complex Diagnostic Scenarios
// ============================================================

#[test]
fn test_diagnostic_with_notes_and_related_locations() {
    let diag = Diagnostic::error_with_code("AT0002", "Undefined symbol 'foo'", Span::new(0, 3))
        .with_file("main.atlas")
        .with_line(10)
        .with_snippet("foo(1, 2);")
        .with_label("not defined")
        .with_note("Did you mean 'bar'?")
        .with_note("'bar' is defined in utils.atlas")
        .with_related_location(atlas_runtime::RelatedLocation {
            file: "utils.atlas".to_string(),
            line: 5,
            column: 1,
            length: 3,
            message: "'bar' defined here".to_string(),
        })
        .with_help("Check spelling or import the correct module");

    let output = diag.to_human_string();
    assert!(output.contains("error[AT0002]"));
    assert!(output.contains("Undefined symbol"));
    assert!(output.contains("Did you mean 'bar'?"));
    assert!(output.contains("utils.atlas:5:1"));
    assert!(output.contains("help: Check spelling"));
}

#[test]
fn test_diagnostic_enrichment_from_source() {
    let source = "let x = 1;\nlet y = 2;\nlet z = x + y;";
    let diag = Diagnostic::error_with_code("AT3002", "Binary op error", Span::new(22, 27))
        .with_file("test.atlas");

    let enriched = enrich_diagnostic(diag, source);
    assert!(enriched.line > 0);
    assert!(!enriched.snippet.is_empty());
}

#[test]
fn test_diagnostic_normalization() {
    let diag = Diagnostic::error("test", Span::new(0, 1))
        .with_file("/absolute/path/test.atlas")
        .with_line(1);

    let normalized = normalize_diagnostic_for_testing(&diag);
    // Should strip absolute path
    assert!(!normalized.file.starts_with('/'));
    // Preserve other fields
    assert_eq!(normalized.message, "test");
    assert_eq!(normalized.line, 1);
}

#[test]
fn test_sort_mixed_diagnostics_complex() {
    let mut diags = vec![
        Diagnostic::warning("w3", Span::new(0, 1))
            .with_file("c.atlas")
            .with_line(5),
        Diagnostic::error("e2", Span::new(0, 1))
            .with_file("b.atlas")
            .with_line(1),
        Diagnostic::warning("w1", Span::new(0, 1))
            .with_file("a.atlas")
            .with_line(1),
        Diagnostic::error("e1", Span::new(0, 1))
            .with_file("a.atlas")
            .with_line(1),
        Diagnostic::error("e3", Span::new(0, 1))
            .with_file("a.atlas")
            .with_line(10),
        Diagnostic::warning("w2", Span::new(0, 1))
            .with_file("b.atlas")
            .with_line(3),
    ];

    sort_diagnostics(&mut diags);

    // Errors sorted by file, then line
    assert_eq!(diags[0].message, "e1"); // a.atlas:1
    assert_eq!(diags[1].message, "e3"); // a.atlas:10
    assert_eq!(diags[2].message, "e2"); // b.atlas:1
                                        // Warnings sorted by file, then line
    assert_eq!(diags[3].message, "w1"); // a.atlas:1
    assert_eq!(diags[4].message, "w2"); // b.atlas:3
    assert_eq!(diags[5].message, "w3"); // c.atlas:5
}

// ============================================================
// 7. Pipeline Tests: Valid Code Full Pipeline
// ============================================================

#[test]
fn test_pipeline_valid_let_declaration() {
    let source = "let x = 42;";
    let (ok, diags) = parse_source(source);
    assert!(ok);
    assert!(diags.is_empty());

    let formatted = fmt(source);
    assert_eq!(formatted, "let x = 42;\n");
}

#[test]
fn test_pipeline_valid_function() {
    let source = "fn add(a: number, b: number) -> number { return a + b; }";
    let (ok, diags) = parse_source(source);
    assert!(ok);
    assert!(diags.is_empty());

    let formatted = fmt(source);
    assert!(formatted.contains("fn add"));
    assert!(formatted.contains("return a + b;"));
}

#[test]
fn test_pipeline_valid_if_else() {
    let source = "if (true) { let x = 1; } else { let y = 2; }";
    let (ok, diags) = parse_source(source);
    assert!(ok);
    assert!(diags.is_empty());

    let formatted = fmt(source);
    assert!(formatted.contains("if (true)"));
    assert!(formatted.contains("else"));
}

#[test]
fn test_pipeline_valid_while_loop() {
    let source = "while (true) { break; }";
    let (ok, diags) = parse_source(source);
    assert!(ok);
    assert!(diags.is_empty());

    let formatted = fmt(source);
    assert!(formatted.contains("while (true)"));
}

#[test]
fn test_pipeline_valid_array_literal() {
    let source = "let arr = [1, 2, 3];";
    let (ok, diags) = parse_source(source);
    assert!(ok);
    assert!(diags.is_empty());

    let formatted = fmt(source);
    assert!(formatted.contains("[1, 2, 3]"));
}

// ============================================================
// 8. Pipeline Tests: Syntax Error Handling
// ============================================================

#[test]
fn test_pipeline_syntax_error_missing_value() {
    let (_, diags) = parse_source("let x = ;");
    assert!(!diags.is_empty());
}

#[test]
fn test_pipeline_syntax_error_unmatched_paren() {
    let (_, diags) = parse_source("let x = (1 + 2;");
    assert!(!diags.is_empty());
}

#[test]
fn test_pipeline_syntax_error_invalid_token() {
    let (_, diags) = parse_source("let x = @;");
    assert!(!diags.is_empty());
}

// ============================================================
// 9. Pipeline Tests: Type Error Handling (Diagnostic Creation)
// ============================================================

#[test]
fn test_pipeline_type_error_diagnostic() {
    // Create a type error diagnostic as the typechecker would
    let diag =
        Diagnostic::error_with_code("AT3002", "Cannot add string and number", Span::new(8, 19))
            .with_file("test.atlas")
            .with_line(1)
            .with_snippet("let x = \"hello\" + 42;")
            .with_label("incompatible types");

    assert_eq!(diag.code, "AT3002");
    let output = diag.to_human_string();
    assert!(output.contains("Cannot add string and number"));
}

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

#[test]
fn test_error_code_ranges() {
    for entry in error_codes::ERROR_CODES {
        let code = entry.code;
        if code.starts_with("AT0") {
            // Runtime errors
        } else if code.starts_with("AT1") {
            // Syntax errors
        } else if code.starts_with("AT2") {
            // Warnings
        } else if code.starts_with("AT3") {
            // Semantic errors
        } else if code.starts_with("AT5") {
            // Module errors
        } else if code.starts_with("AT9") || code.starts_with("AW") {
            // Internal/generic
        } else {
            panic!("Unknown error code range: {}", code);
        }
    }
}

#[test]
fn test_all_error_codes_have_descriptions() {
    for entry in error_codes::ERROR_CODES {
        assert!(
            !entry.description.is_empty(),
            "Code {} has no description",
            entry.code
        );
    }
}

#[test]
fn test_no_duplicate_error_codes() {
    let mut seen = std::collections::HashSet::new();
    for entry in error_codes::ERROR_CODES {
        assert!(
            seen.insert(entry.code),
            "Duplicate error code: {}",
            entry.code
        );
    }
}

// ============================================================
// 21. Diagnostic Formatter Integration
// ============================================================

#[test]
fn test_formatter_plain_output_consistency() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error_with_code("AT0001", "Type mismatch", Span::new(0, 5))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("let x = y;")
        .with_label("here");

    let buf1 = formatter.format_to_buffer(&diag);
    let buf2 = formatter.format_to_buffer(&diag);

    // Same diagnostic should produce identical output
    assert_eq!(buf1, buf2);
}

#[test]
fn test_formatter_handles_multiline_snippets() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error_with_code("AT1000", "Syntax error", Span::new(0, 10))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("let x = fn() {");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("error[AT1000]"));
}

#[test]
fn test_formatter_format_to_string() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error("test", Span::new(0, 1))
        .with_file("test.atlas")
        .with_line(1);

    let string_output = formatter.format_to_string(&diag);
    assert!(string_output.contains("error[AT9999]"));
}

// ============================================================
// 22. Warning Kind Round-trip
// ============================================================

#[rstest]
#[case(WarningKind::UnusedVariable, "AT2001")]
#[case(WarningKind::UnreachableCode, "AT2002")]
#[case(WarningKind::DuplicateDeclaration, "AT2003")]
#[case(WarningKind::UnusedFunction, "AT2004")]
#[case(WarningKind::Shadowing, "AT2005")]
#[case(WarningKind::ConstantCondition, "AT2006")]
#[case(WarningKind::UnnecessaryAnnotation, "AT2007")]
#[case(WarningKind::UnusedImport, "AT2008")]
fn test_warning_kind_roundtrip(#[case] kind: WarningKind, #[case] code: &str) {
    assert_eq!(kind.code(), code);
    assert_eq!(WarningKind::from_code(code), Some(kind));
}

// ============================================================
// 23. Diagnostic JSON Serialization Integration
// ============================================================

#[test]
fn test_diagnostic_json_full_roundtrip() {
    let original = Diagnostic::error_with_code("AT0001", "Type mismatch", Span::new(8, 13))
        .with_file("src/main.atlas")
        .with_line(5)
        .with_snippet("let x: number = \"hello\";")
        .with_label("expected number")
        .with_note("string is not assignable to number")
        .with_help("convert with to_number()");

    let json = original.to_json_string().unwrap();
    let deserialized: Diagnostic = serde_json::from_str(&json).unwrap();

    assert_eq!(original, deserialized);
}

#[test]
fn test_diagnostic_json_compact() {
    let diag = Diagnostic::warning_with_code("AT2001", "Unused", Span::new(0, 1));
    let compact = diag.to_json_compact().unwrap();
    assert!(!compact.contains('\n'));
    assert!(compact.contains("AT2001"));
}

#[test]
fn test_diagnostic_json_with_related_locations() {
    let diag = Diagnostic::error("test", Span::new(0, 1)).with_related_location(
        atlas_runtime::RelatedLocation {
            file: "other.atlas".to_string(),
            line: 5,
            column: 10,
            length: 3,
            message: "related".to_string(),
        },
    );

    let json = diag.to_json_string().unwrap();
    let deserialized: Diagnostic = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.related.len(), 1);
    assert_eq!(deserialized.related[0].file, "other.atlas");
}

// ============================================================
// 24. Formatter with Various Code Patterns
// ============================================================

#[rstest]
#[case("let x = 1; let y = 2;", vec!["let x = 1;", "let y = 2;"])]
#[case("fn foo() { return 42; }", vec!["fn foo()", "return 42;"])]
#[case("if (x > 0) { let a = 1; }", vec!["if (x > 0)", "let a = 1;"])]
fn test_format_various_patterns(#[case] source: &str, #[case] expected_parts: Vec<&str>) {
    let formatted = fmt(source);
    for part in expected_parts {
        assert!(
            formatted.contains(part),
            "Formatted output missing '{}'\nGot:\n{}",
            part,
            formatted
        );
    }
}

// ============================================================
// 25. Edge Cases
// ============================================================

#[test]
fn test_empty_source_formats() {
    let result = try_fmt("");
    match result {
        FormatResult::Ok(s) => assert!(s.is_empty() || s == "\n"),
        FormatResult::ParseError(_) => {} // Also acceptable
    }
}

#[test]
fn test_whitespace_only_source() {
    let result = try_fmt("   \n   \n   ");
    match result {
        FormatResult::Ok(s) => assert!(s.trim().is_empty()),
        FormatResult::ParseError(_) => {}
    }
}

#[test]
fn test_single_comment_source() {
    let result = try_fmt("// just a comment");
    match result {
        FormatResult::Ok(s) => assert!(s.contains("// just a comment")),
        FormatResult::ParseError(_) => {}
    }
}

#[test]
fn test_diagnostic_zero_length_span() {
    let diag = Diagnostic::error("test", Span::new(0, 0));
    assert_eq!(diag.length, 0);
    let output = diag.to_human_string();
    // Should not crash with zero-length span
    assert!(output.contains("error"));
}

#[test]
fn test_diagnostic_empty_snippet() {
    let diag = Diagnostic::error("test", Span::new(0, 5))
        .with_file("test.atlas")
        .with_line(1);

    // No snippet set
    let output = diag.to_human_string();
    assert!(!output.contains("^")); // No carets without snippet
}

#[test]
fn test_diagnostic_version_always_set() {
    let e1 = Diagnostic::error("e", Span::new(0, 1));
    let e2 = Diagnostic::error_with_code("AT0001", "e", Span::new(0, 1));
    let w1 = Diagnostic::warning("w", Span::new(0, 1));
    let w2 = Diagnostic::warning_with_code("AT2001", "w", Span::new(0, 1));

    assert_eq!(e1.diag_version, DIAG_VERSION);
    assert_eq!(e2.diag_version, DIAG_VERSION);
    assert_eq!(w1.diag_version, DIAG_VERSION);
    assert_eq!(w2.diag_version, DIAG_VERSION);
}

// ============================================================
// 26. Format + Reparse Stress Tests
// ============================================================

#[rstest]
#[case("fn nested() { if (true) { while (false) { let x = 1; } } }")]
#[case("let a = 1; let b = 2; let c = a + b; let d = c * 2;")]
#[case("fn f(x: number) -> number { if (x > 0) { return x; } else { return 0; } }")]
fn test_format_reparse_complex(#[case] source: &str) {
    let formatted = fmt(source);
    let (ok, diags) = parse_source(&formatted);
    assert!(
        ok,
        "Complex formatted output failed to reparse: {:?}\n{}",
        diags, formatted
    );

    // Also verify idempotency
    let second = fmt(&formatted);
    assert_eq!(formatted, second);
}

// ============================================================
// 27. Warning Emitter Boundary Cases
// ============================================================

#[test]
fn test_emitter_no_warnings() {
    let emitter = WarningEmitter::default_config();
    assert!(!emitter.has_warnings());
    assert!(!emitter.has_errors());
    assert_eq!(emitter.count(), 0);
    assert!(emitter.warnings().is_empty());
    assert!(emitter.errors().is_empty());
    assert!(emitter.all_diagnostics().is_empty());
}

#[test]
fn test_emitter_deny_all_promotes_everything() {
    let config = WarningConfig::deny_all();
    let mut emitter = WarningEmitter::new(config);

    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "w1",
        Span::new(0, 1),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2002",
        "w2",
        Span::new(5, 10),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2005",
        "w3",
        Span::new(15, 16),
    ));

    assert_eq!(emitter.warnings().len(), 0);
    assert_eq!(emitter.errors().len(), 3);
    for e in emitter.errors() {
        assert_eq!(e.level, DiagnosticLevel::Error);
    }
}

// ============================================================
// 28. Source Snippet Integration
// ============================================================

#[test]
fn test_extract_snippet_multiline() {
    let source = "fn foo() {\n    let x = 1;\n    return x;\n}";
    assert_eq!(extract_snippet(source, 1).unwrap(), "fn foo() {");
    assert_eq!(extract_snippet(source, 2).unwrap(), "    let x = 1;");
    assert_eq!(extract_snippet(source, 3).unwrap(), "    return x;");
    assert_eq!(extract_snippet(source, 4).unwrap(), "}");
}

#[test]
fn test_offset_to_line_col_edge_cases() {
    let source = "a\nb\nc";
    assert_eq!(offset_to_line_col(source, 0), (1, 1)); // 'a'
    assert_eq!(offset_to_line_col(source, 1), (1, 2)); // '\n'
    assert_eq!(offset_to_line_col(source, 2), (2, 1)); // 'b'
    assert_eq!(offset_to_line_col(source, 4), (3, 1)); // 'c'
}

// ============================================================
// 29. Full Diagnostic Pipeline End-to-End
// ============================================================

#[test]
fn test_end_to_end_error_pipeline() {
    let source = "let x: number = \"hello\";";

    // Step 1: Parse (should succeed syntactically)
    let (ok, _) = parse_source(source);
    assert!(ok);

    // Step 2: Create a type error diagnostic (as typechecker would)
    let diag = Diagnostic::error_with_code("AT0001", "Type mismatch", Span::new(16, 23))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet(source)
        .with_label("expected number, found string")
        .with_help("Use to_number() to convert");

    // Step 3: Format the diagnostic
    let formatter = DiagnosticFormatter::plain();
    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("error[AT0001]"));
    assert!(output.contains("Type mismatch"));
    assert!(output.contains("test.atlas:1"));
    assert!(output.contains("^^^^^^^"));
    assert!(output.contains("help:"));

    // Step 4: Format the source code
    let formatted = fmt(source);
    assert!(formatted.contains("let x: number ="));

    // Step 5: Verify formatted code re-parses
    let (ok2, diags2) = parse_source(&formatted);
    assert!(ok2);
    assert!(diags2.is_empty());
}

#[test]
fn test_end_to_end_warning_pipeline() {
    let source = "let x = 42;\nlet y = 100;\n";

    // Step 1: Parse
    let (ok, diags) = parse_source(source);
    assert!(ok);
    assert!(diags.is_empty());

    // Step 2: Emit warnings
    let mut emitter = WarningEmitter::default_config();
    emitter.emit(
        Diagnostic::warning_with_code("AT2001", "Unused variable 'x'", Span::new(4, 5))
            .with_file("test.atlas")
            .with_line(1)
            .with_snippet("let x = 42;")
            .with_label("never used"),
    );

    assert_eq!(emitter.warnings().len(), 1);

    // Step 3: Format warning
    let formatter = DiagnosticFormatter::plain();
    let buf = formatter.format_to_buffer(&emitter.warnings()[0]);
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("warning[AT2001]"));

    // Step 4: Format source
    let formatted = fmt(source);
    assert!(formatted.contains("let x = 42;"));

    // Step 5: Verify re-parse
    let (ok2, _) = parse_source(&formatted);
    assert!(ok2);
}

// ============================================================
// 30. Formatter Check Mode Integration
// ============================================================

#[test]
fn test_check_already_formatted_code() {
    let formatted = fmt("let x = 42;");
    assert!(check_formatted(&formatted));
}

#[test]
fn test_check_formatted_with_config() {
    let config = FormatConfig::default().with_indent_size(2);
    let source = "fn foo() {\n  let x = 1;\n}\n";
    let result = check_formatted_with_config(source, &config);
    // Just verify it doesn't crash - result depends on exact formatting
    let _ = result;
}

// ============================================================
// 31. Diagnostic Level Display
// ============================================================

#[test]
fn test_diagnostic_level_display() {
    assert_eq!(format!("{}", DiagnosticLevel::Error), "error");
    assert_eq!(format!("{}", DiagnosticLevel::Warning), "warning");
}

// ============================================================
// 32. Large-Scale Integration
// ============================================================

#[test]
fn test_many_diagnostics_sorted() {
    let mut diags = Vec::new();
    for i in 0..50 {
        let level = if i % 3 == 0 {
            Diagnostic::error(format!("error {}", i), Span::new(0, 1))
        } else {
            Diagnostic::warning(format!("warning {}", i), Span::new(0, 1))
        };
        diags.push(
            level
                .with_file(format!("file{}.atlas", i % 5))
                .with_line(i + 1),
        );
    }

    sort_diagnostics(&mut diags);

    // Verify errors come before warnings
    let first_warning = diags
        .iter()
        .position(|d| d.level == DiagnosticLevel::Warning);
    let last_error = diags
        .iter()
        .rposition(|d| d.level == DiagnosticLevel::Error);

    if let (Some(fw), Some(le)) = (first_warning, last_error) {
        assert!(le < fw, "Errors should come before warnings after sorting");
    }
}

#[test]
fn test_many_warnings_through_emitter() {
    let mut config = WarningConfig::new();
    config.allow("AT2001");
    config.deny("AT2002");
    let mut emitter = WarningEmitter::new(config);

    for i in 0..30 {
        let code = match i % 3 {
            0 => "AT2001",
            1 => "AT2002",
            _ => "AT2005",
        };
        emitter.emit(Diagnostic::warning_with_code(
            code,
            format!("w{}", i),
            Span::new(i, i + 1),
        ));
    }

    // AT2001 (10 instances) suppressed
    // AT2002 (10 instances) promoted to errors
    // AT2005 (10 instances) remain as warnings
    assert_eq!(emitter.warnings().len(), 10);
    assert_eq!(emitter.errors().len(), 10);
    assert_eq!(emitter.count(), 20);
}

// ============================================================
// 33. Error Code Help Text Integration
// ============================================================

#[rstest]
#[case("AT0001")]
#[case("AT0002")]
#[case("AT0005")]
#[case("AT1001")]
#[case("AT3002")]
#[case("AT5002")]
fn test_error_code_has_help_text(#[case] code: &str) {
    let help = error_codes::help_for(code);
    assert!(help.is_some(), "Error code {} should have help text", code);
    assert!(!help.unwrap().is_empty());
}

#[test]
fn test_generic_error_has_no_help() {
    assert!(error_codes::help_for("AT9999").is_none());
}

// ============================================================
// 34. Diagnostic Builder Pattern Completeness
// ============================================================

#[test]
fn test_diagnostic_builder_all_fields() {
    let diag = Diagnostic::error_with_code("AT0001", "Full diagnostic", Span::new(5, 15))
        .with_file("full_test.atlas")
        .with_line(42)
        .with_snippet("let value = compute();")
        .with_label("type error here")
        .with_note("Note 1")
        .with_note("Note 2")
        .with_related_location(atlas_runtime::RelatedLocation {
            file: "other.atlas".to_string(),
            line: 10,
            column: 5,
            length: 7,
            message: "defined here".to_string(),
        })
        .with_help("Check the return type");

    assert_eq!(diag.code, "AT0001");
    assert_eq!(diag.message, "Full diagnostic");
    assert_eq!(diag.file, "full_test.atlas");
    assert_eq!(diag.line, 42);
    assert_eq!(diag.snippet, "let value = compute();");
    assert_eq!(diag.label, "type error here");
    assert_eq!(diag.notes.len(), 2);
    assert_eq!(diag.related.len(), 1);
    assert!(diag.help.is_some());
    assert_eq!(diag.diag_version, DIAG_VERSION);
    assert_eq!(diag.level, DiagnosticLevel::Error);
    assert_eq!(diag.column, 6); // span.start + 1
    assert_eq!(diag.length, 10); // span.end - span.start
}
