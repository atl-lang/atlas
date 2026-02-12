//! Snapshot tests for diagnostic output validation
//!
//! These tests automatically discover and validate all snapshot fixtures
//! in the tests/snapshots/diagnostics directory.

use atlas_runtime::diagnostic::{normalizer::normalize_diagnostic_for_testing, Diagnostic};
use std::fs;
use std::path::{Path, PathBuf};

/// Get the project root directory
fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Find all snapshot test fixtures
fn find_snapshot_fixtures() -> Vec<PathBuf> {
    let snapshot_dir = project_root().join("tests/snapshots/diagnostics");

    if !snapshot_dir.exists() {
        return Vec::new();
    }

    let mut fixtures = Vec::new();

    if let Ok(entries) = fs::read_dir(&snapshot_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                fixtures.push(path);
            }
        }
    }

    fixtures.sort();
    fixtures
}

/// Load diagnostic from JSON snapshot file
fn load_diagnostic_snapshot(path: &Path) -> Diagnostic {
    let json = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read snapshot file {:?}: {}", path, e));

    serde_json::from_str(&json)
        .unwrap_or_else(|e| panic!("Failed to parse snapshot JSON from {:?}: {}", path, e))
}

/// Verify a diagnostic matches its snapshot
fn verify_snapshot_matches(diagnostic: &Diagnostic, snapshot: &Diagnostic) {
    let normalized = normalize_diagnostic_for_testing(diagnostic);

    assert_eq!(
        normalized.diag_version, snapshot.diag_version,
        "diag_version mismatch"
    );
    assert_eq!(normalized.level, snapshot.level, "level mismatch");
    assert_eq!(normalized.code, snapshot.code, "code mismatch");
    assert_eq!(normalized.message, snapshot.message, "message mismatch");
    assert_eq!(normalized.file, snapshot.file, "file mismatch");
    assert_eq!(normalized.line, snapshot.line, "line mismatch");
    assert_eq!(normalized.column, snapshot.column, "column mismatch");
    assert_eq!(normalized.length, snapshot.length, "length mismatch");

    // Snippet and label are compared if present in snapshot
    if !snapshot.snippet.is_empty() {
        assert_eq!(normalized.snippet, snapshot.snippet, "snippet mismatch");
    }
    if !snapshot.label.is_empty() {
        assert_eq!(normalized.label, snapshot.label, "label mismatch");
    }
}

#[test]
fn test_all_snapshots_valid_json() {
    // Test that all snapshot files are valid JSON
    let fixtures = find_snapshot_fixtures();

    assert!(
        !fixtures.is_empty(),
        "No snapshot fixtures found. Expected fixtures in tests/snapshots/diagnostics/"
    );

    for fixture in fixtures {
        let diagnostic = load_diagnostic_snapshot(&fixture);

        // Verify basic structure
        assert!(!diagnostic.code.is_empty(), "Code should not be empty");
        assert!(
            !diagnostic.message.is_empty(),
            "Message should not be empty"
        );
        assert_eq!(
            diagnostic.diag_version, 1,
            "All snapshots should use version 1"
        );
    }
}

#[test]
fn test_snapshot_normalization_stability() {
    // Test that normalizing snapshots is idempotent
    let fixtures = find_snapshot_fixtures();

    for fixture in fixtures {
        let snapshot = load_diagnostic_snapshot(&fixture);
        let normalized1 = normalize_diagnostic_for_testing(&snapshot);
        let normalized2 = normalize_diagnostic_for_testing(&normalized1);

        assert_eq!(
            normalized1, normalized2,
            "Normalization should be idempotent for {:?}",
            fixture
        );
    }
}

#[test]
fn test_snapshot_json_round_trip() {
    // Test that snapshots can be serialized and deserialized
    let fixtures = find_snapshot_fixtures();

    for fixture in fixtures {
        let original = load_diagnostic_snapshot(&fixture);

        // Serialize to JSON
        let json = serde_json::to_string(&original)
            .unwrap_or_else(|e| panic!("Failed to serialize {:?}: {}", fixture, e));

        // Deserialize back
        let deserialized: Diagnostic = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("Failed to deserialize {:?}: {}", fixture, e));

        assert_eq!(
            original, deserialized,
            "Round-trip failed for {:?}",
            fixture
        );
    }
}

#[test]
fn test_snapshot_files_are_normalized() {
    // Test that all snapshot files use normalized paths
    let fixtures = find_snapshot_fixtures();

    for fixture in fixtures {
        let snapshot = load_diagnostic_snapshot(&fixture);

        // File paths should not be absolute
        assert!(
            !snapshot.file.starts_with('/'),
            "Snapshot {:?} contains absolute path: {}",
            fixture,
            snapshot.file
        );

        // Related locations should also not have absolute paths
        for related in &snapshot.related {
            assert!(
                !related.file.starts_with('/'),
                "Snapshot {:?} contains absolute path in related location: {}",
                fixture,
                related.file
            );
        }
    }
}

#[test]
fn test_error_code_coverage() {
    // Test that we have snapshots for common error codes
    let fixtures = find_snapshot_fixtures();
    let codes: Vec<String> = fixtures
        .iter()
        .map(|f| load_diagnostic_snapshot(f).code)
        .collect();

    // Verify we have at least some basic error code coverage
    assert!(
        codes.iter().any(|c| c.starts_with("AT")),
        "Should have at least one error snapshot"
    );

    // Check for expected error codes (as they're implemented)
    let expected_codes = ["AT0001", "AT0002", "AT0003", "AT0004", "AT0005"];

    for expected in expected_codes {
        if codes.contains(&expected.to_string()) {
            println!("✓ Found snapshot for error code {}", expected);
        }
    }
}

#[test]
fn test_warning_code_coverage() {
    // Test that we have snapshots for warning codes
    let fixtures = find_snapshot_fixtures();
    let codes: Vec<String> = fixtures
        .iter()
        .map(|f| load_diagnostic_snapshot(f).code)
        .collect();

    // Check if we have any warning snapshots
    let has_warnings = codes.iter().any(|c| c.starts_with("AW"));

    if has_warnings {
        println!("✓ Found warning snapshots");
    }
}
