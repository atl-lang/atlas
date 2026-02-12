//! Golden tests for diagnostic output
//!
//! These tests verify that diagnostics are formatted correctly and
//! consistently across different machines using normalization.

use atlas_runtime::diagnostic::{normalizer::normalize_diagnostics_for_testing, Diagnostic};
use atlas_runtime::span::Span;
use std::fs;
use std::path::PathBuf;

/// Load expected diagnostic JSON from a file
fn load_expected_diagnostics(path: &str) -> Vec<Diagnostic> {
    let json = fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read expected diagnostics from {}", path));

    serde_json::from_str(&json)
        .unwrap_or_else(|e| panic!("Failed to parse expected diagnostics from {}: {}", path, e))
}

/// Compare actual diagnostics with expected golden file
fn assert_diagnostics_match_golden(actual: &[Diagnostic], golden_path: &str) {
    let expected = load_expected_diagnostics(golden_path);
    let normalized_actual = normalize_diagnostics_for_testing(actual);

    assert_eq!(
        normalized_actual.len(),
        expected.len(),
        "Different number of diagnostics. Expected {}, got {}",
        expected.len(),
        normalized_actual.len()
    );

    for (i, (actual_diag, expected_diag)) in
        normalized_actual.iter().zip(expected.iter()).enumerate()
    {
        assert_eq!(
            actual_diag.diag_version, expected_diag.diag_version,
            "Diagnostic {} version mismatch",
            i
        );
        assert_eq!(
            actual_diag.level, expected_diag.level,
            "Diagnostic {} level mismatch",
            i
        );
        assert_eq!(
            actual_diag.code, expected_diag.code,
            "Diagnostic {} code mismatch",
            i
        );
        assert_eq!(
            actual_diag.message, expected_diag.message,
            "Diagnostic {} message mismatch",
            i
        );
        assert_eq!(
            actual_diag.file, expected_diag.file,
            "Diagnostic {} file mismatch",
            i
        );
        assert_eq!(
            actual_diag.line, expected_diag.line,
            "Diagnostic {} line mismatch",
            i
        );
        assert_eq!(
            actual_diag.column, expected_diag.column,
            "Diagnostic {} column mismatch",
            i
        );
    }
}

#[test]
fn test_diagnostic_normalization_stability() {
    // Test that normalization produces stable output

    // Create a diagnostic with an absolute path
    let diag = Diagnostic::error("test error", Span::new(0, 5))
        .with_file("/some/absolute/path/test.atlas")
        .with_line(10)
        .with_snippet("test code");

    // Normalize it multiple times
    let normalized1 = normalize_diagnostics_for_testing(&[diag.clone()]);
    let normalized2 = normalize_diagnostics_for_testing(&[diag.clone()]);

    // Should produce identical results
    assert_eq!(normalized1, normalized2);

    // Path should be normalized
    assert_eq!(normalized1[0].file, "test.atlas");
}

#[test]
fn test_diagnostic_json_serialization_stable() {
    // Test that JSON serialization is deterministic

    let diag = Diagnostic::error("test", Span::new(0, 1))
        .with_file("test.atlas")
        .with_line(1);

    let json1 = serde_json::to_string(&diag).unwrap();
    let json2 = serde_json::to_string(&diag).unwrap();

    assert_eq!(json1, json2);
}

#[test]
fn test_diagnostic_normalization_preserves_special_paths() {
    // Test that special paths like <input> are preserved

    let diag = Diagnostic::error("test", Span::new(0, 1))
        .with_file("<input>")
        .with_line(5);

    let normalized = normalize_diagnostics_for_testing(&[diag]);

    assert_eq!(normalized[0].file, "<input>");
}

#[test]
fn test_diagnostic_golden_comparison() {
    // Test comparing diagnostics with a golden file

    let expected_json = r#"[
  {
    "diag_version": 1,
    "level": "error",
    "code": "AT0001",
    "message": "Test error",
    "file": "test.atlas",
    "line": 1,
    "column": 5,
    "length": 5,
    "snippet": "test",
    "label": "error here"
  }
]"#;

    let expected: Vec<Diagnostic> = serde_json::from_str(expected_json).unwrap();

    // Create an actual diagnostic that should match
    let actual = vec![Diagnostic::error_with_code("AT0001", "Test error", Span::new(4, 9))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("test")
        .with_label("error here")];

    let normalized = normalize_diagnostics_for_testing(&actual);

    assert_eq!(normalized, expected);
}
