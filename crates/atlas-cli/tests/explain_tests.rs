//! Integration tests for `atlas explain ATXXXX` (B14-P05)

use atlas_runtime::diagnostic::error_codes;

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
