//! Test utilities and helpers
//!
//! Shared utilities for testing across the codebase.

#![cfg(test)]

use crate::diagnostic::{normalizer::normalize_diagnostic_for_testing, Diagnostic};

/// Normalize diagnostics for golden testing
///
/// This should be used in all golden tests to ensure deterministic output
/// across different machines and environments.
pub fn normalize_for_golden_test(diagnostics: &[Diagnostic]) -> Vec<Diagnostic> {
    diagnostics
        .iter()
        .map(normalize_diagnostic_for_testing)
        .collect()
}

/// Convert diagnostics to JSON for golden test comparison
pub fn diagnostics_to_json_string(diagnostics: &[Diagnostic]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(diagnostics)
}

/// Compare diagnostics JSON output for testing
pub fn assert_diagnostics_match_json(actual: &[Diagnostic], expected_json: &str) {
    let normalized = normalize_for_golden_test(actual);
    let actual_json = diagnostics_to_json_string(&normalized)
        .expect("Failed to serialize diagnostics to JSON");

    // Normalize whitespace for comparison
    let actual_parsed: serde_json::Value =
        serde_json::from_str(&actual_json).expect("Failed to parse actual JSON");
    let expected_parsed: serde_json::Value =
        serde_json::from_str(expected_json).expect("Failed to parse expected JSON");

    assert_eq!(
        actual_parsed, expected_parsed,
        "\nActual JSON:\n{}\n\nExpected JSON:\n{}\n",
        actual_json, expected_json
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    #[test]
    fn test_normalize_for_golden_test() {
        let diags = vec![
            Diagnostic::error("test 1", Span::new(0, 5))
                .with_file("/absolute/path/test.atlas")
                .with_line(1),
            Diagnostic::error("test 2", Span::new(0, 3)).with_file("<input>"),
        ];

        let normalized = normalize_for_golden_test(&diags);

        assert_eq!(normalized.len(), 2);
        // Absolute path should be normalized
        assert_eq!(normalized[0].file, "test.atlas");
        // Special path should remain unchanged
        assert_eq!(normalized[1].file, "<input>");
    }

    #[test]
    fn test_diagnostics_to_json_string() {
        let diags = vec![Diagnostic::error("test", Span::new(0, 1))
            .with_file("test.atlas")
            .with_line(1)];

        let json = diagnostics_to_json_string(&diags).unwrap();

        assert!(json.contains("\"message\": \"test\""));
        assert!(json.contains("\"file\": \"test.atlas\""));
    }
}
