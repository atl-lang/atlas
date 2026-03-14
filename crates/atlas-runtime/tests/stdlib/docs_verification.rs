// Documentation Verification Tests (Polish Phase 03)
use super::*;
use common::{
    assert_eval_bool, assert_eval_number, assert_eval_option_none, assert_eval_option_some_number,
    assert_eval_option_some_string, assert_eval_result_ok_number, assert_eval_string,
};

// --- Core functions ---

#[test]
fn docs_len_string() {
    assert_eval_number(r#"len("hello")"#, 5.0);
}

#[test]
fn docs_len_empty_string() {
    assert_eval_number(r#"len("")"#, 0.0);
}

#[test]
fn docs_len_array() {
    assert_eval_number(r#"len([1, 2, 3])"#, 3.0);
}

#[test]
fn docs_str_number() {
    assert_eval_string(r#"str(42)"#, "42");
}

#[test]
fn docs_str_bool() {
    assert_eval_string(r#"str(true)"#, "true");
}

#[test]
fn docs_typeof_number() {
    assert_eval_string(r#"typeof(42)"#, "number");
}

#[test]
fn docs_typeof_string() {
    assert_eval_string(r#"typeof("hi")"#, "string");
}

#[test]
fn docs_typeof_bool() {
    assert_eval_string(r#"typeof(true)"#, "boolean");
}

#[test]
fn docs_typeof_null() {
    assert_eval_string(r#"typeof(null)"#, "null");
}

#[test]
fn docs_typeof_array() {
    assert_eval_string(r#"typeof([])"#, "array");
}

#[test]
fn docs_to_number_string() {
    assert_eval_result_ok_number(r#"("42").toNumber()"#, 42.0);
}

#[test]
fn docs_to_bool_zero() {
    assert_eval_bool(r#"(0).toBool()"#, false);
}

#[test]
fn docs_to_bool_nonzero() {
    assert_eval_bool(r#"(1).toBool()"#, true);
}

// --- String functions ---

#[test]
fn docs_split_basic() {
    assert_eval_bool(
        r#"
        let parts = split("a,b,c", ",");
        len(parts) == 3 && parts[0] == "a" && parts[2] == "c"
        "#,
        true,
    );
}

#[test]
fn docs_join_basic() {
    assert_eval_string(r#"["x", "y", "z"].join("-")"#, "x-y-z");
}

#[test]
fn docs_trim() {
    assert_eval_string(r#""  hello  ".trim()"#, "hello");
}

#[test]
fn docs_trim_start() {
    assert_eval_string(r#""  hello  ".trimStart()"#, "hello  ");
}

#[test]
fn docs_trim_end() {
    assert_eval_string(r#""  hello  ".trimEnd()"#, "  hello");
}

#[test]
fn docs_to_upper_case() {
    assert_eval_string(r#""hello".toUpperCase()"#, "HELLO");
}

#[test]
fn docs_to_lower_case() {
    assert_eval_string(r#""WORLD".toLowerCase()"#, "world");
}

#[test]
fn docs_starts_with_true() {
    assert_eval_bool(r#""hello world".startsWith("hello")"#, true);
}

#[test]
fn docs_starts_with_false() {
    assert_eval_bool(r#""hello world".startsWith("world")"#, false);
}

#[test]
fn docs_ends_with_true() {
    assert_eval_bool(r#""hello world".endsWith("world")"#, true);
}

#[test]
fn docs_includes_true() {
    assert_eval_bool(r#""hello world".includes("lo wo")"#, true);
}

#[test]
fn docs_includes_false() {
    assert_eval_bool(r#""hello world".includes("xyz")"#, false);
}

#[test]
fn docs_index_of_found() {
    assert_eval_option_some_number(r#""hello".indexOf("ll")"#, 2.0);
}

#[test]
fn docs_index_of_not_found() {
    assert_eval_option_none(r#""hello".indexOf("xyz")"#);
}

#[test]
fn docs_replace() {
    assert_eval_string(r#""hello world".replace("world", "Atlas")"#, "hello Atlas");
}

#[test]
fn docs_repeat() {
    assert_eval_string(r#""ab".repeat(3)"#, "ababab");
}

#[test]
fn docs_repeat_zero() {
    assert_eval_string(r#""x".repeat(0)"#, "");
}

#[test]
fn docs_substring() {
    assert_eval_string(r#""hello world".substring(6, 11)"#, "world");
}

#[test]
fn docs_char_at() {
    assert_eval_option_some_string(r#""hello".charAt(0)"#, "h");
}

#[test]
fn docs_pad_start() {
    assert_eval_string(r#""42".padStart(5, "0")"#, "00042");
}

#[test]
fn docs_pad_end() {
    assert_eval_string(r#""hi".padEnd(5, ".")"#, "hi...");
}

// --- Array functions ---

#[test]
fn docs_reverse() {
    assert_eval_bool(
        r#"
        let rev = [1, 2, 3].reverse();
        rev[0] == 3 && rev[2] == 1
        "#,
        true,
    );
}

#[test]
fn docs_flatten() {
    assert_eval_bool(
        r#"
        let flat = [[1, 2], [3, 4]].flatten();
        len(flat) == 4 && flat[0] == 1 && flat[3] == 4
        "#,
        true,
    );
}

#[test]
fn docs_slice() {
    assert_eval_bool(
        r#"
        let s = [1, 2, 3, 4, 5].slice(1, 4);
        len(s) == 3 && s[0] == 2 && s[2] == 4
        "#,
        true,
    );
}

#[test]
fn docs_array_includes_true() {
    assert_eval_bool(r#"[1, 2, 3].includes(2)"#, true);
}

#[test]
fn docs_array_includes_false() {
    assert_eval_bool(r#"["a", "b"].includes("c")"#, false);
}

#[test]
fn docs_array_index_of() {
    assert_eval_option_some_number(r#"[10, 20, 30].indexOf(20)"#, 1.0);
}

// --- Math functions ---

#[test]
fn docs_abs_negative() {
    assert_eval_number(r#"Math.abs(-5)"#, 5.0);
}

#[test]
fn docs_floor() {
    assert_eval_number(r#"Math.floor(3.7)"#, 3.0);
}

#[test]
fn docs_ceil() {
    assert_eval_number(r#"Math.ceil(3.1)"#, 4.0);
}

#[test]
fn docs_sqrt() {
    assert_eval_result_ok_number(r#"Math.sqrt(9)"#, 3.0);
}

#[test]
fn docs_pow() {
    assert_eval_number(r#"Math.pow(2, 10)"#, 1024.0);
}

#[test]
fn docs_max() {
    assert_eval_number(r#"Math.max(3, 7)"#, 7.0);
}

#[test]
fn docs_min() {
    assert_eval_number(r#"Math.min(3, 7)"#, 3.0);
}

#[test]
fn docs_clamp_in_range() {
    assert_eval_result_ok_number(r#"Math.clamp(5, 0, 10)"#, 5.0);
}

#[test]
fn docs_clamp_below() {
    assert_eval_result_ok_number(r#"Math.clamp(-3, 0, 10)"#, 0.0);
}

#[test]
fn docs_clamp_above() {
    assert_eval_result_ok_number(r#"Math.clamp(15, 0, 10)"#, 10.0);
}

#[test]
fn docs_sign_negative() {
    assert_eval_number(r#"Math.sign(-7)"#, -1.0);
}

#[test]
fn docs_sign_zero() {
    assert_eval_number(r#"Math.sign(0)"#, 0.0);
}

#[test]
fn docs_sign_positive() {
    assert_eval_number(r#"Math.sign(3)"#, 1.0);
}

// --- Type checking ---

#[test]
fn docs_is_string_true() {
    assert_eval_bool(r#"typeof("hello") == "string""#, true);
}

#[test]
fn docs_is_string_false() {
    assert_eval_bool(r#"typeof(42) == "string""#, false);
}

#[test]
fn docs_is_number_true() {
    assert_eval_bool(r#"typeof(3.14) == "number""#, true);
}

#[test]
fn docs_is_bool() {
    assert_eval_bool(r#"typeof(true) == "boolean""#, true);
}

#[test]
fn docs_is_null() {
    assert_eval_bool(r#"typeof(null) == "null""#, true);
}

#[test]
fn docs_is_array() {
    assert_eval_bool(r#"typeof([1, 2]) == "array""#, true);
}

#[test]
fn docs_is_object() {
    // objects in Atlas are records/structs, arrays return "array" not "object"
    assert_eval_bool(r#"typeof([1, 2]) == "object""#, false);
}

// --- JSON functions ---

#[test]
fn docs_parse_json_object() {
    assert_eval_bool(
        r#"
        let json_str = "[1, 2, 3]";
        let arr: json = Json.parse(json_str)?;
        arr[0].asNumber() == 1
        "#,
        true,
    );
}

#[test]
fn docs_to_json() {
    assert_eval_bool(r#"typeof(Json.stringify([1, 2, 3])) == "string""#, true);
}

#[test]
fn docs_is_valid_json_true() {
    assert_eval_bool(r#"Json.isValid("{\"key\": \"value\"}")"#, true);
}

#[test]
fn docs_is_valid_json_false() {
    assert_eval_bool(r#"Json.isValid("not json")"#, false);
}

#[test]
fn docs_json_as_number() {
    assert_eval_number(r#"Json.parse("42")?.asNumber()"#, 42.0);
}

#[test]
fn docs_json_is_null() {
    assert_eval_bool(r#"Json.parse("null")?.isNull()"#, true);
}

// --- Result / Option functions ---

#[test]
fn docs_is_ok() {
    assert_eval_bool(r#"is_ok(Ok(42))"#, true);
}

#[test]
fn docs_is_err() {
    assert_eval_bool(r#"is_err(Err("x"))"#, true);
}

#[test]
fn docs_is_some() {
    assert_eval_bool(r#"is_some(Some(1))"#, true);
}

#[test]
fn docs_is_none() {
    assert_eval_bool(r#"is_none(None())"#, true);
}

#[test]
fn docs_unwrap_ok() {
    assert_eval_number(r#"unwrap(Ok(99))"#, 99.0);
}

#[test]
fn docs_unwrap_some() {
    assert_eval_number(r#"unwrap(Some(42))"#, 42.0);
}

#[test]
fn docs_unwrap_or_present() {
    assert_eval_number(r#"unwrap_or(Some(5), 0)"#, 5.0);
}

#[test]
fn docs_unwrap_or_absent() {
    assert_eval_number(r#"unwrap_or(None(), 0)"#, 0.0);
}

// --- Reflection functions ---

#[test]
fn docs_reflect_typeof() {
    assert_eval_string(r#"reflect_typeof(42)"#, "number");
}

#[test]
fn docs_reflect_is_primitive_true() {
    assert_eval_bool(r#"reflect_is_primitive(42)"#, true);
}

#[test]
fn docs_reflect_is_primitive_false() {
    assert_eval_bool(r#"reflect_is_primitive([])"#, false);
}

#[test]
fn docs_reflect_deep_equals_true() {
    assert_eval_bool(r#"reflect_deep_equals([1, 2], [1, 2])"#, true);
}

#[test]
fn docs_reflect_deep_equals_false() {
    assert_eval_bool(r#"reflect_deep_equals([1], [2])"#, false);
}

#[test]
fn docs_reflect_same_type_true() {
    assert_eval_bool(r#"reflect_same_type(1, 2)"#, true);
}

#[test]
fn docs_reflect_same_type_false() {
    assert_eval_bool(r#"reflect_same_type(1, "1")"#, false);
}

#[test]
fn docs_reflect_is_empty_array() {
    assert_eval_bool(r#"reflect_is_empty([])"#, true);
}

#[test]
fn docs_reflect_is_empty_string() {
    assert_eval_bool(r#"reflect_is_empty("")"#, true);
}

#[test]
fn docs_reflect_is_empty_nonempty() {
    assert_eval_bool(r#"reflect_is_empty([1])"#, false);
}

// --- HashMap documentation examples ---

#[test]
fn docs_hashmap_basic_ops() {
    assert_eval_bool(
        r#"
        let hmap = new Map<string, string>();
        hmap.set("name", "Alice");
        hmap.has("name") && !hmap.has("email")
        "#,
        true,
    );
}

#[test]
fn docs_hashmap_get_returns_some() {
    assert_eval_bool(
        r#"
        let hmap = new Map<string, number>();
        hmap.set("x", 42);
        is_some(hmap.get("x"))
        "#,
        true,
    );
}

#[test]
fn docs_hashmap_get_returns_none() {
    assert_eval_bool(
        r#"
        let hmap = new Map<string, string>();
        is_none(hmap.get("missing"))
        "#,
        true,
    );
}

#[test]
fn docs_hashmap_size() {
    assert_eval_number(
        r#"
        let hmap = new Map<string, number>();
        hmap.set("a", 1);
        hmap.set("b", 2);
        hmap.size()
        "#,
        2.0,
    );
}

// --- HashSet documentation examples ---

#[test]
fn docs_hashset_deduplication() {
    assert_eval_number(
        r#"
        let set = new Set<number>();
        set.add(1); set.add(2); set.add(2); set.add(3); set.add(3); set.add(3);
        set.size()
        "#,
        3.0,
    );
}

#[test]
fn docs_hashset_union() {
    assert_eval_number(
        r#"
        let a = new Set<number>(); a.add(1); a.add(2); a.add(3);
        let b = new Set<number>(); b.add(3); b.add(4); b.add(5);
        a.union(b).size()
        "#,
        5.0,
    );
}

#[test]
fn docs_hashset_intersection() {
    assert_eval_number(
        r#"
        let a = new Set<number>(); a.add(1); a.add(2); a.add(3);
        let b = new Set<number>(); b.add(2); b.add(3); b.add(4);
        a.intersection(b).size()
        "#,
        2.0,
    );
}
