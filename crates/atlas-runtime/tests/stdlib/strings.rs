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
        let result: string[] = "a,b,c".split(",");
        len(result)
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_split_empty_separator() {
    let code = r#"
        let result: string[] = "abc".split("");
        len(result)
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_split_no_match() {
    let code = r#"
        let result: string[] = "hello".split(",");
        len(result)
    "#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_split_unicode() {
    let code = r#"
        let result: string[] = "🎉,🔥,✨".split(",");
        len(result)
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_join_basic() {
    let code = r#"["a", "b", "c"].join(",")"#;
    assert_eval_string(code, "a,b,c");
}

#[test]
fn test_join_empty_array() {
    let code = r#"["a"].slice(1, 1).join(",")"#;
    assert_eval_string(code, "");
}

#[test]
fn test_join_empty_separator() {
    let code = r#"["a", "b", "c"].join("")"#;
    assert_eval_string(code, "abc");
}

#[test]
fn test_trim_basic() {
    let code = r#""  hello  ".trim()"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_trim_unicode_whitespace() {
    let code = "\"\u{00A0}hello\u{00A0}\".trim()";
    assert_eval_string(code, "hello");
}

#[test]
fn test_trim_start() {
    let code = r#""  hello".trimStart()"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_trim_end() {
    let code = r#""hello  ".trimEnd()"#;
    assert_eval_string(code, "hello");
}

// ============================================================================
// Search Operations Tests
// ============================================================================

#[test]
fn test_index_of_found() {
    let code = r#""hello".indexOf("ll")"#;
    assert_eval_option_some_number(code, 2.0);
}

#[test]
fn test_index_of_not_found() {
    let code = r#""hello".indexOf("x")"#;
    assert_eval_option_none(code);
}

#[test]
fn test_index_of_empty_needle() {
    let code = r#""hello".indexOf("")"#;
    assert_eval_option_some_number(code, 0.0);
}

#[test]
fn test_index_of_unicode_offset() {
    let code = r#""éa😊".indexOf("😊")"#;
    assert_eval_option_some_number(code, 2.0);
}

#[test]
fn test_last_index_of_found() {
    let code = r#""hello".lastIndexOf("l")"#;
    assert_eval_option_some_number(code, 3.0);
}

#[test]
fn test_last_index_of_not_found() {
    let code = r#""hello".lastIndexOf("x")"#;
    assert_eval_option_none(code);
}

#[test]
fn test_last_index_of_unicode_offset() {
    let code = r#""éa😊a😊".lastIndexOf("😊")"#;
    assert_eval_option_some_number(code, 4.0);
}

#[test]
fn test_includes_found() {
    let code = r#""hello".includes("ll")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_includes_not_found() {
    let code = r#""hello".includes("x")"#;
    assert_eval_bool(code, false);
}

// ============================================================================
// Transformation Tests
// ============================================================================

#[test]
fn test_to_upper_case() {
    let code = r#""hello".toUpperCase()"#;
    assert_eval_string(code, "HELLO");
}

#[test]
fn test_to_upper_case_unicode() {
    let code = r#""café".toUpperCase()"#;
    assert_eval_string(code, "CAFÉ");
}

#[test]
fn test_to_lower_case() {
    let code = r#""HELLO".toLowerCase()"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_to_lower_case_unicode() {
    let code = r#""CAFÉ".toLowerCase()"#;
    assert_eval_string(code, "café");
}

#[test]
fn test_substring_basic() {
    let code = r#""hello".substring(1, 4)"#;
    assert_eval_string(code, "ell");
}

#[test]
fn test_substring_empty() {
    let code = r#""hello".substring(2, 2)"#;
    assert_eval_string(code, "");
}

#[test]
fn test_substring_out_of_bounds() {
    let code = r#""hello".substring(0, 100)"#;
    assert_has_error(code);
}

#[test]
fn test_char_at_basic() {
    let code = r#""hello".charAt(0)"#;
    assert_eval_option_some_string(code, "h");
}

#[test]
fn test_char_at_unicode() {
    let code = r#""🎉🔥✨".charAt(1)"#;
    assert_eval_option_some_string(code, "🔥");
}

#[test]
fn test_char_at_out_of_bounds() {
    let code = r#""hello".charAt(10)"#;
    assert_eval_option_none(code);
}

#[test]
fn test_repeat_basic() {
    let code = r#""ha".repeat(3)"#;
    assert_eval_string(code, "hahaha");
}

#[test]
fn test_repeat_zero() {
    let code = r#""ha".repeat(0)"#;
    assert_eval_string(code, "");
}

#[test]
fn test_repeat_negative() {
    let code = r#""ha".repeat(-1)"#;
    assert_has_error(code);
}

#[test]
fn test_replace_basic() {
    let code = r#""hello".replace("l", "L")"#;
    assert_eval_string(code, "heLlo");
}

#[test]
fn test_replace_not_found() {
    let code = r#""hello".replace("x", "y")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_replace_empty_search() {
    let code = r#""hello".replace("", "x")"#;
    assert_eval_string(code, "hello");
}

// ============================================================================
// Formatting Tests
// ============================================================================

#[test]
fn test_pad_start_basic() {
    let code = r#""5".padStart(3, "0")"#;
    assert_eval_string(code, "005");
}

#[test]
fn test_pad_start_already_long() {
    let code = r#""hello".padStart(3, "0")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_pad_start_multichar_fill() {
    let code = r#""x".padStart(5, "ab")"#;
    assert_eval_string(code, "ababx");
}

#[test]
fn test_pad_end_basic() {
    let code = r#""5".padEnd(3, "0")"#;
    assert_eval_string(code, "500");
}

#[test]
fn test_pad_end_already_long() {
    let code = r#""hello".padEnd(3, "0")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_starts_with_true() {
    let code = r#""hello".startsWith("he")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_starts_with_false() {
    let code = r#""hello".startsWith("x")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_starts_with_empty() {
    let code = r#""hello".startsWith("")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_ends_with_true() {
    let code = r#""hello".endsWith("lo")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_ends_with_false() {
    let code = r#""hello".endsWith("x")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_ends_with_empty() {
    let code = r#""hello".endsWith("")"#;
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
// str() conversion tests (H-136)
// ============================================================================

#[test]
fn test_h136_str_array() {
    // str() accepts any value per spec: fn str(value: any) -> string
    let code = r#"str([1, 2, 3])"#;
    assert_eval_string(code, "[1, 2, 3]");
}

#[test]
fn test_h136_str_string() {
    let code = r#"str("hello")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_h136_str_chain_result() {
    // arr.filter(fn).map(fn) result must be usable with str()
    let code = r#"
        fn gt2(borrow x: number): bool { return x > 2; }
        fn dbl(borrow x: number): number { return x * 2; }
        let arr = [1, 2, 3, 4, 5];
        str(arr.filter(gt2).map(dbl))
    "#;
    assert_eval_string(code, "[6, 8, 10]");
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
