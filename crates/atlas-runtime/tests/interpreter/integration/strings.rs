use super::*;

// ============================================================================
// B10-P05: String method surface — dot-syntax for all str.method() calls
// Both interpreter and VM parity tested throughout.
// ============================================================================

// --- str.len() / str.length ---

#[test]
fn test_string_method_len() {
    let src = r#"let s: string = "hello"; s.len();"#;
    assert_eval_number(src, 5.0);
    assert_parity(src);
}

#[test]
fn test_string_method_length() {
    let src = r#"let s: string = "world"; s.length();"#;
    assert_eval_number(src, 5.0);
    assert_parity(src);
}

#[test]
fn test_string_method_len_empty() {
    let src = r#"let s: string = ""; s.len();"#;
    assert_eval_number(src, 0.0);
    assert_parity(src);
}

// --- str.toUpperCase() / str.toLowerCase() ---

#[test]
fn test_string_method_to_upper_case() {
    let src = r#"let s: string = "hello"; s.toUpperCase();"#;
    assert_eval_string(src, "HELLO");
    assert_parity(src);
}

#[test]
fn test_string_method_to_lower_case() {
    let src = r#"let s: string = "WORLD"; s.toLowerCase();"#;
    assert_eval_string(src, "world");
    assert_parity(src);
}

// --- str.trim() / str.trimStart() / str.trimEnd() ---

#[test]
fn test_string_method_trim() {
    let src = r#"let s: string = "  hello  "; s.trim();"#;
    assert_eval_string(src, "hello");
    assert_parity(src);
}

#[test]
fn test_string_method_trim_start() {
    let src = r#"let s: string = "  hello"; s.trimStart();"#;
    assert_eval_string(src, "hello");
    assert_parity(src);
}

#[test]
fn test_string_method_trim_end() {
    let src = r#"let s: string = "hello  "; s.trimEnd();"#;
    assert_eval_string(src, "hello");
    assert_parity(src);
}

// --- str.includes() ---

#[test]
fn test_string_method_includes_true() {
    let src = r#"let s: string = "hello world"; s.includes("world");"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_string_method_includes_false() {
    let src = r#"let s: string = "hello world"; s.includes("xyz");"#;
    assert_eval_bool(src, false);
    assert_parity(src);
}

// --- str.startsWith() / str.endsWith() ---

#[test]
fn test_string_method_starts_with_true() {
    let src = r#"let s: string = "hello world"; s.startsWith("hello");"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_string_method_starts_with_false() {
    let src = r#"let s: string = "hello world"; s.startsWith("world");"#;
    assert_eval_bool(src, false);
    assert_parity(src);
}

#[test]
fn test_string_method_ends_with_true() {
    let src = r#"let s: string = "hello world"; s.endsWith("world");"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_string_method_ends_with_false() {
    let src = r#"let s: string = "hello world"; s.endsWith("hello");"#;
    assert_eval_bool(src, false);
    assert_parity(src);
}

// --- str.indexOf() / str.lastIndexOf() ---
// These return Option<number> (Atlas model). Use unwrap() to extract.

#[test]
fn test_string_method_index_of_found() {
    let src = r#"let s: string = "hello world"; unwrap(s.indexOf("world"));"#;
    assert_eval_number(src, 6.0);
    assert_parity(src);
}

#[test]
fn test_string_method_index_of_not_found() {
    let src = r#"let s: string = "hello world"; let idx: Option<number> = s.indexOf("xyz"); is_none(idx);"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_string_method_last_index_of() {
    let src = r#"let s: string = "abcabc"; unwrap(s.lastIndexOf("b"));"#;
    assert_eval_number(src, 4.0);
    assert_parity(src);
}

// --- str.charAt() ---
// Returns Option<string>. Use unwrap() to extract.

#[test]
fn test_string_method_char_at() {
    let src = r#"let s: string = "hello"; unwrap(s.charAt(1));"#;
    assert_eval_string(src, "e");
    assert_parity(src);
}

// --- str.substring() ---

#[test]
fn test_string_method_substring() {
    let src = r#"let s: string = "hello world"; s.substring(6, 11);"#;
    assert_eval_string(src, "world");
    assert_parity(src);
}

// --- str.split() ---

#[test]
fn test_string_method_split() {
    let src = r#"let s: string = "a,b,c"; let parts: []string = s.split(","); parts.len();"#;
    assert_eval_number(src, 3.0);
    assert_parity(src);
}

// --- str.repeat() ---

#[test]
fn test_string_method_repeat() {
    let src = r#"let s: string = "ab"; s.repeat(3);"#;
    assert_eval_string(src, "ababab");
    assert_parity(src);
}

// --- str.replace() ---

#[test]
fn test_string_method_replace_first() {
    let src = r#"let s: string = "aaa"; s.replace("a", "b");"#;
    assert_eval_string(src, "baa");
    assert_parity(src);
}

// --- str.replaceAll() ---

#[test]
fn test_string_method_replace_all() {
    let src = r#"let s: string = "aaa"; s.replaceAll("a", "b");"#;
    assert_eval_string(src, "bbb");
    assert_parity(src);
}

#[test]
fn test_string_method_replace_all_word() {
    let src = r#"let s: string = "foo bar foo"; s.replaceAll("foo", "baz");"#;
    assert_eval_string(src, "baz bar baz");
    assert_parity(src);
}

// --- str.padStart() / str.padEnd() ---

#[test]
fn test_string_method_pad_start() {
    let src = r#"let s: string = "5"; s.padStart(3, "0");"#;
    assert_eval_string(src, "005");
    assert_parity(src);
}

#[test]
fn test_string_method_pad_end() {
    let src = r#"let s: string = "hi"; s.padEnd(5, ".");"#;
    assert_eval_string(src, "hi...");
    assert_parity(src);
}

// --- chaining / composition ---

#[test]
fn test_string_method_chain_upper_trim() {
    let src =
        r#"let s: string = "  hello  "; let trimmed: string = s.trim(); trimmed.toUpperCase();"#;
    assert_eval_string(src, "HELLO");
    assert_parity(src);
}

// ============================================================================
// End B10-P05 tests
// ============================================================================

#[test]
fn test_string_concatenation() {
    let code = r#"
        let s: string = "Hello, " + "World!";
        s
    "#;
    assert_eval_string(code, "Hello, World!");
}

#[test]
fn test_string_indexing() {
    let code = r#"
        let s: string = "Hello";
        s[1]
    "#;
    assert_eval_string(code, "e");
}

#[test]
fn test_stdlib_len_string() {
    let code = r#"
        let s: string = "hello";
        len(s)
    "#;
    assert_eval_number(code, 5.0);
}

#[test]
fn test_stdlib_str() {
    let code = r#"
        let n: number = 42;
        str(n)
    "#;
    assert_eval_string(code, "42");
}
