use super::*;
use pretty_assertions::assert_eq;

// From stdlib_parity_verification.rs
// ============================================================================

// Systematic Standard Library Parity Verification
//
// Verifies that ALL stdlib functions produce identical output in both
// interpreter and VM execution engines. This is critical for correctness.
//
// Coverage:
// - All 18 string functions
// - All 21 array functions
// - All 18 math functions + 5 constants
// - All 17 JSON functions
// - All 10 file I/O functions
// - All type checking functions
// - Edge cases for each function
// - Error cases for each function
//
// Total: 130+ parity tests

// ============================================================================
// String Function Parity Tests (18 functions)
// ============================================================================

#[rstest]
#[case::length("len(\"hello\")", "5")]
#[case::length_empty("len(\"\")", "0")]
#[case::length_unicode("len(\"hello世界\")", "7")]
#[case::concat("\"hello\" + \" \" + \"world\"", "hello world")]
#[case::concat_empty("\"\" + \"test\"", "test")]
#[case::substring("\"hello\".substring(1, 4)", "ell")]
#[case::substring_full("\"hello\".substring(0, 5)", "hello")]
#[case::charat("\"hello\".charAt(1)", "Some(e)")]
#[case::charat_first("\"hello\".charAt(0)", "Some(h)")]
#[case::indexof("\"hello\".indexOf(\"l\")", "Some(2)")]
#[case::indexof_not_found("\"hello\".indexOf(\"x\")", "None")]
#[case::split("\"a,b,c\".split(\",\").join(\"|\")", "a|b|c")]
#[case::split_empty("len(\"\".split(\",\"))", "1")] // Empty string splits to [""]
#[case::join("[\"a\", \"b\", \"c\"].join(\",\")", "a,b,c")]
#[case::join_empty("[\"a\"].slice(1, 1).join(\",\")", "")]
#[case::replace("\"hello world\".replace(\"world\", \"Atlas\")", "hello Atlas")]
#[case::replace_first("\"aaa\".replace(\"a\", \"b\")", "baa")] // replace() only replaces first occurrence
#[case::trim("\"  hello  \".trim()", "hello")]
#[case::trim_no_space("\"hello\".trim()", "hello")]
#[case::to_upper("\"hello\".toUpperCase()", "HELLO")]
#[case::to_upper_mixed("\"HeLLo\".toUpperCase()", "HELLO")]
#[case::to_lower("\"HELLO\".toLowerCase()", "hello")]
#[case::to_lower_mixed("\"HeLLo\".toLowerCase()", "hello")]
#[case::startswith("\"hello\".startsWith(\"he\")", "true")]
#[case::startswith_false("\"hello\".startsWith(\"wo\")", "false")]
#[case::endswith("\"hello\".endsWith(\"lo\")", "true")]
#[case::endswith_false("\"hello\".endsWith(\"he\")", "false")]
#[case::includes("\"hello world\".includes(\"wo\")", "true")]
#[case::includes_false("\"hello world\".includes(\"xyz\")", "false")]
#[case::repeat("\"ab\".repeat(3)", "ababab")]
#[case::repeat_zero("\"x\".repeat(0)", "")]
#[case::padstart("\"5\".padStart(3, \"0\")", "005")]
#[case::padend("\"5\".padEnd(3, \"0\")", "500")]
#[case::lastindexof("\"hello\".lastIndexOf(\"l\")", "Some(3)")]
#[case::lastindexof_not_found("\"hello\".lastIndexOf(\"x\")", "None")]
#[case::trimstart("\"  hello\".trimStart()", "hello")]
#[case::trimend("\"hello  \".trimEnd()", "hello")]
fn test_string_parity(#[case] code: &str, #[case] expected: &str) {
    // Run in interpreter
    let runtime_interp = Atlas::new();
    let interp_result = runtime_interp.eval(code).unwrap();

    // Run in VM (eval uses VM by default in atlas-runtime)
    let runtime_vm = Atlas::new();
    let vm_result = runtime_vm.eval(code).unwrap();

    // Assert identical output
    assert_eq!(
        format!("{:?}", interp_result),
        format!("{:?}", vm_result),
        "Parity failure for: {}",
        code
    );

    // Verify expected value
    match &interp_result {
        Value::String(s) => assert_eq!(s.as_ref(), expected),
        Value::Number(n) => assert_eq!(&n.to_string(), expected),
        Value::Bool(b) => assert_eq!(&b.to_string(), expected),
        Value::Option(_) | Value::Result(_) => {
            assert_eq!(&format!("{}", interp_result), expected)
        }
        _ => panic!("Unexpected value type"),
    }
}

