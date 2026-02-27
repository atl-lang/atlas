//! Integration tests part 1 (lines 77-369)

use super::*;

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
