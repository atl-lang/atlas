use super::*;
use pretty_assertions::assert_eq;

// ============================================================================
// Global Scope Shadowing Tests (Disallowed - AT1012)
// ============================================================================

#[rstest]
#[case("global_shadowing_function.atl", "print")]
#[case("global_shadowing_variable.atl", "len")]
fn test_global_shadowing_produces_at1012(#[case] filename: &str, #[case] builtin_name: &str) {
    let diagnostics = check_file(filename);

    // Should have exactly 1 error
    assert_eq!(
        diagnostics.len(),
        1,
        "Expected exactly 1 diagnostic for {}, got: {:?}",
        filename,
        diagnostics
    );

    // Should be AT1012
    assert_eq!(
        diagnostics[0].code, "AT1012",
        "Expected AT1012 for global shadowing, got: {}",
        diagnostics[0].code
    );

    // Should mention the builtin name
    assert!(
        diagnostics[0].message.contains(builtin_name),
        "Error message should mention '{}', got: {}",
        builtin_name,
        diagnostics[0].message
    );

    // Should mention "Cannot shadow prelude builtin"
    assert!(
        diagnostics[0]
            .message
            .contains("Cannot shadow prelude builtin"),
        "Error message should mention prelude shadowing, got: {}",
        diagnostics[0].message
    );

    // Snapshot the diagnostic for stability tracking
    insta::assert_yaml_snapshot!(
        format!("prelude_{}", filename.replace(".atl", "")),
        diagnostics
    );
}

#[test]
fn test_global_shadowing_all_builtins() {
    let diagnostics = check_file("global_shadowing_all.atl");

    // Should have exactly 3 errors (one for each builtin)
    assert_eq!(
        diagnostics.len(),
        3,
        "Expected 3 diagnostics for shadowing all builtins, got: {:?}",
        diagnostics
    );

    // All should be AT1012
    for diag in &diagnostics {
        assert_eq!(
            diag.code, "AT1012",
            "Expected all diagnostics to be AT1012, got: {}",
            diag.code
        );
    }

    // Should mention each builtin
    let messages: Vec<&str> = diagnostics.iter().map(|d| d.message.as_str()).collect();
    assert!(
        messages.iter().any(|m| m.contains("print")),
        "Should have error for 'print'"
    );
    assert!(
        messages.iter().any(|m| m.contains("len")),
        "Should have error for 'len'"
    );
    assert!(
        messages.iter().any(|m| m.contains("str")),
        "Should have error for 'str'"
    );

    // Snapshot all diagnostics
    insta::assert_yaml_snapshot!("prelude_global_shadowing_all", diagnostics);
}

// ============================================================================
// Stability Test
// ============================================================================

#[test]
fn test_prelude_diagnostic_stability() {
    // Verify that running the same file twice produces identical diagnostics
    let diag1 = check_file("global_shadowing_function.atl");
    let diag2 = check_file("global_shadowing_function.atl");

    assert_eq!(
        diag1.len(),
        diag2.len(),
        "Diagnostic count should be stable"
    );
    for (d1, d2) in diag1.iter().zip(diag2.iter()) {
        assert_eq!(d1.code, d2.code, "Diagnostic codes should be stable");
        assert_eq!(
            d1.message, d2.message,
            "Diagnostic messages should be stable"
        );
        assert_eq!(d1.line, d2.line, "Diagnostic lines should be stable");
        assert_eq!(d1.column, d2.column, "Diagnostic columns should be stable");
    }
}

// ============================================================================

// NOTE: test block removed — required access to private function `future_resolve`
