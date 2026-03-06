use super::*;

// From stdlib_string_tests.rs
// ============================================================================

// String stdlib tests (Interpreter engine)
//
// Tests all 18 string functions with comprehensive edge case coverage

// ============================================================================
// Core Operations Tests
// ============================================================================

#[test]
fn test_split_basic() {
    let code = r#"
        let result: string[] = split("a,b,c", ",");
        len(result)
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_split_empty_separator() {
    let code = r#"
        let result: string[] = split("abc", "");
        len(result)
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_split_no_match() {
    let code = r#"
        let result: string[] = split("hello", ",");
        len(result)
    "#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_split_unicode() {
    let code = r#"
        let result: string[] = split("🎉,🔥,✨", ",");
        len(result)
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_join_basic() {
    let code = r#"join(["a", "b", "c"], ",")"#;
    assert_eval_string(code, "a,b,c");
}

#[test]
fn test_join_empty_array() {
    let code = r#"join(slice(["a"], 1, 1), ",")"#;
    assert_eval_string(code, "");
}

#[test]
fn test_join_empty_separator() {
    let code = r#"join(["a", "b", "c"], "")"#;
    assert_eval_string(code, "abc");
}

#[test]
fn test_trim_basic() {
    let code = r#"trim("  hello  ")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_trim_unicode_whitespace() {
    let code = "trim(\"\u{00A0}hello\u{00A0}\")";
    assert_eval_string(code, "hello");
}

#[test]
fn test_trim_start() {
    let code = r#"trim_start("  hello")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_trim_end() {
    let code = r#"trim_end("hello  ")"#;
    assert_eval_string(code, "hello");
}

// ============================================================================
// Search Operations Tests
// ============================================================================

#[test]
fn test_index_of_found() {
    let code = r#"index_of("hello", "ll")"#;
    assert_eval_option_some_number(code, 2.0);
}

#[test]
fn test_index_of_not_found() {
    let code = r#"index_of("hello", "x")"#;
    assert_eval_option_none(code);
}

#[test]
fn test_index_of_empty_needle() {
    let code = r#"index_of("hello", "")"#;
    assert_eval_option_some_number(code, 0.0);
}

#[test]
fn test_index_of_unicode_offset() {
    let code = r#"index_of("éa😊", "😊")"#;
    assert_eval_option_some_number(code, 2.0);
}

#[test]
fn test_last_index_of_found() {
    let code = r#"last_index_of("hello", "l")"#;
    assert_eval_option_some_number(code, 3.0);
}

#[test]
fn test_last_index_of_not_found() {
    let code = r#"last_index_of("hello", "x")"#;
    assert_eval_option_none(code);
}

#[test]
fn test_last_index_of_unicode_offset() {
    let code = r#"last_index_of("éa😊a😊", "😊")"#;
    assert_eval_option_some_number(code, 4.0);
}

#[test]
fn test_includes_found() {
    let code = r#"includes("hello", "ll")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_includes_not_found() {
    let code = r#"includes("hello", "x")"#;
    assert_eval_bool(code, false);
}

// ============================================================================
// Transformation Tests
// ============================================================================

#[test]
fn test_to_upper_case() {
    let code = r#"to_upper_case("hello")"#;
    assert_eval_string(code, "HELLO");
}

#[test]
fn test_to_upper_case_unicode() {
    let code = r#"to_upper_case("café")"#;
    assert_eval_string(code, "CAFÉ");
}

#[test]
fn test_to_lower_case() {
    let code = r#"to_lower_case("HELLO")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_to_lower_case_unicode() {
    let code = r#"to_lower_case("CAFÉ")"#;
    assert_eval_string(code, "café");
}

#[test]
fn test_substring_basic() {
    let code = r#"substring("hello", 1, 4)"#;
    assert_eval_string(code, "ell");
}

#[test]
fn test_substring_empty() {
    let code = r#"substring("hello", 2, 2)"#;
    assert_eval_string(code, "");
}

#[test]
fn test_substring_out_of_bounds() {
    let code = r#"substring("hello", 0, 100)"#;
    assert_has_error(code);
}

#[test]
fn test_char_at_basic() {
    let code = r#"char_at("hello", 0)"#;
    assert_eval_option_some_string(code, "h");
}

#[test]
fn test_char_at_unicode() {
    let code = r#"char_at("🎉🔥✨", 1)"#;
    assert_eval_option_some_string(code, "🔥");
}

#[test]
fn test_char_at_out_of_bounds() {
    let code = r#"char_at("hello", 10)"#;
    assert_eval_option_none(code);
}

#[test]
fn test_repeat_basic() {
    let code = r#"repeat("ha", 3)"#;
    assert_eval_string(code, "hahaha");
}

#[test]
fn test_repeat_zero() {
    let code = r#"repeat("ha", 0)"#;
    assert_eval_string(code, "");
}

#[test]
fn test_repeat_negative() {
    let code = r#"repeat("ha", -1)"#;
    assert_has_error(code);
}

#[test]
fn test_replace_basic() {
    let code = r#"replace("hello", "l", "L")"#;
    assert_eval_string(code, "heLlo");
}

#[test]
fn test_replace_not_found() {
    let code = r#"replace("hello", "x", "y")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_replace_empty_search() {
    let code = r#"replace("hello", "", "x")"#;
    assert_eval_string(code, "hello");
}

// ============================================================================
// Formatting Tests
// ============================================================================

#[test]
fn test_pad_start_basic() {
    let code = r#"pad_start("5", 3, "0")"#;
    assert_eval_string(code, "005");
}

#[test]
fn test_pad_start_already_long() {
    let code = r#"pad_start("hello", 3, "0")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_pad_start_multichar_fill() {
    let code = r#"pad_start("x", 5, "ab")"#;
    assert_eval_string(code, "ababx");
}

#[test]
fn test_pad_end_basic() {
    let code = r#"pad_end("5", 3, "0")"#;
    assert_eval_string(code, "500");
}

#[test]
fn test_pad_end_already_long() {
    let code = r#"pad_end("hello", 3, "0")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_starts_with_true() {
    let code = r#"starts_with("hello", "he")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_starts_with_false() {
    let code = r#"starts_with("hello", "x")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_starts_with_empty() {
    let code = r#"starts_with("hello", "")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_ends_with_true() {
    let code = r#"ends_with("hello", "lo")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_ends_with_false() {
    let code = r#"ends_with("hello", "x")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_ends_with_empty() {
    let code = r#"ends_with("hello", "")"#;
    assert_eval_bool(code, true);
}

// ============================================================================
// String Indexing Tests (H-022)
// ============================================================================

#[test]
fn test_string_index_basic() {
    let code = r#""hello"[0]"#;
    assert_eval_string(code, "h");
}

#[test]
fn test_string_index_middle() {
    let code = r#""hello"[2]"#;
    assert_eval_string(code, "l");
}

#[test]
fn test_string_index_last() {
    let code = r#""hello"[4]"#;
    assert_eval_string(code, "o");
}

#[test]
fn test_string_index_unicode() {
    let code = r#""🎉🔥✨"[1]"#;
    assert_eval_string(code, "🔥");
}

#[test]
fn test_string_index_out_of_bounds() {
    let code = r#""hello"[10]"#;
    assert_has_error(code);
}

#[test]
fn test_string_index_negative() {
    let code = r#""hello"[-1]"#;
    assert_has_error(code);
}

#[test]
fn test_string_index_non_integer() {
    let code = r#""hello"[1.5]"#;
    assert_has_error(code);
}

// ============================================================================
// String Length Method Tests (H-022)
// ============================================================================

#[test]
fn test_string_length_basic() {
    let code = r#""hello".length()"#;
    assert_eval_number(code, 5.0);
}

#[test]
fn test_string_length_empty() {
    let code = r#""".length()"#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_string_length_unicode() {
    // Unicode scalars, not bytes
    let code = r#""🎉🔥✨".length()"#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_string_length_via_variable() {
    let code = r#"
        let s = "hello world";
        s.length()
    "#;
    assert_eval_number(code, 11.0);
}

// ============================================================================
// String Method Chaining Tests
// ============================================================================

#[test]
fn test_string_trim_length() {
    let code = r#""  hello  ".trim().length()"#;
    assert_eval_number(code, 5.0);
}

#[test]
fn test_string_to_upper_includes() {
    let code = r#""hello".toUpperCase().includes("ELL")"#;
    assert_eval_bool(code, true);
}

// NOTE: test block removed — required access to private function `repeat`