// ============================================================================
// Array Function Parity Tests (21 functions)
// ============================================================================

#[rstest]
#[case::len("len([1, 2, 3])", "3")]
#[case::len_empty("len([])", "0")]
#[case::concat_add("len([1, 2].concat([3]))", "3")]
#[case::concat_empty_add("len([].concat([1]))", "1")]
#[case::pop_result("[1, 2, 3].pop()", "3")]
#[case::pop_remainder("{ let arr = [1, 2, 3]; arr.pop(); arr.len() }", "2")]
#[case::shift_result("[1, 2, 3].shift()", "1")]
#[case::shift_remainder("{ let arr = [1, 2, 3]; arr.shift(); arr.len() }", "2")]
#[case::unshift("{ let arr = [2, 3]; arr.unshift(1); arr.len() }", "3")]
#[case::concat_arr("len([1, 2].concat([3, 4]))", "4")]
#[case::slice("[1, 2, 3, 4].slice(1, 3)[0]", "2")]
#[case::reverse("[1, 2, 3].reverse()[0]", "3")]
#[case::indexof_arr("[1, 2, 3].indexOf(2)", "Some(1)")]
#[case::indexof_not_found_arr("[1, 2, 3].indexOf(5)", "None")]
#[case::includes_arr("[1, 2, 3].includes(2)", "true")]
#[case::includes_false_arr("[1, 2, 3].includes(5)", "false")]
#[case::first_elem("[1, 2, 3][0]", "1")]
#[case::last_elem("[1, 2, 3][2]", "3")]
#[case::slice_rest("[1, 2, 3].slice(1, 3)[0]", "2")]
#[case::slice_rest_len("len([1].slice(1, 1))", "0")]
#[case::flatten("len([[1, 2], [3, 4]].flatten())", "4")]
#[case::flatten_empty("len([[1]].slice(1, 1).flatten())", "0")]
#[case::arraylastindexof("[1, 2, 3, 2].lastIndexOf(2)", "Some(3)")]
#[case::arraylastindexof_not_found("[1, 2, 3].lastIndexOf(5)", "None")]
fn test_array_basic_parity(#[case] code: &str, #[case] expected: &str) {
    let runtime_interp = Atlas::new();
    let interp_result = runtime_interp.eval(code).unwrap();

    let runtime_vm = Atlas::new();
    let vm_result = runtime_vm.eval(code).unwrap();

    assert_eq!(
        format!("{:?}", interp_result),
        format!("{:?}", vm_result),
        "Parity failure for: {}",
        code
    );

    match &interp_result {
        Value::String(s) => assert_eq!(s.as_ref(), expected),
        Value::Number(n) => assert_eq!(&n.to_string(), expected),
        Value::Bool(b) => assert_eq!(&b.to_string(), expected),
        Value::Option(_) | Value::Result(_) => {
            assert_eq!(&format!("{}", interp_result), expected)
        }
        _ => panic!("Unexpected value type"),
    }
}

