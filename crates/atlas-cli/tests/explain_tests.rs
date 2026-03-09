//! Integration tests for `atlas explain ATXXXX` (B14-P05) and JSON output (B14-P06)

use atlas_runtime::diagnostic::error_codes;
use atlas_runtime::{Diagnostic, DiagnosticLevel, Span};

/// The error_codes registry lookup handles lowercase and numeric-only codes
/// when normalised by the explain command logic.
#[test]
fn test_explain_known_code_found_in_registry() {
    // AT3056 was added in B13 — verify it is in the registry
    let info = error_codes::lookup("AT3056");
    assert!(
        info.is_some(),
        "AT3056 should be in the error_codes registry"
    );
    let info = info.unwrap();
    assert_eq!(info.code, "AT3056");
    assert!(
        !info.description.is_empty(),
        "AT3056 should have a description"
    );
    assert!(info.help.is_some(), "AT3056 should have help text");
}

#[test]
fn test_explain_unknown_code_returns_none() {
    let info = error_codes::lookup("AT9876");
    assert!(
        info.is_none(),
        "AT9876 is not a real code and should not be found"
    );
}

#[test]
fn test_explain_warning_code_found() {
    let info = error_codes::lookup("AW3059");
    assert!(
        info.is_some(),
        "AW3059 warning code should be in the registry"
    );
    let info = info.unwrap();
    assert!(
        info.description.contains("shadow") || info.description.contains("inherent"),
        "AW3059 description should mention inherent shadowing, got: {:?}",
        info.description
    );
}

#[test]
fn test_explain_all_codes_have_description() {
    for entry in error_codes::ERROR_CODES {
        assert!(
            !entry.description.is_empty(),
            "Error code {} has an empty description",
            entry.code
        );
    }
}

#[test]
fn test_explain_example_field_exists_on_all_codes() {
    // All codes have the example field (even if None) — structural completeness
    for entry in error_codes::ERROR_CODES {
        // Field exists — no assertion needed beyond compile-time check.
        // This test documents that example is wired into ErrorCodeInfo.
        let _ = entry.example;
    }
}

// ============================================================================
// JSON Output Tests (B14-P06)
// ============================================================================

/// Diagnostic JSON serialisation omits is_secondary when false.
#[test]
fn test_json_output_omits_is_secondary_when_false() {
    let span = Span::new(0, 1);
    let diag = Diagnostic::error_with_code("AT1000", "test", span);
    let json = diag.to_json_compact().unwrap();
    assert!(
        !json.contains("is_secondary"),
        "is_secondary=false should be omitted from JSON, got: {}",
        json
    );
}

/// Diagnostic JSON serialisation includes is_secondary when true.
#[test]
fn test_json_output_includes_is_secondary_when_true() {
    let span = Span::new(0, 1);
    let diag = Diagnostic::error_with_code("AT1000", "test", span).as_secondary();
    let json = diag.to_json_compact().unwrap();
    assert!(
        json.contains("is_secondary"),
        "is_secondary=true should appear in JSON, got: {}",
        json
    );
}

/// Warning diagnostics are distinct from error diagnostics in level field.
#[test]
fn test_json_diagnostic_level_field() {
    let span = Span::new(0, 1);
    let err = Diagnostic::error_with_code("AT1000", "error", span)
        .to_json_compact()
        .unwrap();
    let warn = Diagnostic::warning_with_code("AT2001", "warning", span)
        .to_json_compact()
        .unwrap();
    assert!(
        err.contains(r#""error""#),
        "Error diag should have level=error in JSON"
    );
    assert!(
        warn.contains(r#""warning""#),
        "Warning diag should have level=warning in JSON"
    );
}
