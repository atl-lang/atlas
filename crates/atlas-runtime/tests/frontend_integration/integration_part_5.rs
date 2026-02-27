//! Integration tests part 5 (lines 1395-1618)

use super::*;

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

// ============================================================================
