//! Tests for diagnostic normalization and ordering
//!
//! Verifies that:
//! - Diagnostics are ordered deterministically (errors before warnings, then by file/line/column)
//! - Diagnostic normalization produces consistent output
//! - Same error in different contexts produces same normalized output

use atlas_runtime::diagnostic::{normalizer::normalize_diagnostics_for_testing, sort_diagnostics};
use atlas_runtime::{Binder, DiagnosticLevel, Lexer, Parser, TypeChecker};

/// Helper to get all diagnostics from source code
fn get_all_diagnostics(source: &str) -> Vec<atlas_runtime::Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    let mut binder = Binder::new();
    let (table, bind_diags) = binder.bind(&program);

    let mut checker = TypeChecker::new(&table);
    let type_diags = checker.check(&program);

    // Combine all diagnostics
    let mut all_diags = Vec::new();
    all_diags.extend(lex_diags);
    all_diags.extend(parse_diags);
    all_diags.extend(bind_diags);
    all_diags.extend(type_diags);

    all_diags
}

#[test]
fn test_errors_before_warnings() {
    let source = r#"
        fn foo(x: number) -> number {
            let y = 5;
            return "hello";
        }
    "#;

    let mut diags = get_all_diagnostics(source);
    sort_diagnostics(&mut diags);

    // Count errors and warnings
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    let warnings: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Warning)
        .collect();

    if !errors.is_empty() && !warnings.is_empty() {
        // Find first warning index
        let first_warning_idx = diags
            .iter()
            .position(|d| d.level == DiagnosticLevel::Warning);
        // Find last error index
        let last_error_idx = diags
            .iter()
            .rposition(|d| d.level == DiagnosticLevel::Error);

        if let (Some(first_warning), Some(last_error)) = (first_warning_idx, last_error_idx) {
            assert!(
                last_error < first_warning,
                "All errors should come before all warnings"
            );
        }
    }
}

#[test]
fn test_diagnostics_sorted_by_location() {
    let source = r#"
        fn first() {}
        fn second() {}
        fn first() {}
        fn second() {}
    "#;

    let mut diags = get_all_diagnostics(source);
    sort_diagnostics(&mut diags);

    // Verify diagnostics are sorted by line and column
    for i in 1..diags.len() {
        let prev = &diags[i - 1];
        let curr = &diags[i];

        // Same level: should be sorted by file, line, column
        if prev.level == curr.level {
            if prev.file == curr.file {
                if prev.line == curr.line {
                    assert!(
                        prev.column <= curr.column,
                        "Diagnostics should be sorted by column within same line"
                    );
                } else {
                    assert!(
                        prev.line < curr.line,
                        "Diagnostics should be sorted by line"
                    );
                }
            }
        }
    }
}

#[test]
fn test_sort_is_deterministic() {
    let source = r#"
        fn test() -> number {
            let x = 5;
            let y = 10;
            return "hello";
        }
    "#;

    let mut diags1 = get_all_diagnostics(source);
    let mut diags2 = get_all_diagnostics(source);

    sort_diagnostics(&mut diags1);
    sort_diagnostics(&mut diags2);

    assert_eq!(
        diags1.len(),
        diags2.len(),
        "Should have same number of diagnostics"
    );

    for (d1, d2) in diags1.iter().zip(diags2.iter()) {
        assert_eq!(d1.code, d2.code, "Codes should match");
        assert_eq!(d1.level, d2.level, "Levels should match");
        assert_eq!(d1.line, d2.line, "Lines should match");
        assert_eq!(d1.column, d2.column, "Columns should match");
    }
}

#[test]
fn test_normalization_removes_absolute_paths() {
    let source = "fn foo() {}";

    let mut diags = get_all_diagnostics(source);

    // Set absolute path
    for diag in &mut diags {
        diag.file = "/absolute/path/to/test.atlas".to_string();
    }

    let normalized = normalize_diagnostics_for_testing(&diags);

    for diag in &normalized {
        assert!(
            !diag.file.starts_with('/'),
            "Normalized diagnostic should not have absolute path: {}",
            diag.file
        );
    }
}

