//! Integration tests part 2 (lines 370-697)

use super::*;

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
        .with_file(absolute_test_path("test.atlas"))
        .with_line(1);

    let normalized = normalize_diagnostic_for_testing(&diag);
    // Should strip absolute path
    assert!(
        !is_absolute_path(&normalized.file),
        "Path should be normalized: {}",
        normalized.file
    );
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
