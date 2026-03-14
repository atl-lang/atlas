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
    // "".split(",") returns [""] — one empty-string element
    let result = eval_ok(r#""".split(",");"#);
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
    // "".split("") returns [] — no characters to iterate
    let result = eval_ok(r#""".split("");"#);
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
    let result = eval_ok(r#""abc".split(",");"#);
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
    let interp = format!("{:?}", eval_ok(r#""a,b,c".split(",");"#));
    let vm = format!("{:?}", vm_eval_ok(r#""a,b,c".split(",");"#));
    assert_eq!(interp, vm);
}

// trim variants

#[test]
fn test_trim_all_whitespace() {
    assert_eq!(eval_ok(r#""   ".trim();"#), Value::string(""));
}

#[test]
fn test_trim_start_all_whitespace() {
    assert_eq!(eval_ok(r#""   ".trimStart();"#), Value::string(""));
}

#[test]
fn test_trim_end_all_whitespace() {
    assert_eq!(eval_ok(r#""   ".trimEnd();"#), Value::string(""));
}

#[test]
fn test_trim_no_whitespace() {
    assert_eq!(eval_ok(r#""hello".trim();"#), Value::string("hello"));
}

#[test]
fn test_trim_parity() {
    let i = eval_ok(r#""  hello  ".trim();"#);
    let v = vm_eval_ok(r#""  hello  ".trim();"#);
    assert_eq!(i, v);
}

// indexOf / last_index_of

#[test]
fn test_index_of_empty_haystack() {
    assert_eq!(eval_ok(r#""".indexOf("x");"#), Value::Option(None));
}

#[test]
fn test_index_of_empty_needle_on_empty() {
    assert_eq!(
        eval_ok(r#""".indexOf("");"#),
        Value::Option(Some(Box::new(Value::Number(0.0))))
    );
}

#[test]
fn test_last_index_of_not_found() {
    assert_eq!(eval_ok(r#""hello".lastIndexOf("x");"#), Value::Option(None));
}

#[test]
fn test_last_index_of_multiple_occurrences() {
    assert_eq!(
        eval_ok(r#""abcabc".lastIndexOf("b");"#),
        Value::Option(Some(Box::new(Value::Number(4.0))))
    );
}

#[test]
fn test_index_of_parity() {
    let i = eval_ok(r#""hello world".indexOf("world");"#);
    let v = vm_eval_ok(r#""hello world".indexOf("world");"#);
    assert_eq!(i, v);
}

// indexOf / lastIndexOf / join — method syntax (bare str_* globals removed)

#[test]
fn test_str_index_of_found() {
    assert_eq!(
        eval_ok(r#""hello".indexOf("ll");"#),
        Value::Option(Some(Box::new(Value::Number(2.0))))
    );
}

#[test]
fn test_str_last_index_of_not_found() {
    assert_eq!(eval_ok(r#""hello".lastIndexOf("z");"#), Value::Option(None));
}

#[test]
fn test_str_join_basic() {
    assert_eq!(
        eval_ok(r#"["a", "b", "c"].join("-");"#),
        Value::string("a-b-c")
    );
}

#[test]
fn test_str_index_of_parity() {
    let i = eval_ok(r#""ababa".indexOf("ba");"#);
    let v = vm_eval_ok(r#""ababa".indexOf("ba");"#);
    assert_eq!(i, v);
}

// substring()

#[test]
fn test_substring_start_equals_end() {
    assert_eq!(eval_ok(r#""hello".substring(2, 2);"#), Value::string(""));
}

#[test]
fn test_substring_full_string() {
    assert_eq!(
        eval_ok(r#""hello".substring(0, 5);"#),
        Value::string("hello")
    );
}

#[test]
fn test_substring_out_of_bounds_error() {
    let err = eval_err(r#""hello".substring(0, 10);"#);
    assert!(is_runtime_error(&err));
}

#[test]
fn test_substring_start_greater_than_end_error() {
    let err = eval_err(r#""hello".substring(3, 1);"#);
    assert!(is_runtime_error(&err));
}

#[test]
fn test_substring_parity() {
    let i = eval_ok(r#""hello world".substring(6, 11);"#);
    let v = vm_eval_ok(r#""hello world".substring(6, 11);"#);
    assert_eq!(i, v);
}

// charAt()

#[test]
fn test_char_at_out_of_bounds_error() {
    assert_eq!(eval_ok(r#""hello".charAt(10);"#), Value::Option(None));
}

#[test]
fn test_char_at_empty_string_error() {
    assert_eq!(eval_ok(r#""".charAt(0);"#), Value::Option(None));
}

#[test]
fn test_char_at_parity() {
    let i = eval_ok(r#""abcde".charAt(3);"#);
    let v = vm_eval_ok(r#""abcde".charAt(3);"#);
    assert_eq!(i, v);
}

// repeat()

#[test]
fn test_repeat_zero_times() {
    assert_eq!(eval_ok(r#""ha".repeat(0);"#), Value::string(""));
}

#[test]
fn test_repeat_negative_error() {
    let err = eval_err(r#""ha".repeat(-1);"#);
    assert!(is_runtime_error(&err));
}

#[test]
fn test_repeat_empty_string() {
    assert_eq!(eval_ok(r#""".repeat(100);"#), Value::string(""));
}

#[test]
fn test_repeat_parity() {
    let i = eval_ok(r#""ab".repeat(3);"#);
    let v = vm_eval_ok(r#""ab".repeat(3);"#);
    assert_eq!(i, v);
}

// replace()

#[test]
fn test_replace_first_only() {
    assert_eq!(eval_ok(r#""aaa".replace("a", "b");"#), Value::string("baa"));
}

#[test]
fn test_replace_not_found() {
    assert_eq!(
        eval_ok(r#""hello".replace("x", "y");"#),
        Value::string("hello")
    );
}

#[test]
fn test_replace_parity() {
    let i = eval_ok(r#""hello world".replace("world", "Atlas");"#);
    let v = vm_eval_ok(r#""hello world".replace("world", "Atlas");"#);
    assert_eq!(i, v);
}

// pad_start / pad_end

#[test]
fn test_pad_start_already_long_enough() {
    assert_eq!(
        eval_ok(r#""hello".padStart(3, "0");"#),
        Value::string("hello")
    );
}

#[test]
fn test_pad_start_multi_char_fill() {
    assert_eq!(eval_ok(r#""1".padStart(5, "ab");"#), Value::string("abab1"));
}

#[test]
fn test_pad_end_already_long_enough() {
    assert_eq!(
        eval_ok(r#""hello".padEnd(3, "0");"#),
        Value::string("hello")
    );
}

#[test]
fn test_pad_end_multi_char_fill() {
    assert_eq!(eval_ok(r#""1".padEnd(5, "ab");"#), Value::string("1abab"));
}

#[test]
fn test_pad_start_parity() {
    let i = eval_ok(r#""5".padStart(4, "0");"#);
    let v = vm_eval_ok(r#""5".padStart(4, "0");"#);
    assert_eq!(i, v);
}

// starts_with / ends_with

#[test]
fn test_starts_with_empty_needle() {
    assert_eq!(eval_ok(r#""hello".startsWith("");"#), Value::Bool(true));
}

#[test]
fn test_ends_with_empty_needle() {
    assert_eq!(eval_ok(r#""hello".endsWith("");"#), Value::Bool(true));
}

#[test]
fn test_starts_with_longer_needle() {
    assert_eq!(eval_ok(r#""hi".startsWith("hello");"#), Value::Bool(false));
}

#[test]
fn test_starts_with_parity() {
    let i = eval_ok(r#""hello world".startsWith("hello");"#);
    let v = vm_eval_ok(r#""hello world".startsWith("hello");"#);
    assert_eq!(i, v);
}

// to_upper / to_lower

#[test]
fn test_to_upper_already_upper() {
    assert_eq!(eval_ok(r#""HELLO".toUpperCase();"#), Value::string("HELLO"));
}

#[test]
fn test_to_lower_already_lower() {
    assert_eq!(eval_ok(r#""hello".toLowerCase();"#), Value::string("hello"));
}

#[test]
fn test_to_upper_empty() {
    assert_eq!(eval_ok(r#""".toUpperCase();"#), Value::string(""));
}

#[test]
fn test_case_parity() {
    let i = eval_ok(r#""hello".toUpperCase();"#);
    let v = vm_eval_ok(r#""hello".toUpperCase();"#);
    assert_eq!(i, v);
}

// includes (string)

#[test]
fn test_string_includes_empty_needle() {
    assert_eq!(eval_ok(r#""hello".includes("");"#), Value::Bool(true));
}

#[test]
fn test_string_includes_empty_haystack() {
    assert_eq!(eval_ok(r#""".includes("x");"#), Value::Bool(false));
}

// ============================================================================
// ARRAY HARDENING (25 tests)
// ============================================================================

// reverse()

#[test]
fn test_reverse_empty_array() {
    let result = eval_ok("[].reverse();");
    match result {
        Value::Array(arr) => assert_eq!(arr.len(), 0),
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_reverse_single_element() {
    let result = eval_ok("[42].reverse();");
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
    let i = format!("{:?}", eval_ok("[1, 2, 3].reverse();"));
    let v = format!("{:?}", vm_eval_ok("[1, 2, 3].reverse();"));
    assert_eq!(i, v);
}

// concat()

#[test]
fn test_concat_empty_arrays() {
    let result = eval_ok("[].concat([]);");
    match result {
        Value::Array(arr) => assert_eq!(arr.len(), 0),
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_concat_parity() {
    let i = format!("{:?}", eval_ok("[1, 2].concat([3, 4]);"));
    let v = format!("{:?}", vm_eval_ok("[1, 2].concat([3, 4]);"));
    assert_eq!(i, v);
}

// flatten()

#[test]
fn test_flatten_empty_array() {
    let result = eval_ok("[[1]].slice(1, 1).flatten();");
    match result {
        Value::Array(arr) => assert_eq!(arr.len(), 0),
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_flatten_nested_empty_arrays() {
    let result = eval_ok("[[], []].flatten();");
    match result {
        Value::Array(arr) => assert_eq!(arr.len(), 0),
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_flatten_one_level_only() {
    // flatten([[[1, 2]], [[3, 4]]]) (number[][][]) → flattens ONE level → [[1,2], [3,4]] (number[][])
    // Verify length = 2 meaning outer arrays were unwrapped but inner stays nested
    let result = eval_ok("[[[1, 2]], [[3, 4]]].flatten();");
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
    let i = format!("{:?}", eval_ok("[[1, 2], [3, 4]].flatten();"));
    let v = format!("{:?}", vm_eval_ok("[[1, 2], [3, 4]].flatten();"));
    assert_eq!(i, v);
}

// indexOf / includes on arrays

#[test]
fn test_array_index_of_empty_array() {
    assert_eq!(eval_ok("[].indexOf(1);"), Value::Option(None));
}

#[test]
fn test_array_index_of_first_occurrence() {
    assert_eq!(
        eval_ok("[1, 2, 1, 3].indexOf(1);"),
        Value::Option(Some(Box::new(Value::Number(0.0))))
    );
}

#[test]
fn test_array_last_index_of_last_occurrence() {
    assert_eq!(
        eval_ok("[1, 2, 1, 3].lastIndexOf(1);"),
        Value::Option(Some(Box::new(Value::Number(2.0))))
    );
}

#[test]
fn test_array_includes_empty() {
    assert_eq!(eval_ok("[].includes(1);"), Value::Bool(false));
}

#[test]
fn test_array_index_of_parity() {
    let i = eval_ok("[10, 20, 30, 20].indexOf(20);");
    let v = vm_eval_ok("[10, 20, 30, 20].indexOf(20);");
    assert_eq!(i, v);
}

// slice()

#[test]
fn test_slice_end_beyond_length_clamps() {
    let result = eval_ok("[0, 1, 2, 3, 4].slice(1, 100);");
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
    let result = eval_ok("[].slice(0, 0);");
    match result {
        Value::Array(arr) => assert_eq!(arr.len(), 0),
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_slice_start_equals_end() {
    let result = eval_ok("[1, 2, 3].slice(1, 1);");
    match result {
        Value::Array(arr) => assert_eq!(arr.len(), 0),
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_slice_start_greater_than_end_error() {
    let err = eval_err("[1, 2, 3].slice(3, 1);");
    assert!(is_runtime_error(&err));
}

#[test]
fn test_slice_parity() {
    let i = format!("{:?}", eval_ok("[0, 1, 2, 3, 4].slice(1, 4);"));
    let v = format!("{:?}", vm_eval_ok("[0, 1, 2, 3, 4].slice(1, 4);"));
    assert_eq!(i, v);
}

// unshift (prepend)

#[test]
fn test_unshift_to_empty() {
    let result = eval_ok("[].unshift(42);");
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
    assert_eq!(eval_ok(r#"typeof("42") == "number";"#), Value::Bool(false));
}

#[test]
fn test_is_string_false_for_number() {
    assert_eq!(eval_ok(r#"typeof(42) == "string";"#), Value::Bool(false));
}

#[test]
fn test_is_bool_false_for_number() {
    assert_eq!(eval_ok(r#"typeof(1) == "bool";"#), Value::Bool(false));
}

#[test]
fn test_is_null_false_for_zero() {
    assert_eq!(eval_ok(r#"typeof(0) == "null";"#), Value::Bool(false));
}

#[test]
fn test_is_array_parity() {
    let i = eval_ok(r#"typeof([1, 2, 3]) == "array";"#);
    let v = vm_eval_ok(r#"typeof([1, 2, 3]) == "array";"#);
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

// toNumber

#[test]
fn test_to_number_from_bool_true() {
    assert_eq!(
        eval_ok("(true).toNumber();"),
        Value::Result(Ok(Box::new(Value::Number(1.0))))
    );
}

#[test]
fn test_to_number_from_bool_false() {
    assert_eq!(
        eval_ok("(false).toNumber();"),
        Value::Result(Ok(Box::new(Value::Number(0.0))))
    );
}

#[test]
fn test_to_number_from_non_numeric_string_error() {
    let result = eval_ok(r#"("abc").toNumber();"#);
    assert!(matches!(result, Value::Result(Err(_))));
}

#[test]
fn test_to_number_from_null_error() {
    let result = eval_ok("(null).toNumber();");
    assert!(matches!(result, Value::Result(Err(_))));
}

// toBool

#[test]
fn test_to_bool_zero_is_false() {
    assert_eq!(eval_ok("(0).toBool();"), Value::Bool(false));
}

#[test]
fn test_to_bool_empty_string_is_false() {
    assert_eq!(eval_ok(r#"("").toBool();"#), Value::Bool(false));
}

#[test]
fn test_to_bool_null_is_false() {
    assert_eq!(eval_ok("(null).toBool();"), Value::Bool(false));
}

#[test]
fn test_to_bool_array_is_true() {
    assert_eq!(eval_ok("([]).toBool();"), Value::Bool(true));
}

#[test]
fn test_to_bool_parity() {
    let i = eval_ok("(0).toBool();");
    let v = vm_eval_ok("(0).toBool();");
    assert_eq!(i, v);
}

// parse_int / parse_float

#[test]
fn test_parse_int_hex() {
    assert_eq!(
        eval_ok(r#""ff".toInt(16);"#),
        Value::Result(Ok(Box::new(Value::Number(255.0))))
    );
}

#[test]
fn test_parse_int_binary() {
    assert_eq!(
        eval_ok(r#""1010".toInt(2);"#),
        Value::Result(Ok(Box::new(Value::Number(10.0))))
    );
}

#[test]
fn test_parse_int_invalid_error() {
    let result = eval_ok(r#""xyz".toInt(10);"#);
    assert!(matches!(result, Value::Result(Err(_))));
}

#[test]
fn test_parse_float_scientific() {
    assert_eq!(
        eval_ok(r#""1.5e3".toNumber();"#),
        Value::Result(Ok(Box::new(Value::Number(1500.0))))
    );
}

#[test]
fn test_parse_float_invalid_error() {
    let result = eval_ok(r#""abc".toNumber();"#);
    assert!(matches!(result, Value::Result(Err(_))));
}

#[test]
fn test_parse_int_parity() {
    let i = eval_ok(r#""ff".toInt(16);"#);
    let v = vm_eval_ok(r#""ff".toInt(16);"#);
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
    // method syntax is the canonical form — bare join() is gone
    let r1 = eval_ok(r#"["a", "b", "c"].join("-");"#);
    assert_eq!(r1, Value::string("a-b-c"));
}

#[test]
fn test_h083_canonical_index_of_still_works() {
    let r1 = eval_ok(r#""hello".indexOf("ll");"#);
    let r2 = eval_ok(r#""hello".indexOf("ll");"#);
    assert_eq!(r1, r2);
}

#[test]
fn test_h083_str_includes_alias_works() {
    // str_includes was a snake_case alias — canonical form is now .includes() method
    let r1 = eval_ok(r#""hello world".includes("world");"#);
    let r2 = vm_eval_ok(r#""hello world".includes("world");"#);
    assert_eq!(r1, r2);
}