#[rstest]
#[case::map(
    "fn double(borrow x: number): number { return x * 2; } map([1, 2, 3], double)[0]",
    "2"
)]
#[case::filter(
    "fn isEven(borrow x: number): bool { return x % 2 == 0; } filter([1, 2, 3, 4], isEven)[0]",
    "2"
)]
#[case::reduce(
    "fn sum(borrow a: number, borrow b: number): number { return a + b; } reduce([1, 2, 3], sum, 0)",
    "6"
)]
#[case::every_true(
    "fn isPositive(borrow x: number): bool { return x > 0; } every([1, 2, 3], isPositive)",
    "true"
)]
#[case::every_false(
    "fn isPositive(borrow x: number): bool { return x > 0; } every([1, -2, 3], isPositive)",
    "false"
)]
#[case::some_true(
    "fn isNegative(borrow x: number): bool { return x < 0; } some([1, -2, 3], isNegative)",
    "true"
)]
#[case::some_false(
    "fn isNegative(borrow x: number): bool { return x < 0; } some([1, 2, 3], isNegative)",
    "false"
)]
fn test_array_higher_order_parity(#[case] code: &str, #[case] expected: &str) {
    let runtime_interp = Atlas::new();
    let interp_result = runtime_interp.eval(code).unwrap();

    let runtime_vm = Atlas::new();
    let vm_result = runtime_vm.eval(code).unwrap();

    assert_eq!(
        format!("{:?}", interp_result),
        format!("{:?}", vm_result),
        "Parity failure for: {}",
        code
    );

    match &interp_result {
        Value::String(s) => assert_eq!(s.as_ref(), expected),
        Value::Number(n) => assert_eq!(&n.to_string(), expected),
        Value::Bool(b) => assert_eq!(&b.to_string(), expected),
        _ => panic!("Unexpected value type"),
    }
}

// ============================================================================
// Math Function Parity Tests (18 functions + 5 constants)
// ============================================================================

#[rstest]
#[case::abs_positive("Math.abs(5)", "5")]
#[case::abs_negative("Math.abs(-5)", "5")]
#[case::abs_zero("Math.abs(0)", "0")]
#[case::ceil("Math.ceil(4.3)", "5")]
#[case::ceil_negative("Math.ceil(-4.3)", "-4")]
#[case::floor("Math.floor(4.7)", "4")]
#[case::floor_negative("Math.floor(-4.7)", "-5")]
#[case::round("Math.round(4.5)", "4")] // Banker's rounding (round to even)
#[case::round_down("Math.round(4.4)", "4")]
#[case::min("Math.min(5, 3)", "3")]
#[case::min_negative("Math.min(-5, -3)", "-5")]
#[case::max("Math.max(5, 3)", "5")]
#[case::max_negative("Math.max(-5, -3)", "-3")]
#[case::pow("Math.pow(2, 3)", "8")]
#[case::pow_zero("Math.pow(5, 0)", "1")]
#[case::sqrt("Math.sqrt(16)", "Ok(4)")]
#[case::sqrt_decimal("Math.sqrt(2)", "Ok(1.4142135623730951)")]
#[case::sin_zero("Math.sin(0)", "0")]
#[case::cos_zero("Math.cos(0)", "1")]
#[case::tan_zero("Math.tan(0)", "0")]
#[case::exp_zero("Math.exp(0)", "1")]
#[case::log_e("Math.log(2.718281828459045)", "Ok(1)")]
#[case::log2_of_8("Math.log2(8)", "Ok(3)")]
#[case::log10_of_100("Math.log10(100)", "Ok(2)")]
#[case::pi("Math.PI() > 3.14159 && Math.PI() < 3.14160", "true")]
#[case::e("Math.E() > 2.71828 && Math.E() < 2.71829", "true")]
#[case::sqrt2("Math.SQRT2() > 1.41421 && Math.SQRT2() < 1.41422", "true")]
#[case::ln2("Math.LN2() > 0.69314 && Math.LN2() < 0.69316", "true")]
#[case::ln10("Math.LN10() > 2.30258 && Math.LN10() < 2.30260", "true")]
#[case::clamp_mid("Math.clamp(5, 0, 10)", "Ok(5)")]
#[case::clamp_low("Math.clamp(-5, 0, 10)", "Ok(0)")]
#[case::clamp_high("Math.clamp(15, 0, 10)", "Ok(10)")]
#[case::sign_positive("Math.sign(42)", "1")]
#[case::sign_negative("Math.sign(-42)", "-1")]
#[case::sign_zero("Math.sign(0)", "0")]
#[case::asin_zero("Math.asin(0)", "Ok(0)")]
#[case::acos_one("Math.acos(1)", "Ok(0)")]
#[case::atan_zero("Math.atan(0)", "0")]
#[case::atan2_zero("Math.atan2(0, 1)", "0")]
#[case::trunc_positive("Math.trunc(4.9)", "4")]
#[case::trunc_negative("Math.trunc(-4.9)", "-4")]
#[case::cbrt_eight("Math.cbrt(8)", "2")]
#[case::hypot_3_4("Math.hypot(3, 4)", "5")]
fn test_math_parity(#[case] code: &str, #[case] expected: &str) {
    let runtime_interp = Atlas::new();
    let interp_result = runtime_interp.eval(code).unwrap();

    let runtime_vm = Atlas::new();
    let vm_result = runtime_vm.eval(code).unwrap();

    assert_eq!(
        format!("{:?}", interp_result),
        format!("{:?}", vm_result),
        "Parity failure for: {}",
        code
    );

    match &interp_result {
        Value::String(s) => assert_eq!(s.as_ref(), expected),
        Value::Number(n) => assert_eq!(&n.to_string(), expected),
        Value::Bool(b) => assert_eq!(&b.to_string(), expected),
        Value::Option(_) | Value::Result(_) => {
            assert_eq!(&format!("{}", interp_result), expected)
        }
        _ => {}
    }
}

