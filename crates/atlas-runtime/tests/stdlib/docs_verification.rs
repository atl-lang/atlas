use super::*;

// Documentation Verification Tests (Polish Phase 03)
use super::*;
use common::{assert_eval_bool, assert_eval_number, assert_eval_string};

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
    assert_eval_string(r#"typeof(true)"#, "bool");
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
    assert_eval_number(r#"toNumber("42")"#, 42.0);
}

#[test]
fn docs_to_bool_zero() {
    assert_eval_bool(r#"toBool(0)"#, false);
}

#[test]
fn docs_to_bool_nonzero() {
    assert_eval_bool(r#"toBool(1)"#, true);
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
    assert_eval_string(r#"join(["x", "y", "z"], "-")"#, "x-y-z");
}

#[test]
fn docs_trim() {
    assert_eval_string(r#"trim("  hello  ")"#, "hello");
}

#[test]
fn docs_trim_start() {
    assert_eval_string(r#"trimStart("  hello  ")"#, "hello  ");
}

#[test]
fn docs_trim_end() {
    assert_eval_string(r#"trimEnd("  hello  ")"#, "  hello");
}

#[test]
fn docs_to_upper_case() {
    assert_eval_string(r#"toUpperCase("hello")"#, "HELLO");
}

#[test]
fn docs_to_lower_case() {
    assert_eval_string(r#"toLowerCase("WORLD")"#, "world");
}

#[test]
fn docs_starts_with_true() {
    assert_eval_bool(r#"startsWith("hello world", "hello")"#, true);
}

#[test]
fn docs_starts_with_false() {
    assert_eval_bool(r#"startsWith("hello world", "world")"#, false);
}

#[test]
fn docs_ends_with_true() {
    assert_eval_bool(r#"endsWith("hello world", "world")"#, true);
}

#[test]
fn docs_includes_true() {
    assert_eval_bool(r#"includes("hello world", "lo wo")"#, true);
}

#[test]
fn docs_includes_false() {
    assert_eval_bool(r#"includes("hello world", "xyz")"#, false);
}

#[test]
fn docs_index_of_found() {
    assert_eval_number(r#"indexOf("hello", "ll")"#, 2.0);
}

#[test]
fn docs_index_of_not_found() {
    assert_eval_number(r#"indexOf("hello", "xyz")"#, -1.0);
}

#[test]
fn docs_replace() {
    assert_eval_string(r#"replace("hello world", "world", "Atlas")"#, "hello Atlas");
}

#[test]
fn docs_repeat() {
    assert_eval_string(r#"repeat("ab", 3)"#, "ababab");
}

#[test]
fn docs_repeat_zero() {
    assert_eval_string(r#"repeat("x", 0)"#, "");
}

#[test]
fn docs_substring() {
    assert_eval_string(r#"substring("hello world", 6, 11)"#, "world");
}

#[test]
fn docs_char_at() {
    assert_eval_string(r#"charAt("hello", 0)"#, "h");
}

#[test]
fn docs_pad_start() {
    assert_eval_string(r#"padStart("42", 5, "0")"#, "00042");
}

#[test]
fn docs_pad_end() {
    assert_eval_string(r#"padEnd("hi", 5, ".")"#, "hi...");
}

// --- Array functions ---

#[test]
fn docs_reverse() {
    assert_eval_bool(
        r#"
        let rev = reverse([1, 2, 3]);
        rev[0] == 3 && rev[2] == 1
        "#,
        true,
    );
}

#[test]
fn docs_flatten() {
    assert_eval_bool(
        r#"
        let flat = flatten([[1, 2], [3, 4]]);
        len(flat) == 4 && flat[0] == 1 && flat[3] == 4
        "#,
        true,
    );
}

#[test]
fn docs_slice() {
    assert_eval_bool(
        r#"
        let s = slice([1, 2, 3, 4, 5], 1, 4);
        len(s) == 3 && s[0] == 2 && s[2] == 4
        "#,
        true,
    );
}

#[test]
fn docs_array_includes_true() {
    assert_eval_bool(r#"arrayIncludes([1, 2, 3], 2)"#, true);
}

#[test]
fn docs_array_includes_false() {
    assert_eval_bool(r#"arrayIncludes(["a", "b"], "c")"#, false);
}

#[test]
fn docs_array_index_of() {
    assert_eval_number(r#"arrayIndexOf([10, 20, 30], 20)"#, 1.0);
}

// --- Math functions ---

#[test]
fn docs_abs_negative() {
    assert_eval_number(r#"abs(-5)"#, 5.0);
}

#[test]
fn docs_floor() {
    assert_eval_number(r#"floor(3.7)"#, 3.0);
}

#[test]
fn docs_ceil() {
    assert_eval_number(r#"ceil(3.1)"#, 4.0);
}

#[test]
fn docs_sqrt() {
    assert_eval_number(r#"sqrt(9)"#, 3.0);
}

#[test]
fn docs_pow() {
    assert_eval_number(r#"pow(2, 10)"#, 1024.0);
}

#[test]
fn docs_max() {
    assert_eval_number(r#"max(3, 7)"#, 7.0);
}

#[test]
fn docs_min() {
    assert_eval_number(r#"min(3, 7)"#, 3.0);
}

#[test]
fn docs_clamp_in_range() {
    assert_eval_number(r#"clamp(5, 0, 10)"#, 5.0);
}

#[test]
fn docs_clamp_below() {
    assert_eval_number(r#"clamp(-3, 0, 10)"#, 0.0);
}

#[test]
fn docs_clamp_above() {
    assert_eval_number(r#"clamp(15, 0, 10)"#, 10.0);
}

#[test]
fn docs_sign_negative() {
    assert_eval_number(r#"sign(-7)"#, -1.0);
}

#[test]
fn docs_sign_zero() {
    assert_eval_number(r#"sign(0)"#, 0.0);
}

#[test]
fn docs_sign_positive() {
    assert_eval_number(r#"sign(3)"#, 1.0);
}

// --- Type checking ---

#[test]
fn docs_is_string_true() {
    assert_eval_bool(r#"isString("hello")"#, true);
}

#[test]
fn docs_is_string_false() {
    assert_eval_bool(r#"isString(42)"#, false);
}

#[test]
fn docs_is_number_true() {
    assert_eval_bool(r#"isNumber(3.14)"#, true);
}

#[test]
fn docs_is_bool() {
    assert_eval_bool(r#"isBool(true)"#, true);
}

#[test]
fn docs_is_null() {
    assert_eval_bool(r#"isNull(null)"#, true);
}

#[test]
fn docs_is_array() {
    assert_eval_bool(r#"isArray([1, 2])"#, true);
}

#[test]
fn docs_is_object() {
    // isObject returns false for non-objects
    assert_eval_bool(r#"isObject([1, 2])"#, false);
}

// --- JSON functions ---

#[test]
fn docs_parse_json_object() {
    assert_eval_bool(
        r#"
        let json_str = "[1, 2, 3]";
        let arr: json = parseJSON(json_str);
        jsonAsNumber(arr[0]) == 1
        "#,
        true,
    );
}

#[test]
fn docs_to_json() {
    assert_eval_bool(r#"isString(toJSON([1, 2, 3]))"#, true);
}

#[test]
fn docs_is_valid_json_true() {
    assert_eval_bool(r#"isValidJSON("{\"key\": \"value\"}")"#, true);
}

#[test]
fn docs_is_valid_json_false() {
    assert_eval_bool(r#"isValidJSON("not json")"#, false);
}

#[test]
fn docs_json_as_number() {
    assert_eval_number(r#"jsonAsNumber(parseJSON("42"))"#, 42.0);
}

#[test]
fn docs_json_is_null() {
    assert_eval_bool(r#"jsonIsNull(parseJSON("null"))"#, true);
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
        let hmap = hashMapNew();
        hashMapPut(hmap, "name", "Alice");
        hashMapHas(hmap, "name") && !hashMapHas(hmap, "email")
        "#,
        true,
    );
}

#[test]
fn docs_hashmap_get_returns_some() {
    assert_eval_bool(
        r#"
        let hmap = hashMapNew();
        hashMapPut(hmap, "x", 42);
        is_some(hashMapGet(hmap, "x"))
        "#,
        true,
    );
}

#[test]
fn docs_hashmap_get_returns_none() {
    assert_eval_bool(
        r#"
        let hmap = hashMapNew();
        is_none(hashMapGet(hmap, "missing"))
        "#,
        true,
    );
}

#[test]
fn docs_hashmap_size() {
    assert_eval_number(
        r#"
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapSize(hmap)
        "#,
        2.0,
    );
}

// --- HashSet documentation examples ---

#[test]
fn docs_hashset_deduplication() {
    assert_eval_number(
        r#"
        let set = hashSetFromArray([1, 2, 2, 3, 3, 3]);
        hashSetSize(set)
        "#,
        3.0,
    );
}

#[test]
fn docs_hashset_union() {
    assert_eval_number(
        r#"
        let a = hashSetFromArray([1, 2, 3]);
        let b = hashSetFromArray([3, 4, 5]);
        hashSetSize(hashSetUnion(a, b))
        "#,
        5.0,
    );
}

#[test]
fn docs_hashset_intersection() {
    assert_eval_number(
        r#"
        let a = hashSetFromArray([1, 2, 3]);
        let b = hashSetFromArray([2, 3, 4]);
        hashSetSize(hashSetIntersection(a, b))
        "#,
        2.0,
    );
}
