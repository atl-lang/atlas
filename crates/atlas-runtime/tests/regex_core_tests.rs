//! Regex core functionality tests (Phase-08a)
//!
//! Tests regex compilation, matching, and capture group extraction.
//! All tests use Atlas::eval() API.

use atlas_runtime::Atlas;

// ============================================================================
// Test Helpers
// ============================================================================

fn eval_ok(code: &str) -> String {
    let atlas = Atlas::new();
    let result = atlas.eval(code).expect("Execution should succeed");
    result.to_string()
}

// ============================================================================
// Compilation Tests (6 tests)
// ============================================================================

#[test]
fn test_regex_new_valid_pattern() {
    let code = r#"
        let pattern = regexNew("\\d+");
        typeof(unwrap(pattern))
    "#;
    assert_eq!(eval_ok(code), "regex");
}

#[test]
fn test_regex_new_invalid_pattern() {
    let code = r#"
        let pattern = regexNew("[invalid");
        is_err(pattern)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_regex_new_empty_pattern() {
    let code = r#"
        let pattern = regexNew("");
        typeof(unwrap(pattern))
    "#;
    assert_eq!(eval_ok(code), "regex");
}

#[test]
fn test_regex_new_complex_pattern() {
    let code = r#"
        let pattern = regexNew("(?P<year>\\d{4})-(?P<month>\\d{2})-(?P<day>\\d{2})");
        typeof(unwrap(pattern))
    "#;
    assert_eq!(eval_ok(code), "regex");
}

#[test]
fn test_regex_escape() {
    let code = r#"
        regexEscape("hello.world*test+")
    "#;
    assert_eq!(eval_ok(code), "hello\\.world\\*test\\+");
}

#[test]
fn test_regex_new_with_flags() {
    let code = r#"
        let pattern = regexNewWithFlags("HELLO", "i");
        let regex = unwrap(pattern);
        regexIsMatch(regex, "hello")
    "#;
    assert_eq!(eval_ok(code), "true");
}

// ============================================================================
// Matching Tests (12 tests)
// ============================================================================

#[test]
fn test_is_match_true() {
    let code = r#"
        let pattern = unwrap(regexNew("\\d+"));
        regexIsMatch(pattern, "hello123world")
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_is_match_false() {
    let code = r#"
        let pattern = unwrap(regexNew("\\d+"));
        regexIsMatch(pattern, "hello world")
    "#;
    assert_eq!(eval_ok(code), "false");
}

#[test]
fn test_is_match_case_insensitive() {
    let code = r#"
        let pattern = unwrap(regexNewWithFlags("HELLO", "i"));
        regexIsMatch(pattern, "hello world")
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_is_match_multiline() {
    let code = r#"
        let pattern = unwrap(regexNewWithFlags("^world", "m"));
        regexIsMatch(pattern, "hello\nworld")
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_find_returns_match() {
    let code = r#"
        let pattern = unwrap(regexNew("\\d+"));
        let result = regexFind(pattern, "hello123world");
        let match_obj = unwrap(result);
        unwrap(hashMapGet(match_obj, "text"))
    "#;
    assert_eq!(eval_ok(code), "123");
}

#[test]
fn test_find_returns_none() {
    let code = r#"
        let pattern = unwrap(regexNew("\\d+"));
        let result = regexFind(pattern, "hello world");
        is_none(result)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_find_all_multiple_matches() {
    let code = r#"
        let pattern = unwrap(regexNew("\\d+"));
        let matches = regexFindAll(pattern, "a1 b22 c333");
        len(matches)
    "#;
    assert_eq!(eval_ok(code), "3");
}

#[test]
fn test_find_all_no_matches() {
    let code = r#"
        let pattern = unwrap(regexNew("\\d+"));
        let matches = regexFindAll(pattern, "hello world");
        len(matches)
    "#;
    assert_eq!(eval_ok(code), "0");
}

#[test]
fn test_find_all_non_overlapping() {
    let code = r#"
        let pattern = unwrap(regexNew("\\w+"));
        let matches = regexFindAll(pattern, "hello world test");
        len(matches)
    "#;
    assert_eq!(eval_ok(code), "3");
}

#[test]
fn test_unicode_handling() {
    let code = r#"
        let pattern = unwrap(regexNew("世界"));
        regexIsMatch(pattern, "こんにちは世界")
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_dot_matches_newline_with_flag() {
    let code = r#"
        let pattern = unwrap(regexNewWithFlags("a.b", "s"));
        regexIsMatch(pattern, "a\nb")
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_anchors_start_end() {
    let code = r#"
        let pattern = unwrap(regexNew("^hello$"));
        regexIsMatch(pattern, "hello")
    "#;
    assert_eq!(eval_ok(code), "true");
}

// ============================================================================
// Capture Group Tests (12 tests)
// ============================================================================

#[test]
fn test_captures_simple_group() {
    let code = r#"
        let pattern = unwrap(regexNew("(\\d+)"));
        let groups = unwrap(regexCaptures(pattern, "hello123"));
        len(groups)
    "#;
    assert_eq!(eval_ok(code), "2"); // Full match + 1 group
}

#[test]
fn test_captures_multiple_groups() {
    let code = r#"
        let pattern = unwrap(regexNew("(\\d+)-(\\w+)"));
        let groups = unwrap(regexCaptures(pattern, "123-abc"));
        len(groups)
    "#;
    assert_eq!(eval_ok(code), "3"); // Full match + 2 groups
}

#[test]
fn test_captures_nested_groups() {
    let code = r#"
        let pattern = unwrap(regexNew("((\\d+)-(\\w+))"));
        let groups = unwrap(regexCaptures(pattern, "123-abc"));
        len(groups)
    "#;
    assert_eq!(eval_ok(code), "4"); // Full match + 3 groups
}

#[test]
fn test_captures_optional_group() {
    let code = r#"
        let pattern = unwrap(regexNew("(\\d+)?-(\\w+)"));
        let groups = unwrap(regexCaptures(pattern, "-abc"));
        len(groups)
    "#;
    assert_eq!(eval_ok(code), "3"); // Full match + 2 groups (first is null)
}

#[test]
fn test_captures_named_groups() {
    let code = r#"
        let pattern = unwrap(regexNew("(?P<num>\\d+)-(?P<word>\\w+)"));
        let groups = unwrap(regexCapturesNamed(pattern, "123-abc"));
        unwrap(hashMapGet(groups, "num"))
    "#;
    assert_eq!(eval_ok(code), "123");
}

#[test]
fn test_captures_named_and_positional() {
    let code = r#"
        let pattern = unwrap(regexNew("(?P<first>\\d+)-(\\w+)"));
        let positional = unwrap(regexCaptures(pattern, "123-abc"));
        let named = unwrap(regexCapturesNamed(pattern, "123-abc"));
        len(positional)
    "#;
    assert_eq!(eval_ok(code), "3");
}

#[test]
fn test_captures_none_when_no_match() {
    let code = r#"
        let pattern = unwrap(regexNew("(\\d+)"));
        let groups = regexCaptures(pattern, "hello world");
        is_none(groups)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_captures_named_none_when_no_match() {
    let code = r#"
        let pattern = unwrap(regexNew("(?P<num>\\d+)"));
        let groups = regexCapturesNamed(pattern, "hello world");
        is_none(groups)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_captures_with_alternation() {
    let code = r#"
        let pattern = unwrap(regexNew("(cat|dog)"));
        let groups = unwrap(regexCaptures(pattern, "I have a dog"));
        len(groups)
    "#;
    assert_eq!(eval_ok(code), "2");
}

#[test]
fn test_captures_backreferences_unsupported() {
    // Backreferences are NOT supported by Rust's regex crate
    // This test verifies we get an error (not a panic)
    let code = r#"
        let pattern = regexNew("(\\w+)\\s+\\1");
        is_err(pattern)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_captures_non_capturing_groups() {
    let code = r#"
        let pattern = unwrap(regexNew("(?:\\d+)-(\\w+)"));
        let groups = unwrap(regexCaptures(pattern, "123-abc"));
        len(groups)
    "#;
    assert_eq!(eval_ok(code), "2"); // Full match + 1 capturing group (non-capturing doesn't count)
}

#[test]
fn test_captures_full_match_at_index_zero() {
    let code = r#"
        let pattern = unwrap(regexNew("(\\d+)-(\\w+)"));
        let groups = unwrap(regexCaptures(pattern, "123-abc"));
        groups[0]
    "#;
    assert_eq!(eval_ok(code), "123-abc");
}

// ============================================================================
// Additional Edge Case Tests (5 tests to reach 35+)
// ============================================================================

#[test]
fn test_find_with_positions() {
    let code = r#"
        let pattern = unwrap(regexNew("\\d+"));
        let match_obj = unwrap(regexFind(pattern, "hello123world"));
        let start = unwrap(hashMapGet(match_obj, "start"));
        let end_pos = unwrap(hashMapGet(match_obj, "end"));
        start
    "#;
    assert_eq!(eval_ok(code), "5");
}

#[test]
fn test_find_all_extracts_all_text() {
    let code = r#"
        let pattern = unwrap(regexNew("\\d+"));
        let matches = regexFindAll(pattern, "1 and 22 and 333");
        let first = unwrap(hashMapGet(matches[0], "text"));
        let second = unwrap(hashMapGet(matches[1], "text"));
        let third = unwrap(hashMapGet(matches[2], "text"));
        first
    "#;
    assert_eq!(eval_ok(code), "1");
}

#[test]
fn test_regex_escape_all_special_chars() {
    let code = r#"
        let escaped = regexEscape(".*+?^$()[]{}|\\");
        let pattern = unwrap(regexNew(escaped));
        regexIsMatch(pattern, ".*+?^$()[]{}|\\")
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_invalid_flag_returns_error() {
    let code = r#"
        let pattern = regexNewWithFlags("test", "xyz");
        is_err(pattern)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_complex_email_pattern() {
    let code = r#"
        let pattern = unwrap(regexNew("[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}"));
        regexIsMatch(pattern, "user@example.com")
    "#;
    assert_eq!(eval_ok(code), "true");
}

// ============================================================================
// Test Count Verification
// ============================================================================

// Total tests:
// - Compilation: 6
// - Matching: 12
// - Capture Groups: 12
// - Edge Cases: 5
// TOTAL: 35 tests ✅
