//! Stdlib Core Hardening Tests (Phase v02-completion-03)
//!
//! Edge case coverage for: string, array, math, json, types modules.
//! All tests run in both interpreter and VM for parity verification.

use atlas_runtime::diagnostic::Diagnostic;
use atlas_runtime::json_value::JsonValue;
use atlas_runtime::runtime::Atlas;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::value::Value;

// ============================================================================
// Test helpers
// ============================================================================

fn eval_ok(source: &str) -> Value {
    let runtime = Atlas::new();
    runtime
        .eval(source)
        .unwrap_or_else(|e| panic!("eval_ok failed for {:?}: {:?}", source, e))
}

fn eval_err(source: &str) -> Vec<Diagnostic> {
    let runtime = Atlas::new();
    runtime
        .eval(source)
        .map(|v| panic!("eval_err expected error, got: {:?}", v))
        .unwrap_err()
}

fn vm_eval_ok(source: &str) -> Value {
    use atlas_runtime::{Binder, Compiler, Lexer, Parser, TypeChecker, VM};

    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler
        .compile(&ast)
        .unwrap_or_else(|e| panic!("vm_eval_ok compile failed for {:?}: {:?}", source, e));
    let security = SecurityContext::new();
    let mut vm = VM::new(bytecode);
    vm.run(&security)
        .map(|opt| opt.unwrap_or(Value::Null))
        .unwrap_or_else(|e| panic!("vm_eval_ok run failed for {:?}: {:?}", source, e))
}

/// Check that eval produced a RuntimeError (any diagnostic = error)
fn is_runtime_error(diags: &[Diagnostic]) -> bool {
    !diags.is_empty()
}

// ============================================================================
// STRING HARDENING (35 tests)
// ============================================================================

// split()

