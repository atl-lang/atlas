//! Integration tests part 4 (lines 1065-1394)

use super::*;

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