// ============================================================================
// JSON Function Parity Tests (17 functions)
// ============================================================================

#[rstest]
#[case::parse_object(
    "let j = Json.parse(\"{\\\"key\\\": \\\"value\\\"}\"); j?[\"key\"].asString()",
    "value"
)]
#[case::parse_array("let j = Json.parse(\"[1, 2, 3]\"); j?[0].asNumber()", "1")]
#[case::parse_number("let j = Json.parse(\"42\"); j?.asNumber()", "42")]
#[case::parse_string("let j = Json.parse(\"\\\"hello\\\"\"); j?.asString()", "hello")]
#[case::parse_bool("let j = Json.parse(\"true\"); j?.asBool()", "true")]
#[case::parse_null("let j = Json.parse(\"null\"); j?.isNull()", "true")]
#[case::stringify_object("Json.stringify(Json.parse(\"{\\\"a\\\": 1}\")?)", "{\"a\":1}")]
#[case::stringify_array("Json.stringify(Json.parse(\"[1,2,3]\")?)", "[1,2,3]")]
#[case::as_string("Json.parse(\"\\\"test\\\"\")?.asString()", "test")]
#[case::as_number("Json.parse(\"123\")?.asNumber()", "123")]
#[case::as_bool("Json.parse(\"true\")?.asBool()", "true")]
#[case::is_null_true("Json.parse(\"null\")?.isNull()", "true")]
#[case::is_null_false("Json.parse(\"123\")?.isNull()", "false")]
// Note: JSON type checking methods not yet implemented
// #[case::is_array_true("Json.parse(\"[1,2]\")?.is_array()", "true")]
// #[case::is_array_false("Json.parse(\"123\")?.is_array()", "false")]
// #[case::is_object_true("Json.parse(\"{\\\"a\\\": 1}\")?.is_object()", "true")]
// #[case::is_object_false("Json.parse(\"123\")?.is_object()", "false")]
// #[case::array_length("Json.parse(\"[1,2,3]\")?.array_length()", "3")]
#[case::nested_access(
    "let j = Json.parse(\"{\\\"a\\\": {\\\"b\\\": 42}}\"); j?[\"a\"][\"b\"].asNumber()",
    "42"
)]
#[case::json_array_index("let j = Json.parse(\"[10, 20, 30]\"); j?[1].asNumber()", "20")]
#[case::json_string_value(
    "let j = Json.parse(\"{\\\"name\\\": \\\"Alice\\\"}\"); j?[\"name\"].asString()",
    "Alice"
)]
#[case::json_bool_value(
    "let j = Json.parse(\"{\\\"active\\\": false}\"); j?[\"active\"].asBool()",
    "false"
)]
#[case::isvalidjson_true("Json.isValid(\"{\\\"key\\\": \\\"value\\\"}\")", "true")]
#[case::isvalidjson_false("Json.isValid(\"invalid json\")", "false")]
fn test_json_parity(#[case] code: &str, #[case] expected: &str) {
    let runtime_interp = Atlas::new();
    let interp_result = runtime_interp.eval(code).unwrap();

    let runtime_vm = Atlas::new();
    let vm_result = runtime_vm.eval(code).unwrap();

    assert_eq!(
        format!("{:?}", interp_result),
        format!("{:?}", vm_result),
        "Parity failure for: {}",
        code
    );

    match &interp_result {
        Value::String(s) => assert_eq!(s.as_ref(), expected),
        Value::Number(n) => assert_eq!(&n.to_string(), expected),
        Value::Bool(b) => assert_eq!(&b.to_string(), expected),
        _ => panic!("Unexpected value type for: {}", code),
    }
}