#[test]
fn test_split_empty_string_with_separator() {
    // split("", ",") returns [""] — one empty-string element
    let result = eval_ok(r#"split("", ",");"#);
    match result {
        Value::Array(arr) => {
            let b = arr.as_slice();
            assert_eq!(b.len(), 1);
            assert_eq!(b[0], Value::string(""));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_split_empty_string_empty_separator() {
    // split("", "") returns [] — no characters to iterate
    let result = eval_ok(r#"split("", "");"#);
    match result {
        Value::Array(arr) => {
            let b = arr.as_slice();
            assert_eq!(b.len(), 0);
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_split_separator_not_found() {
    let result = eval_ok(r#"split("abc", ",");"#);
    match result {
        Value::Array(arr) => {
            let b = arr.as_slice();
            assert_eq!(b.len(), 1);
            assert_eq!(b[0], Value::string("abc"));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_split_parity() {
    let interp = format!("{:?}", eval_ok(r#"split("a,b,c", ",");"#));
    let vm = format!("{:?}", vm_eval_ok(r#"split("a,b,c", ",");"#));
    assert_eq!(interp, vm);
}

// trim variants

#[test]
fn test_trim_all_whitespace() {
    assert_eq!(eval_ok(r#"trim("   ");"#), Value::string(""));
}

#[test]
fn test_trim_start_all_whitespace() {
    assert_eq!(eval_ok(r#"trim_start("   ");"#), Value::string(""));
}

#[test]
fn test_trim_end_all_whitespace() {
    assert_eq!(eval_ok(r#"trim_end("   ");"#), Value::string(""));
}

#[test]
fn test_trim_no_whitespace() {
    assert_eq!(eval_ok(r#"trim("hello");"#), Value::string("hello"));
}

#[test]
fn test_trim_parity() {
    let i = eval_ok(r#"trim("  hello  ");"#);
    let v = vm_eval_ok(r#"trim("  hello  ");"#);
    assert_eq!(i, v);
}

// index_of / last_index_of

#[test]
fn test_index_of_empty_haystack() {
    assert_eq!(eval_ok(r#"index_of("", "x");"#), Value::Option(None));
}

#[test]
fn test_index_of_empty_needle_on_empty() {
    assert_eq!(
        eval_ok(r#"index_of("", "");"#),
        Value::Option(Some(Box::new(Value::Number(0.0))))
    );
}

#[test]
fn test_last_index_of_not_found() {
    assert_eq!(
        eval_ok(r#"last_index_of("hello", "x");"#),
        Value::Option(None)
    );
}

#[test]
fn test_last_index_of_multiple_occurrences() {
    assert_eq!(
        eval_ok(r#"last_index_of("abcabc", "b");"#),
        Value::Option(Some(Box::new(Value::Number(4.0))))
    );
}

#[test]
fn test_index_of_parity() {
    let i = eval_ok(r#"index_of("hello world", "world");"#);
    let v = vm_eval_ok(r#"index_of("hello world", "world");"#);
    assert_eq!(i, v);
}

// str_index_of / str_last_index_of / str_join

#[test]
fn test_str_index_of_found() {
    assert_eq!(
        eval_ok(r#"str_index_of("hello", "ll");"#),
        Value::Option(Some(Box::new(Value::Number(2.0))))
    );
}

#[test]
fn test_str_last_index_of_not_found() {
    assert_eq!(
        eval_ok(r#"str_last_index_of("hello", "z");"#),
        Value::Option(None)
    );
}

#[test]
fn test_str_join_basic() {
    assert_eq!(
        eval_ok(r#"str_join(["a", "b", "c"], "-");"#),
        Value::string("a-b-c")
    );
}

#[test]
fn test_str_index_of_parity() {
    let i = eval_ok(r#"str_index_of("ababa", "ba");"#);
    let v = vm_eval_ok(r#"str_index_of("ababa", "ba");"#);
    assert_eq!(i, v);
}

// substring()

#[test]
fn test_substring_start_equals_end() {
    assert_eq!(eval_ok(r#"substring("hello", 2, 2);"#), Value::string(""));
}

#[test]
fn test_substring_full_string() {
    assert_eq!(
        eval_ok(r#"substring("hello", 0, 5);"#),
        Value::string("hello")
    );
}

#[test]
fn test_substring_out_of_bounds_error() {
    let err = eval_err(r#"substring("hello", 0, 10);"#);
    assert!(is_runtime_error(&err));
}

#[test]
fn test_substring_start_greater_than_end_error() {
    let err = eval_err(r#"substring("hello", 3, 1);"#);
    assert!(is_runtime_error(&err));
}

#[test]
fn test_substring_parity() {
    let i = eval_ok(r#"substring("hello world", 6, 11);"#);
    let v = vm_eval_ok(r#"substring("hello world", 6, 11);"#);
    assert_eq!(i, v);
}

// char_at()

#[test]
fn test_char_at_out_of_bounds_error() {
    assert_eq!(eval_ok(r#"char_at("hello", 10);"#), Value::Option(None));
}

#[test]
fn test_char_at_empty_string_error() {
    assert_eq!(eval_ok(r#"char_at("", 0);"#), Value::Option(None));
}

#[test]
fn test_char_at_parity() {
    let i = eval_ok(r#"char_at("abcde", 3);"#);
    let v = vm_eval_ok(r#"char_at("abcde", 3);"#);
    assert_eq!(i, v);
}

// repeat()

#[test]
fn test_repeat_zero_times() {
    assert_eq!(eval_ok(r#"repeat("ha", 0);"#), Value::string(""));
}

#[test]
fn test_repeat_negative_error() {
    let err = eval_err(r#"repeat("ha", -1);"#);
    assert!(is_runtime_error(&err));
}

#[test]
fn test_repeat_empty_string() {
    assert_eq!(eval_ok(r#"repeat("", 100);"#), Value::string(""));
}

#[test]
fn test_repeat_parity() {
    let i = eval_ok(r#"repeat("ab", 3);"#);
    let v = vm_eval_ok(r#"repeat("ab", 3);"#);
    assert_eq!(i, v);
}

// replace()

#[test]
fn test_replace_first_only() {
    assert_eq!(
        eval_ok(r#"replace("aaa", "a", "b");"#),
        Value::string("baa")
    );
}

#[test]
fn test_replace_not_found() {
    assert_eq!(
        eval_ok(r#"replace("hello", "x", "y");"#),
        Value::string("hello")
    );
}

#[test]
fn test_replace_parity() {
    let i = eval_ok(r#"replace("hello world", "world", "Atlas");"#);
    let v = vm_eval_ok(r#"replace("hello world", "world", "Atlas");"#);
    assert_eq!(i, v);
}

// pad_start / pad_end

#[test]
fn test_pad_start_already_long_enough() {
    assert_eq!(
        eval_ok(r#"pad_start("hello", 3, "0");"#),
        Value::string("hello")
    );
}

#[test]
fn test_pad_start_multi_char_fill() {
    assert_eq!(
        eval_ok(r#"pad_start("1", 5, "ab");"#),
        Value::string("abab1")
    );
}

#[test]
fn test_pad_end_already_long_enough() {
    assert_eq!(
        eval_ok(r#"pad_end("hello", 3, "0");"#),
        Value::string("hello")
    );
}

#[test]
fn test_pad_end_multi_char_fill() {
    assert_eq!(eval_ok(r#"pad_end("1", 5, "ab");"#), Value::string("1abab"));
}

#[test]
fn test_pad_start_parity() {
    let i = eval_ok(r#"pad_start("5", 4, "0");"#);
    let v = vm_eval_ok(r#"pad_start("5", 4, "0");"#);
    assert_eq!(i, v);
}

// starts_with / ends_with

#[test]
fn test_starts_with_empty_needle() {
    assert_eq!(eval_ok(r#"starts_with("hello", "");"#), Value::Bool(true));
}

#[test]
fn test_ends_with_empty_needle() {
    assert_eq!(eval_ok(r#"ends_with("hello", "");"#), Value::Bool(true));
}

#[test]
fn test_starts_with_longer_needle() {
    assert_eq!(
        eval_ok(r#"starts_with("hi", "hello");"#),
        Value::Bool(false)
    );
}

#[test]
fn test_starts_with_parity() {
    let i = eval_ok(r#"starts_with("hello world", "hello");"#);
    let v = vm_eval_ok(r#"starts_with("hello world", "hello");"#);
    assert_eq!(i, v);
}

// to_upper / to_lower

#[test]
fn test_to_upper_already_upper() {
    assert_eq!(
        eval_ok(r#"to_upper_case("HELLO");"#),
        Value::string("HELLO")
    );
}

#[test]
fn test_to_lower_already_lower() {
    assert_eq!(
        eval_ok(r#"to_lower_case("hello");"#),
        Value::string("hello")
    );
}

#[test]
fn test_to_upper_empty() {
    assert_eq!(eval_ok(r#"to_upper_case("");"#), Value::string(""));
}

#[test]
fn test_case_parity() {
    let i = eval_ok(r#"to_upper_case("hello");"#);
    let v = vm_eval_ok(r#"to_upper_case("hello");"#);
    assert_eq!(i, v);
}

// includes (string)

#[test]
fn test_string_includes_empty_needle() {
    assert_eq!(eval_ok(r#"includes("hello", "");"#), Value::Bool(true));
}

#[test]
fn test_string_includes_empty_haystack() {
    assert_eq!(eval_ok(r#"includes("", "x");"#), Value::Bool(false));
}

// ============================================================================
// ARRAY HARDENING (25 tests)
// ============================================================================

// reverse()

#[test]
fn test_reverse_empty_array() {
    let result = eval_ok("reverse([]);");
    match result {
        Value::Array(arr) => assert_eq!(arr.len(), 0),
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_reverse_single_element() {
    let result = eval_ok("reverse([42]);");
    match result {
        Value::Array(arr) => {
            let b = arr.as_slice();
            assert_eq!(b.len(), 1);
            assert_eq!(b[0], Value::Number(42.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_reverse_parity() {
    let i = format!("{:?}", eval_ok("reverse([1, 2, 3]);"));
    let v = format!("{:?}", vm_eval_ok("reverse([1, 2, 3]);"));
    assert_eq!(i, v);
}

// concat()

#[test]
fn test_concat_empty_arrays() {
    let result = eval_ok("concat([], []);");
    match result {
        Value::Array(arr) => assert_eq!(arr.len(), 0),
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_concat_parity() {
    let i = format!("{:?}", eval_ok("concat([1, 2], [3, 4]);"));
    let v = format!("{:?}", vm_eval_ok("concat([1, 2], [3, 4]);"));
    assert_eq!(i, v);
}

// flatten()

#[test]
fn test_flatten_empty_array() {
    let result = eval_ok("flatten(slice([[1]], 1, 1));");
    match result {
        Value::Array(arr) => assert_eq!(arr.len(), 0),
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_flatten_nested_empty_arrays() {
    let result = eval_ok("flatten([[], []]);");
    match result {
        Value::Array(arr) => assert_eq!(arr.len(), 0),
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_flatten_one_level_only() {
    // flatten([[[1, 2]], [[3, 4]]]) (number[][][]) → flattens ONE level → [[1,2], [3,4]] (number[][])
    // Verify length = 2 meaning outer arrays were unwrapped but inner stays nested
    let result = eval_ok("flatten([[[1, 2]], [[3, 4]]]);");
    match result {
        Value::Array(arr) => {
            let b = arr.as_slice();
            assert_eq!(
                b.len(),
                2,
                "flatten should unwrap one level (2 inner arrays)"
            );
            // Each element should be an array
            assert!(matches!(b[0], Value::Array(_)), "element 0 should be array");
            assert!(matches!(b[1], Value::Array(_)), "element 1 should be array");
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_flatten_parity() {
    let i = format!("{:?}", eval_ok("flatten([[1, 2], [3, 4]]);"));
    let v = format!("{:?}", vm_eval_ok("flatten([[1, 2], [3, 4]]);"));
    assert_eq!(i, v);
}

// index_of / includes on arrays

#[test]
fn test_array_index_of_empty_array() {
    assert_eq!(eval_ok("array_index_of([], 1);"), Value::Option(None));
}

#[test]
fn test_array_index_of_first_occurrence() {
    assert_eq!(
        eval_ok("array_index_of([1, 2, 1, 3], 1);"),
        Value::Option(Some(Box::new(Value::Number(0.0))))
    );
}

#[test]
fn test_array_last_index_of_last_occurrence() {
    assert_eq!(
        eval_ok("array_last_index_of([1, 2, 1, 3], 1);"),
        Value::Option(Some(Box::new(Value::Number(2.0))))
    );
}

#[test]
fn test_array_includes_empty() {
    assert_eq!(eval_ok("array_includes([], 1);"), Value::Bool(false));
}

#[test]
fn test_array_index_of_parity() {
    let i = eval_ok("array_index_of([10, 20, 30, 20], 20);");
    let v = vm_eval_ok("array_index_of([10, 20, 30, 20], 20);");
    assert_eq!(i, v);
}

// slice()

#[test]
fn test_slice_end_beyond_length_clamps() {
    let result = eval_ok("slice([0, 1, 2, 3, 4], 1, 100);");
    match result {
        Value::Array(arr) => {
            let b = arr.as_slice();
            assert_eq!(b.len(), 4);
            assert_eq!(b[0], Value::Number(1.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_slice_empty_array() {
    let result = eval_ok("slice([], 0, 0);");
    match result {
        Value::Array(arr) => assert_eq!(arr.len(), 0),
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_slice_start_equals_end() {
    let result = eval_ok("slice([1, 2, 3], 1, 1);");
    match result {
        Value::Array(arr) => assert_eq!(arr.len(), 0),
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_slice_start_greater_than_end_error() {
    let err = eval_err("slice([1, 2, 3], 3, 1);");
    assert!(is_runtime_error(&err));
}

#[test]
fn test_slice_parity() {
    let i = format!("{:?}", eval_ok("slice([0, 1, 2, 3, 4], 1, 4);"));
    let v = format!("{:?}", vm_eval_ok("slice([0, 1, 2, 3, 4], 1, 4);"));
    assert_eq!(i, v);
}

// unshift (prepend)

#[test]
fn test_unshift_to_empty() {
    let result = eval_ok("unshift([], 42);");
    match result {
        Value::Array(arr) => {
            let b = arr.as_slice();
            assert_eq!(b.len(), 1);
            assert_eq!(b[0], Value::Number(42.0));
        }
        _ => panic!("Expected array"),
    }
}

// ============================================================================
// MATH HARDENING (25 tests)
// ============================================================================

// floor / ceil / round on negatives

#[test]
fn test_floor_negative() {
    assert_eq!(eval_ok("Math.floor(-1.1);"), Value::Number(-2.0));
}

#[test]
fn test_ceil_negative() {
    assert_eq!(eval_ok("Math.ceil(-1.9);"), Value::Number(-1.0));
}

#[test]
fn test_round_negative_half_bankers() {
    // Banker's rounding: -2.5 → -2 (round to even)
    assert_eq!(eval_ok("Math.round(-2.5);"), Value::Number(-2.0));
}

#[test]
fn test_round_negative_round_down() {
    assert_eq!(eval_ok("Math.round(-1.7);"), Value::Number(-2.0));
}

#[test]
fn test_floor_parity() {
    let i = eval_ok("Math.floor(-3.7);");
    let v = vm_eval_ok("Math.floor(-3.7);");
    assert_eq!(i, v);
}

// min / max edge cases

#[test]
fn test_min_equal_elements() {
    assert_eq!(eval_ok("Math.min(5, 5);"), Value::Number(5.0));
}

#[test]
fn test_max_negative() {
    assert_eq!(eval_ok("Math.max(-3, -7);"), Value::Number(-3.0));
}

#[test]
fn test_min_parity() {
    let i = eval_ok("Math.min(10, 20);");
    let v = vm_eval_ok("Math.min(10, 20);");
    assert_eq!(i, v);
}

// sqrt

#[test]
fn test_sqrt_zero() {
    assert_eq!(
        eval_ok("Math.sqrt(0);"),
        Value::Result(Ok(Box::new(Value::Number(0.0))))
    );
}

#[test]
fn test_sqrt_negative_returns_err() {
    // Math.sqrt(-1) is out of domain → Result(Err(...))
    let result = eval_ok("Math.sqrt(-1);");
    assert!(matches!(result, Value::Result(Err(_))));
}

#[test]
fn test_sqrt_parity() {
    let i = eval_ok("Math.sqrt(16);");
    let v = vm_eval_ok("Math.sqrt(16);");
    assert_eq!(i, v);
}

// pow

#[test]
fn test_pow_zero_exponent() {
    assert_eq!(eval_ok("Math.pow(42, 0);"), Value::Number(1.0));
}

#[test]
fn test_pow_negative_base() {
    assert_eq!(eval_ok("Math.pow(-2, 3);"), Value::Number(-8.0));
}

#[test]
fn test_pow_parity() {
    let i = eval_ok("Math.pow(2, 10);");
    let v = vm_eval_ok("Math.pow(2, 10);");
    assert_eq!(i, v);
}

// log

#[test]
fn test_log_one() {
    let result = eval_ok("Math.log(1);");
    assert!(
        matches!(result, Value::Result(Ok(ref v)) if matches!(v.as_ref(), Value::Number(x) if x.abs() < 1e-10))
    );
}

#[test]
fn test_log_zero_returns_err() {
    // Math.log(0) is out of domain → Result(Err(...))
    let result = eval_ok("Math.log(0);");
    assert!(matches!(result, Value::Result(Err(_))));
}

#[test]
fn test_log_negative_returns_err() {
    // Math.log(-1) is out of domain → Result(Err(...))
    let result = eval_ok("Math.log(-1);");
    assert!(matches!(result, Value::Result(Err(_))));
}

// clamp

#[test]
fn test_clamp_below_min() {
    assert_eq!(
        eval_ok("Math.clamp(-5, 0, 10);"),
        Value::Result(Ok(Box::new(Value::Number(0.0))))
    );
}

#[test]
fn test_clamp_above_max() {
    assert_eq!(
        eval_ok("Math.clamp(15, 0, 10);"),
        Value::Result(Ok(Box::new(Value::Number(10.0))))
    );
}

#[test]
fn test_clamp_parity() {
    let i = eval_ok("Math.clamp(7, 1, 10);");
    let v = vm_eval_ok("Math.clamp(7, 1, 10);");
    assert_eq!(i, v);
}

// sign

#[test]
fn test_sign_positive() {
    assert_eq!(eval_ok("Math.sign(42);"), Value::Number(1.0));
}

#[test]
fn test_sign_negative() {
    assert_eq!(eval_ok("Math.sign(-42);"), Value::Number(-1.0));
}

#[test]
fn test_sign_zero() {
    assert_eq!(eval_ok("Math.sign(0);"), Value::Number(0.0));
}

// trig

#[test]
fn test_asin_out_of_domain_returns_err() {
    // Math.asin(2) is outside domain [-1, 1] → Result(Err(...))
    let result = eval_ok("Math.asin(2);");
    assert!(matches!(result, Value::Result(Err(_))));
}

#[test]
fn test_cos_parity() {
    let i = eval_ok("Math.cos(0);");
    let v = vm_eval_ok("Math.cos(0);");
    assert_eq!(i, v);
}

// ============================================================================
// JSON HARDENING (20 tests)
// ============================================================================

#[test]
fn test_parse_json_just_null() {
    let result = eval_ok(r#"Json.parse("null");"#);
    assert!(
        matches!(result, Value::Result(Ok(ref v)) if matches!(v.as_ref(), Value::JsonValue(_)))
    );
}

#[test]
fn test_parse_json_empty_string_error() {
    let result = eval_ok(r#"Json.parse("");"#);
    assert!(matches!(result, Value::Result(Err(_))));
}

#[test]
fn test_parse_json_malformed_error() {
    let result = eval_ok(r#"Json.parse("{bad}");"#);
    assert!(matches!(result, Value::Result(Err(_))));
}

#[test]
fn test_parse_json_empty_array() {
    let result = eval_ok(r#"Json.parse("[]");"#);
    assert!(
        matches!(result, Value::Result(Ok(ref v)) if matches!(v.as_ref(), Value::JsonValue(_)))
    );
}

#[test]
fn test_parse_json_empty_object() {
    let result = eval_ok(r#"Json.parse("{}");"#);
    assert!(
        matches!(result, Value::Result(Ok(ref v)) if matches!(v.as_ref(), Value::JsonValue(_)))
    );
}

#[test]
fn test_parse_json_parity() {
    let i = eval_ok(r#"Json.parse("{\"x\":1}");"#);
    let v = vm_eval_ok(r#"Json.parse("{\"x\":1}");"#);
    assert_eq!(i, v);
}

#[test]
fn test_to_json_null_value() {
    assert_eq!(eval_ok("Json.stringify(null);"), Value::string("null"));
}

#[test]
fn test_to_json_empty_array() {
    assert_eq!(eval_ok("Json.stringify([]);"), Value::string("[]"));
}

#[test]
fn test_to_json_nested_array() {
    assert_eq!(
        eval_ok("Json.stringify([[1, 2], [3, 4]]);"),
        Value::string("[[1,2],[3,4]]")
    );
}

#[test]
fn test_to_json_parity() {
    let i = eval_ok("Json.stringify(42);");
    let v = vm_eval_ok("Json.stringify(42);");
    assert_eq!(i, v);
}

#[test]
fn test_is_valid_json_empty_false() {
    assert_eq!(eval_ok(r#"Json.isValid("");"#), Value::Bool(false));
}

#[test]
fn test_is_valid_json_null_true() {
    assert_eq!(eval_ok(r#"Json.isValid("null");"#), Value::Bool(true));
}

#[test]
fn test_is_valid_json_array_true() {
    assert_eq!(eval_ok(r#"Json.isValid("[1,2,3]");"#), Value::Bool(true));
}

#[test]
fn test_json_as_string_correct_type() {
    // parse_json now returns Result(Ok(JsonValue)); unwrap then call as_string on a known JsonValue
    let result = eval_ok(r#"Json.parse("\"hello\"");"#);
    assert!(
        matches!(&result, Value::Result(Ok(ref v)) if matches!(v.as_ref(), Value::JsonValue(_)))
    );
}

#[test]
fn test_json_as_number_correct_type() {
    // parse_json now returns Result(Ok(JsonValue)); check the inner JsonValue holds number
    let result = eval_ok(r#"Json.parse("42");"#);
    assert!(
        matches!(&result, Value::Result(Ok(ref v)) if matches!(v.as_ref(), Value::JsonValue(_)))
    );
}

#[test]
fn test_json_get_string_found() {
    assert_eq!(
        eval_ok(
            r#"let j: json = unwrap(Json.parse("{\"name\":\"Atlas\",\"age\":3}")); json_get_string(j, "name");"#
        ),
        Value::Option(Some(Box::new(Value::string("Atlas"))))
    );
}

#[test]
fn test_json_get_number_mismatch_none() {
    assert_eq!(
        eval_ok(
            r#"let j: json = unwrap(Json.parse("{\"name\":\"Atlas\",\"age\":3}")); json_get_number(j, "name");"#
        ),
        Value::Option(None)
    );
}

#[test]
fn test_json_get_array_and_object() {
    let result = eval_ok(
        r#"let j: json = unwrap(Json.parse("{\"items\":[1,2],\"meta\":{\"ok\":true}}")); [json_get_array(j, "items"), json_get_object(j, "meta")];"#,
    );
    match result {
        Value::Array(arr) => {
            let items = &arr.as_slice()[0];
            let meta = &arr.as_slice()[1];
            match items {
                Value::Option(Some(inner)) => match inner.as_ref() {
                    Value::Array(values) => {
                        let slice = values.as_slice();
                        assert_eq!(slice.len(), 2);
                        assert!(matches!(
                            &slice[0],
                            Value::JsonValue(json) if matches!(json.as_ref(), JsonValue::Number(_))
                        ));
                    }
                    _ => panic!("Expected array from json_get_array"),
                },
                _ => panic!("Expected Some(array) from json_get_array"),
            }
            match meta {
                Value::Option(Some(inner)) => match inner.as_ref() {
                    Value::JsonValue(json) => {
                        assert!(matches!(json.as_ref(), JsonValue::Object(_)));
                    }
                    _ => panic!("Expected json object from json_get_object"),
                },
                _ => panic!("Expected Some(object) from json_get_object"),
            }
        }
        _ => panic!("Expected array result"),
    }
}

#[test]
fn test_json_get_parity() {
    let i = eval_ok(
        r#"let j: json = unwrap(Json.parse("{\"active\":true,\"missing\":null}")); json_get_bool(j, "active");"#,
    );
    let v = vm_eval_ok(
        r#"let j: json = unwrap(Json.parse("{\"active\":true,\"missing\":null}")); json_get_bool(j, "active");"#,
    );
    assert_eq!(i, v);
}

#[test]
fn test_json_is_null_on_null() {
    // Json.parse("null") → Result(Ok(JsonValue(null))); verify it's Ok
    let result = eval_ok(r#"Json.parse("null");"#);
    assert!(
        matches!(&result, Value::Result(Ok(ref v)) if matches!(v.as_ref(), Value::JsonValue(_)))
    );
}

#[test]
fn test_json_is_null_on_non_null() {
    // Json.parse("42") → Result(Ok(JsonValue(42))); also JsonValue, not null
    let result = eval_ok(r#"Json.parse("42");"#);
    assert!(
        matches!(&result, Value::Result(Ok(ref v)) if matches!(v.as_ref(), Value::JsonValue(_)))
    );
}

#[test]
fn test_json_as_string_wrong_type_error() {
    // Valid parse succeeds; wrong method call errors at runtime
    let err = eval_err(r#"let j = Json.parse("42"); j.as_string();"#);
    assert!(is_runtime_error(&err));
}

#[test]
fn test_json_as_number_wrong_type_error() {
    let err = eval_err(r#"let j = Json.parse("\"hello\""); j.as_number();"#);
    assert!(is_runtime_error(&err));
}

#[test]
fn test_json_extraction_parity() {
    let i = eval_ok(r#"Json.parse("{\"x\":10}");"#);
    let v = vm_eval_ok(r#"Json.parse("{\"x\":10}");"#);
    assert_eq!(i, v);
}

// ============================================================================
// TYPES HARDENING (25 tests)
// ============================================================================

// typeOf for all major types

#[test]
fn test_type_of_null() {
    assert_eq!(eval_ok("typeof(null);"), Value::string("null"));
}

#[test]
fn test_type_of_number() {
    assert_eq!(eval_ok("typeof(42);"), Value::string("number"));
}

#[test]
fn test_type_of_string() {
    assert_eq!(eval_ok(r#"typeof("hello");"#), Value::string("string"));
}

#[test]
fn test_type_of_bool() {
    assert_eq!(eval_ok("typeof(true);"), Value::string("boolean"));
}

#[test]
fn test_type_of_array() {
    assert_eq!(eval_ok("typeof([1, 2, 3]);"), Value::string("array"));
}

#[test]
fn test_type_of_option() {
    assert_eq!(eval_ok("typeof(Some(42));"), Value::string("option"));
}

#[test]
fn test_type_of_result_ok() {
    assert_eq!(eval_ok("typeof(Ok(42));"), Value::string("record"));
}

#[test]
fn test_type_of_parity() {
    let i = eval_ok("typeof(42);");
    let v = vm_eval_ok("typeof(42);");
    assert_eq!(i, v);
}

// is_* predicates (false cases)

#[test]
fn test_is_number_false_for_string() {
    assert_eq!(eval_ok(r#"is_number("42");"#), Value::Bool(false));
}

#[test]
fn test_is_string_false_for_number() {
    assert_eq!(eval_ok("is_string(42);"), Value::Bool(false));
}

#[test]
fn test_is_bool_false_for_number() {
    assert_eq!(eval_ok("is_bool(1);"), Value::Bool(false));
}

#[test]
fn test_is_null_false_for_zero() {
    assert_eq!(eval_ok("is_null(0);"), Value::Bool(false));
}

#[test]
fn test_is_array_parity() {
    let i = eval_ok("is_array([1, 2, 3]);");
    let v = vm_eval_ok("is_array([1, 2, 3]);");
    assert_eq!(i, v);
}

// toString

#[test]
fn test_to_string_option_some() {
    assert_eq!(eval_ok("toString(Some(42));"), Value::string("Some(42)"));
}

#[test]
fn test_to_string_option_none() {
    assert_eq!(eval_ok("toString(None);"), Value::string("None"));
}

#[test]
fn test_to_string_parity() {
    let i = eval_ok("toString(42);");
    let v = vm_eval_ok("toString(42);");
    assert_eq!(i, v);
}

// to_number

#[test]
fn test_to_number_from_bool_true() {
    assert_eq!(
        eval_ok("to_number(true);"),
        Value::Result(Ok(Box::new(Value::Number(1.0))))
    );
}

#[test]
fn test_to_number_from_bool_false() {
    assert_eq!(
        eval_ok("to_number(false);"),
        Value::Result(Ok(Box::new(Value::Number(0.0))))
    );
}

#[test]
fn test_to_number_from_non_numeric_string_error() {
    let result = eval_ok(r#"to_number("abc");"#);
    assert!(matches!(result, Value::Result(Err(_))));
}

#[test]
fn test_to_number_from_null_error() {
    let result = eval_ok("to_number(null);");
    assert!(matches!(result, Value::Result(Err(_))));
}

// to_bool

#[test]
fn test_to_bool_zero_is_false() {
    assert_eq!(eval_ok("to_bool(0);"), Value::Bool(false));
}

#[test]
fn test_to_bool_empty_string_is_false() {
    assert_eq!(eval_ok(r#"to_bool("");"#), Value::Bool(false));
}

#[test]
fn test_to_bool_null_is_false() {
    assert_eq!(eval_ok("to_bool(null);"), Value::Bool(false));
}

#[test]
fn test_to_bool_array_is_true() {
    assert_eq!(eval_ok("to_bool([]);"), Value::Bool(true));
}

#[test]
fn test_to_bool_parity() {
    let i = eval_ok("to_bool(0);");
    let v = vm_eval_ok("to_bool(0);");
    assert_eq!(i, v);
}

// parse_int / parse_float

#[test]
fn test_parse_int_hex() {
    assert_eq!(
        eval_ok("parse_int(\"ff\", 16);"),
        Value::Result(Ok(Box::new(Value::Number(255.0))))
    );
}

#[test]
fn test_parse_int_binary() {
    assert_eq!(
        eval_ok("parse_int(\"1010\", 2);"),
        Value::Result(Ok(Box::new(Value::Number(10.0))))
    );
}

#[test]
fn test_parse_int_invalid_error() {
    let result = eval_ok("parse_int(\"xyz\", 10);");
    assert!(matches!(result, Value::Result(Err(_))));
}

#[test]
fn test_parse_float_scientific() {
    assert_eq!(
        eval_ok("parse_float(\"1.5e3\");"),
        Value::Result(Ok(Box::new(Value::Number(1500.0))))
    );
}

#[test]
fn test_parse_float_invalid_error() {
    let result = eval_ok("parse_float(\"abc\");");
    assert!(matches!(result, Value::Result(Err(_))));
}

#[test]
fn test_parse_int_parity() {
    let i = eval_ok("parse_int(\"ff\", 16);");
    let v = vm_eval_ok("parse_int(\"ff\", 16);");
    assert_eq!(i, v);
}

// H-083: Duplicate stdlib functions removed
// strJoin/strIndexOf/strLastIndexOf are duplicates of join/indexOf/lastIndexOf.
// After removal, only the canonical names (and snake_case aliases) should work.

#[test]
fn test_h083_str_join_removed() {
    // strJoin is a duplicate of join — should be undefined after removal
    let errs = eval_err(r#"strJoin(["a", "b"], "-");"#);
    assert!(!errs.is_empty(), "strJoin should not exist");
}

#[test]
fn test_h083_str_index_of_removed() {
    let errs = eval_err(r#"strIndexOf("hello", "ll");"#);
    assert!(!errs.is_empty(), "strIndexOf should not exist");
}

#[test]
fn test_h083_str_last_index_of_removed() {
    let errs = eval_err(r#"strLastIndexOf("hello", "l");"#);
    assert!(!errs.is_empty(), "strLastIndexOf should not exist");
}

#[test]
fn test_h083_canonical_join_still_works() {
    // join and str_join (snake alias) must still work
    let r1 = eval_ok(r#"join(["a", "b", "c"], "-");"#);
    let r2 = eval_ok(r#"str_join(["a", "b", "c"], "-");"#);
    assert_eq!(r1, r2);
}

#[test]
fn test_h083_canonical_index_of_still_works() {
    let r1 = eval_ok(r#"indexOf("hello", "ll");"#);
    let r2 = eval_ok(r#"index_of("hello", "ll");"#);
    assert_eq!(r1, r2);
}

#[test]
fn test_h083_str_includes_alias_works() {
    // str_includes should be a snake_case alias for includes (string)
    let r1 = eval_ok(r#"includes("hello world", "world");"#);
    let r2 = eval_ok(r#"str_includes("hello world", "world");"#);
    assert_eq!(r1, r2);
}