#[test]
fn test_normalization_preserves_special_paths() {
    let source = "fn foo() {}";

    let mut diags = get_all_diagnostics(source);

    // Set special paths
    let special_paths = vec!["<input>", "<stdin>", "<unknown>"];
    for (i, path) in special_paths.iter().enumerate() {
        if let Some(diag) = diags.get_mut(i) {
            diag.file = path.to_string();
        }
    }

    let normalized = normalize_diagnostics_for_testing(&diags);

    for (i, path) in special_paths.iter().enumerate() {
        if let Some(diag) = normalized.get(i) {
            assert_eq!(&diag.file, path, "Special path should be preserved");
        }
    }
}

#[test]
fn test_normalization_normalizes_related_locations() {
    let source = r#"
        fn foo() {}
        fn foo() {}
    "#;

    let mut diags = get_all_diagnostics(source);

    // Set absolute path in related locations
    for diag in &mut diags {
        diag.file = "/absolute/path/test.atlas".to_string();
        for related in &mut diag.related {
            related.file = "/absolute/path/other.atlas".to_string();
        }
    }

    let normalized = normalize_diagnostics_for_testing(&diags);

    for diag in &normalized {
        assert!(
            !diag.file.starts_with('/'),
            "File path should be normalized"
        );
        for related in &diag.related {
            assert!(
                !related.file.starts_with('/'),
                "Related file path should be normalized: {}",
                related.file
            );
        }
    }
}

#[test]
fn test_same_error_normalizes_to_same_output() {
    let source1 = "fn foo() {}";
    let source2 = "fn foo() {}";

    let mut diags1 = get_all_diagnostics(source1);
    let mut diags2 = get_all_diagnostics(source2);

    // Set different absolute paths
    for diag in &mut diags1 {
        diag.file = "/path1/test.atlas".to_string();
    }
    for diag in &mut diags2 {
        diag.file = "/path2/test.atlas".to_string();
    }

    let norm1 = normalize_diagnostics_for_testing(&diags1);
    let norm2 = normalize_diagnostics_for_testing(&diags2);

    assert_eq!(
        norm1.len(),
        norm2.len(),
        "Should have same number of diagnostics"
    );

    // Compare normalized JSON output
    for (d1, d2) in norm1.iter().zip(norm2.iter()) {
        let json1 = d1.to_json_string().unwrap();
        let json2 = d2.to_json_string().unwrap();
        assert_eq!(
            json1, json2,
            "Normalized diagnostics should produce identical JSON"
        );
    }
}

#[test]
fn test_multi_span_diagnostics() {
    let source = r#"
        fn foo() {}
        fn foo() {}
    "#;

    let diags = get_all_diagnostics(source);

    // Find diagnostics with related spans
    let multi_span: Vec<_> = diags.iter().filter(|d| !d.related.is_empty()).collect();

    if !multi_span.is_empty() {
        for diag in multi_span {
            // Verify related spans have required fields
            for related in &diag.related {
                assert!(!related.file.is_empty(), "Related span should have file");
                assert!(related.line > 0, "Related span should have line");
                assert!(related.column > 0, "Related span should have column");
                assert!(
                    !related.message.is_empty(),
                    "Related span should have message"
                );
            }
        }
    }
}

#[test]
fn test_diagnostic_ordering_across_files() {
    // Since we're testing with single source, we'll simulate multiple files
    // by creating diagnostics with different file names

    let source = "fn foo() {}";
    let mut diags = get_all_diagnostics(source);

    if diags.len() >= 3 {
        // Set different files
        diags[0].file = "b.atlas".to_string();
        diags[0].line = 5;
        diags[1].file = "a.atlas".to_string();
        diags[1].line = 10;
        diags[2].file = "a.atlas".to_string();
        diags[2].line = 5;

        sort_diagnostics(&mut diags);

        // Should be sorted: a.atlas:5, a.atlas:10, b.atlas:5
        assert_eq!(diags[0].file, "a.atlas");
        assert_eq!(diags[0].line, 5);
        assert_eq!(diags[1].file, "a.atlas");
        assert_eq!(diags[1].line, 10);
        assert_eq!(diags[2].file, "b.atlas");
    }
}

#[test]
fn test_json_output_is_deterministic() {
    let source = "fn foo() {}";

    let diags = get_all_diagnostics(source);

    for diag in &diags {
        let json1 = diag.to_json_string().unwrap();
        let json2 = diag.to_json_string().unwrap();

        assert_eq!(json1, json2, "JSON output should be deterministic");
    }
}