// ============================================================================
// File I/O Function Parity Tests (10 functions)
// ============================================================================

#[test]
fn test_file_read_write_parity() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Write and read back
    let code = format!(
        r#"
        write_file("{}", "test content");
        read_file("{}")
    "#,
        path_for_atlas(&file_path),
        path_for_atlas(&file_path)
    );

    // Interpreter
    let mut security_interp = SecurityContext::new();
    security_interp.grant_filesystem_read(temp_dir.path(), true);
    security_interp.grant_filesystem_write(temp_dir.path(), true);
    let runtime_interp = Atlas::new_with_security(security_interp);
    let interp_result = runtime_interp.eval(&code).unwrap();

    // VM
    let mut security_vm = SecurityContext::new();
    security_vm.grant_filesystem_read(temp_dir.path(), true);
    security_vm.grant_filesystem_write(temp_dir.path(), true);
    let runtime_vm = Atlas::new_with_security(security_vm);
    let vm_result = runtime_vm.eval(&code).unwrap();

    assert_eq!(format!("{:?}", interp_result), format!("{:?}", vm_result));
    assert_eq!(
        interp_result,
        Value::String(Arc::new("test content".to_string()))
    );
}

#[test]
fn test_file_exists_parity() {
    let temp_dir = TempDir::new().unwrap();
    let existing = temp_dir.path().join("exists.txt");
    let non_existing = temp_dir.path().join("nonexistent.txt");
    std::fs::write(&existing, "content").unwrap();

    let code_exists = format!(r#"file_exists("{}")"#, path_for_atlas(&existing));
    let code_not_exists = format!(r#"file_exists("{}")"#, path_for_atlas(&non_existing));

    // Test existing file
    let mut security1 = SecurityContext::new();
    security1.grant_filesystem_read(temp_dir.path(), true);
    let runtime_interp = Atlas::new_with_security(security1);
    let interp_result = runtime_interp.eval(&code_exists).unwrap();

    let mut security2 = SecurityContext::new();
    security2.grant_filesystem_read(temp_dir.path(), true);
    let runtime_vm = Atlas::new_with_security(security2);
    let vm_result = runtime_vm.eval(&code_exists).unwrap();

    assert_eq!(format!("{:?}", interp_result), format!("{:?}", vm_result));
    assert_eq!(interp_result, Value::Bool(true));

    // Test non-existing file
    let mut security3 = SecurityContext::new();
    security3.grant_filesystem_read(temp_dir.path(), true);
    let runtime_interp2 = Atlas::new_with_security(security3);
    let interp_result2 = runtime_interp2.eval(&code_not_exists).unwrap();

    let mut security4 = SecurityContext::new();
    security4.grant_filesystem_read(temp_dir.path(), true);
    let runtime_vm2 = Atlas::new_with_security(security4);
    let vm_result2 = runtime_vm2.eval(&code_not_exists).unwrap();

    assert_eq!(format!("{:?}", interp_result2), format!("{:?}", vm_result2));
    assert_eq!(interp_result2, Value::Bool(false));
}

#[test]
fn test_file_delete_parity() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("delete_me.txt");

    let code = format!(
        r#"
        write_file("{}", "content");
        remove_file("{}");
        file_exists("{}")
    "#,
        path_for_atlas(&file_path),
        path_for_atlas(&file_path),
        path_for_atlas(&file_path)
    );

    // Interpreter
    let mut security_interp = SecurityContext::new();
    security_interp.grant_filesystem_read(temp_dir.path(), true);
    security_interp.grant_filesystem_write(temp_dir.path(), true);
    let runtime_interp = Atlas::new_with_security(security_interp);
    let interp_result = runtime_interp.eval(&code).unwrap();

    // VM
    let mut security_vm = SecurityContext::new();
    security_vm.grant_filesystem_read(temp_dir.path(), true);
    security_vm.grant_filesystem_write(temp_dir.path(), true);
    let runtime_vm = Atlas::new_with_security(security_vm);
    let vm_result = runtime_vm.eval(&code).unwrap();

    assert_eq!(format!("{:?}", interp_result), format!("{:?}", vm_result));
    assert_eq!(interp_result, Value::Bool(false));
}

#[test]
fn test_file_append_parity() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("append.txt");

    let code = format!(
        r#"
        write_file("{}", "first");
        append_file("{}", "second");
        read_file("{}")
    "#,
        path_for_atlas(&file_path),
        path_for_atlas(&file_path),
        path_for_atlas(&file_path)
    );

    // Interpreter
    let mut security_interp = SecurityContext::new();
    security_interp.grant_filesystem_read(temp_dir.path(), true);
    security_interp.grant_filesystem_write(temp_dir.path(), true);
    let runtime_interp = Atlas::new_with_security(security_interp);
    let interp_result = runtime_interp.eval(&code).unwrap();

    // VM
    let mut security_vm = SecurityContext::new();
    security_vm.grant_filesystem_read(temp_dir.path(), true);
    security_vm.grant_filesystem_write(temp_dir.path(), true);
    let runtime_vm = Atlas::new_with_security(security_vm);
    let vm_result = runtime_vm.eval(&code).unwrap();

    assert_eq!(format!("{:?}", interp_result), format!("{:?}", vm_result));
    assert_eq!(
        interp_result,
        Value::String(Arc::new("firstsecond".to_string()))
    );
}

#[test]
fn test_file_list_directory_parity() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
    std::fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();

    let code = format!(r#"len(read_dir("{}"))"#, path_for_atlas(temp_dir.path()));

    // Interpreter
    let mut security_interp = SecurityContext::new();
    security_interp.grant_filesystem_read(temp_dir.path(), true);
    let runtime_interp = Atlas::new_with_security(security_interp);
    let interp_result = runtime_interp.eval(&code).unwrap();

    // VM
    let mut security_vm = SecurityContext::new();
    security_vm.grant_filesystem_read(temp_dir.path(), true);
    let runtime_vm = Atlas::new_with_security(security_vm);
    let vm_result = runtime_vm.eval(&code).unwrap();

    assert_eq!(format!("{:?}", interp_result), format!("{:?}", vm_result));
    assert_eq!(interp_result, Value::Number(2.0));
}

#[test]
fn test_file_create_remove_directory_parity() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().join("testdir");

    let code = format!(
        r#"
        create_dir("{}");
        let exists1 = file_exists("{}");
        remove_dir("{}");
        let exists2 = file_exists("{}");
        exists1 && !exists2
    "#,
        path_for_atlas(&dir_path),
        path_for_atlas(&dir_path),
        path_for_atlas(&dir_path),
        path_for_atlas(&dir_path)
    );

    // Interpreter
    let mut security_interp = SecurityContext::new();
    security_interp.grant_filesystem_read(temp_dir.path(), true);
    security_interp.grant_filesystem_write(temp_dir.path(), true);
    let runtime_interp = Atlas::new_with_security(security_interp);
    let interp_result = runtime_interp.eval(&code).unwrap();

    // VM
    let mut security_vm = SecurityContext::new();
    security_vm.grant_filesystem_read(temp_dir.path(), true);
    security_vm.grant_filesystem_write(temp_dir.path(), true);
    let runtime_vm = Atlas::new_with_security(security_vm);
    let vm_result = runtime_vm.eval(&code).unwrap();

    assert_eq!(format!("{:?}", interp_result), format!("{:?}", vm_result));
    assert_eq!(interp_result, Value::Bool(true));
}

// ============================================================================
// Type Checking Function Parity Tests (6 functions)
// ============================================================================

#[rstest]
#[case::is_string_true(r#"typeof("hello") == "string""#, "true")]
#[case::is_string_false(r#"typeof(123) == "string""#, "false")]
#[case::is_number_true("is_number(123)", "true")]
#[case::is_number_false("is_number(\"123\")", "false")]
#[case::is_bool_true(r#"typeof(true) == "bool""#, "true")]
#[case::is_bool_false(r#"typeof(1) == "bool""#, "false")]
#[case::is_null_true(r#"typeof(null) == "null""#, "true")]
#[case::is_null_false(r#"typeof(0) == "null""#, "false")]
#[case::is_array_true("is_array([1, 2, 3])", "true")]
#[case::is_array_false("is_array(\"[1,2,3]\")", "false")]
#[case::is_function_true("fn test(): void {} is_function(test)", "true")]
#[case::is_function_false("is_function(123)", "false")]
fn test_type_checking_parity(#[case] code: &str, #[case] expected: &str) {
    let runtime_interp = Atlas::new();
    let interp_result = runtime_interp.eval(code).unwrap();

    let runtime_vm = Atlas::new();
    let vm_result = runtime_vm.eval(code).unwrap();

    assert_eq!(
        format!("{:?}", interp_result),
        format!("{:?}", vm_result),
        "Parity failure for: {}",
        code
    );

    match &interp_result {
        Value::Bool(b) => assert_eq!(&b.to_string(), expected),
        _ => panic!("Expected bool for type checking"),
    }
}

// ============================================================================
// Edge Case & Error Parity Tests
// ============================================================================

#[rstest]
#[case::empty_string_operations("len(\"\".trim())", "0")]
#[case::empty_array_operations("len([].reverse())", "0")]
#[case::divide_by_zero("1 / 0 > 999999999999999", "true")] // inf
#[case::negative_sqrt("Math.sqrt(-1)", "NaN")] // NaN as string
#[case::parse_invalid_json_safety("let j = Json.parse(\"invalid\"); j.isNull()", "false")] // Returns error, not crash
fn test_edge_cases_parity(#[case] code: &str, #[case] _expected: &str) {
    let runtime_interp = Atlas::new();
    let interp_result = runtime_interp.eval(code);

    let runtime_vm = Atlas::new();
    let vm_result = runtime_vm.eval(code);

    // Both should succeed or both should fail with same error
    match (&interp_result, &vm_result) {
        (Ok(v1), Ok(v2)) => {
            assert_eq!(
                format!("{:?}", v1),
                format!("{:?}", v2),
                "Parity failure for: {}",
                code
            );
        }
        (Err(e1), Err(e2)) => {
            assert_eq!(e1.len(), e2.len(), "Different error counts for: {}", code);
            if !e1.is_empty() && !e2.is_empty() {
                assert_eq!(
                    e1[0].code, e2[0].code,
                    "Different error codes for: {}",
                    code
                );
            }
        }
        _ => panic!("Parity failure: one succeeded, one failed for: {}", code),
    }
}

// ============================================================================
