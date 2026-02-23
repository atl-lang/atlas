use super::*;
use pretty_assertions::assert_eq;

// From vm_regression_tests.rs
// ============================================================================

// VM Regression Tests
//
// Ensures zero regressions from v0.1, maintains interpreter-VM parity,
// and validates edge cases across all VM features.

// ============================================================================
// Helpers
// ============================================================================

// ============================================================================
// 1. Interpreter-VM Parity (tests 1-25)
// ============================================================================

#[rstest]
#[case("1 + 2;")]
#[case("10 - 3;")]
#[case("4 * 5;")]
#[case("15 / 3;")]
#[case("17 % 5;")]
fn test_parity_arithmetic(#[case] source: &str) {
    assert_parity(source);
}

#[rstest]
#[case("var x = 10; x;")]
#[case("let x = 5; let y = 3; x + y;")]
#[case("var x = 10; x = 20; x;")]
#[case("var x = 1; x = x + 1; x = x + 1; x;")]
fn test_parity_variables(#[case] source: &str) {
    assert_parity(source);
}

#[rstest]
#[case("true;")]
#[case("false;")]
#[case("!true;")]
#[case("true && false;")]
#[case("true || false;")]
fn test_parity_booleans(#[case] source: &str) {
    assert_parity(source);
}

#[rstest]
#[case("1 < 2;")]
#[case("2 > 1;")]
#[case("1 <= 1;")]
#[case("2 >= 3;")]
#[case("1 == 1;")]
#[case("1 != 2;")]
fn test_parity_comparisons(#[case] source: &str) {
    assert_parity(source);
}

#[test]
fn test_parity_string_concat() {
    assert_parity(r#""hello" + " " + "world";"#);
}

#[test]
fn test_parity_null() {
    assert_parity("null;");
}

#[test]
fn test_parity_array_creation() {
    assert_parity("let arr = [1, 2, 3]; arr[0];");
}

#[test]
fn test_parity_array_mutation() {
    assert_parity("let arr = [1, 2, 3]; arr[0] = 10; arr[0];");
}

#[test]
fn test_parity_function_call() {
    assert_parity("fn add(a: number, b: number) -> number { return a + b; } add(3, 4);");
}

#[test]
fn test_parity_recursion() {
    assert_parity("fn fact(n: number) -> number { if (n <= 1) { return 1; } return n * fact(n - 1); } fact(5);");
}

#[test]
fn test_parity_while_loop() {
    assert_parity("var sum = 0; var i = 0; while (i < 10) { sum = sum + i; i = i + 1; } sum;");
}

#[test]
fn test_parity_nested_if() {
    assert_parity(
        "let x = 15; var r = 0; if (x > 10) { if (x > 20) { r = 2; } else { r = 1; } } r;",
    );
}

#[test]
fn test_parity_negative_numbers() {
    assert_parity("let x = -5; -x;");
}

#[test]
fn test_parity_complex_expression() {
    assert_parity("let a = 2; let b = 3; let c = 4; (a + b) * c;");
}

// ============================================================================
// 2. Edge Cases (tests 26-45)
// ============================================================================

#[test]
fn test_edge_zero_division() {
    // VM raises DivideByZero error
    let source = "1 / 0;";
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).expect("Compilation failed");
    let mut vm = VM::new(bytecode);
    let result = vm.run(&SecurityContext::allow_all());
    assert!(result.is_err(), "Expected DivideByZero error");
}

#[test]
fn test_edge_negative_modulo() {
    let result = vm_number("-7 % 3;");
    assert_eq!(result, -1.0);
}

#[test]
fn test_edge_large_number() {
    let result = vm_number("999999999 + 1;");
    assert_eq!(result, 1000000000.0);
}

#[test]
fn test_edge_small_float() {
    let result = vm_number("0.1 + 0.2;");
    assert!((result - 0.3).abs() < 0.0001);
}

#[test]
fn test_edge_empty_array() {
    let source = "let arr = []; arr;";
    let result = vm_eval(source);
    assert!(result.is_some());
}

#[test]
fn test_edge_single_element_array() {
    let result = vm_number("let arr = [42]; arr[0];");
    assert_eq!(result, 42.0);
}

#[test]
fn test_edge_boolean_as_condition() {
    let result = vm_number("var x = 0; if (true) { x = 1; } x;");
    assert_eq!(result, 1.0);
}

#[test]
fn test_edge_while_false() {
    let result = vm_number("var x = 42; while (false) { x = 0; } x;");
    assert_eq!(result, 42.0);
}

#[test]
fn test_edge_nested_function_scope() {
    let source = r#"
fn outer() -> number {
    var x = 10;
    fn inner() -> number {
        return 20;
    }
    return x + inner();
}
outer();
"#;
    assert_eq!(vm_number(source), 30.0);
}

#[test]
fn test_edge_function_no_return() {
    let source = "fn noop() { let x = 1; } noop();";
    let result = vm_eval(source);
    // Should return null/none
    assert!(result.is_none() || result == Some(Value::Null));
}

#[test]
fn test_edge_multiple_assignments() {
    let result = vm_number("var x = 1; x = 2; x = 3; x = 4; x = 5; x;");
    assert_eq!(result, 5.0);
}

#[test]
fn test_edge_deeply_nested_arithmetic() {
    let result = vm_number("((((((1 + 2) + 3) + 4) + 5) + 6) + 7);");
    assert_eq!(result, 28.0);
}

#[test]
fn test_edge_string_equality() {
    assert!(vm_bool(r#""hello" == "hello";"#));
}

#[test]
fn test_edge_string_inequality() {
    assert!(vm_bool(r#""hello" != "world";"#));
}

#[test]
fn test_edge_number_equality() {
    assert!(vm_bool("42 == 42;"));
}

#[test]
fn test_edge_bool_equality() {
    assert!(vm_bool("true == true;"));
}

#[test]
fn test_edge_null_equality() {
    assert!(vm_bool("null == null;"));
}

#[test]
fn test_edge_compound_assignment_add() {
    let result = vm_number("var x = 10; x += 5; x;");
    assert_eq!(result, 15.0);
}

#[test]
fn test_edge_compound_assignment_sub() {
    let result = vm_number("var x = 10; x -= 3; x;");
    assert_eq!(result, 7.0);
}

#[test]
fn test_edge_compound_assignment_mul() {
    let result = vm_number("var x = 4; x *= 3; x;");
    assert_eq!(result, 12.0);
}

// ============================================================================
// 3. V0.1 Programs (tests 46-55)
// ============================================================================

#[test]
fn test_v01_basic_let() {
    assert_eq!(vm_number("let x = 42; x;"), 42.0);
}

#[test]
fn test_v01_basic_arithmetic() {
    assert_eq!(vm_number("2 + 3 * 4;"), 14.0);
}

#[test]
fn test_v01_string_literal() {
    assert_eq!(vm_string(r#""hello world";"#), "hello world");
}

#[test]
fn test_v01_if_else() {
    assert_eq!(
        vm_number("var x = 10; var r = 0; if (x > 5) { r = 1; } else { r = 0; } r;"),
        1.0
    );
}

#[test]
fn test_v01_while_loop() {
    assert_eq!(
        vm_number("var i = 0; while (i < 10) { i = i + 1; } i;"),
        10.0
    );
}

#[test]
fn test_v01_function_definition() {
    assert_eq!(
        vm_number("fn greet(n: number) -> number { return n * 2; } greet(21);"),
        42.0
    );
}

#[test]
fn test_v01_array_operations() {
    assert_eq!(vm_number("let arr = [1, 2, 3]; arr[1];"), 2.0);
}

#[test]
fn test_v01_boolean_operations() {
    assert!(vm_bool("true && true;"));
    assert!(!vm_bool("true && false;"));
    assert!(vm_bool("true || false;"));
}

#[test]
fn test_v01_comparison_operators() {
    assert!(vm_bool("1 < 2;"));
    assert!(vm_bool("2 > 1;"));
    assert!(vm_bool("1 <= 1;"));
    assert!(vm_bool("1 >= 1;"));
    assert!(vm_bool("1 == 1;"));
    assert!(vm_bool("1 != 2;"));
}

#[test]
fn test_v01_nested_functions() {
    let source = r#"
fn outer(x: number) -> number {
    fn inner(y: number) -> number {
        return y + 1;
    }
    return inner(x) * 2;
}
outer(5);
"#;
    assert_eq!(vm_number(source), 12.0);
}

// ============================================================================
// 4. Performance Regression (tests 56-65)
// ============================================================================

#[test]
fn test_perf_large_loop() {
    let start = std::time::Instant::now();
    let result =
        vm_number("var sum = 0; var i = 0; while (i < 100000) { sum = sum + i; i = i + 1; } sum;");
    let elapsed = start.elapsed();
    assert_eq!(result, 4999950000.0);
    assert!(elapsed.as_secs() < 10, "Large loop too slow: {:?}", elapsed);
}

#[test]
fn test_perf_recursive_fib() {
    let start = std::time::Instant::now();
    let result = vm_number("fn fib(n: number) -> number { if (n <= 1) { return n; } return fib(n - 1) + fib(n - 2); } fib(20);");
    let elapsed = start.elapsed();
    assert_eq!(result, 6765.0);
    assert!(
        elapsed.as_secs() < 10,
        "Recursive fib too slow: {:?}",
        elapsed
    );
}

#[test]
fn test_perf_nested_loops() {
    let start = std::time::Instant::now();
    let result = vm_number("var c = 0; var i = 0; while (i < 100) { var j = 0; while (j < 100) { c = c + 1; j = j + 1; } i = i + 1; } c;");
    let elapsed = start.elapsed();
    assert_eq!(result, 10000.0);
    assert!(
        elapsed.as_secs() < 5,
        "Nested loops too slow: {:?}",
        elapsed
    );
}

#[test]
fn test_perf_function_calls() {
    let start = std::time::Instant::now();
    let result = vm_number("fn inc(x: number) -> number { return x + 1; } var r = 0; var i = 0; while (i < 10000) { r = inc(r); i = i + 1; } r;");
    let elapsed = start.elapsed();
    assert_eq!(result, 10000.0);
    assert!(
        elapsed.as_secs() < 5,
        "Function calls too slow: {:?}",
        elapsed
    );
}

#[test]
fn test_perf_string_concat() {
    let start = std::time::Instant::now();
    let result =
        vm_string(r#"var s = ""; var i = 0; while (i < 100) { s = s + "x"; i = i + 1; } s;"#);
    let elapsed = start.elapsed();
    assert_eq!(result.len(), 100);
    assert!(
        elapsed.as_secs() < 5,
        "String concat too slow: {:?}",
        elapsed
    );
}

#[test]
fn test_perf_array_operations() {
    let start = std::time::Instant::now();
    let source = r#"
let arr = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
var i = 0;
while (i < 1000) {
    arr[i % 10] = arr[i % 10] + 1;
    i = i + 1;
}
arr[0];
"#;
    let result = vm_number(source);
    let elapsed = start.elapsed();
    assert_eq!(result, 100.0);
    assert!(elapsed.as_secs() < 5, "Array ops too slow: {:?}", elapsed);
}

#[test]
fn test_perf_deep_recursion() {
    let start = std::time::Instant::now();
    let result = vm_number("fn sum_to(n: number) -> number { if (n <= 0) { return 0; } return n + sum_to(n - 1); } sum_to(500);");
    let elapsed = start.elapsed();
    assert_eq!(result, 125250.0);
    assert!(
        elapsed.as_secs() < 5,
        "Deep recursion too slow: {:?}",
        elapsed
    );
}

#[test]
fn test_perf_complex_computation() {
    let start = std::time::Instant::now();
    let source = r#"
fn power(b: number, e: number) -> number {
    if (e == 0) { return 1; }
    return b * power(b, e - 1);
}
var sum = 0;
var i = 1;
while (i <= 10) {
    sum = sum + power(i, 3);
    i = i + 1;
}
sum;
"#;
    let result = vm_number(source);
    let elapsed = start.elapsed();
    assert_eq!(result, 3025.0);
    assert!(
        elapsed.as_secs() < 5,
        "Complex computation too slow: {:?}",
        elapsed
    );
}

#[test]
fn test_perf_many_variables() {
    let result = vm_number(
        r#"
let a = 1; let b = 2; let c = 3; let d = 4; let e = 5;
let f = 6; let g = 7; let h = 8; let i = 9; let j = 10;
a + b + c + d + e + f + g + h + i + j;
"#,
    );
    assert_eq!(result, 55.0);
}

#[test]
fn test_perf_conditional_heavy() {
    let result = vm_number(
        r#"
var count = 0;
var i = 0;
while (i < 1000) {
    if (i % 2 == 0) { count = count + 1; }
    if (i % 3 == 0) { count = count + 1; }
    if (i % 5 == 0) { count = count + 1; }
    i = i + 1;
}
count;
"#,
    );
    // evens: 500, div3: 334, div5: 200
    assert_eq!(result, 1034.0);
}

// ============================================================================
// 5. Additional Regression (tests 66-75)
// ============================================================================

#[test]
fn test_regression_chained_comparisons() {
    let result = vm_bool("1 < 2 && 2 < 3 && 3 < 4;");
    assert!(result);
}

#[test]
fn test_regression_unary_minus_in_expression() {
    let result = vm_number("let x = 5; let y = -x + 10; y;");
    assert_eq!(result, 5.0);
}

#[test]
fn test_regression_reassignment_in_loop() {
    let result = vm_number("var x = 0; var i = 0; while (i < 5) { x = i; i = i + 1; } x;");
    assert_eq!(result, 4.0);
}

#[test]
fn test_regression_function_returning_bool() {
    assert!(vm_bool(
        "fn is_positive(x: number) -> bool { return x > 0; } is_positive(5);"
    ));
}

#[test]
fn test_regression_function_returning_string() {
    assert_eq!(
        vm_string(
            r#"fn greet(name: string) -> string { return "Hello, " + name; } greet("World");"#
        ),
        "Hello, World"
    );
}

#[test]
fn test_regression_array_in_function() {
    let result = vm_number(
        r#"
fn sum_arr() -> number {
    let arr = [1, 2, 3, 4, 5];
    var sum = 0;
    var i = 0;
    while (i < 5) {
        sum = sum + arr[i];
        i = i + 1;
    }
    return sum;
}
sum_arr();
"#,
    );
    assert_eq!(result, 15.0);
}

#[test]
fn test_regression_multiple_function_calls() {
    let result = vm_number(
        r#"
fn a() -> number { return 1; }
fn b() -> number { return 2; }
fn c() -> number { return 3; }
a() + b() + c();
"#,
    );
    assert_eq!(result, 6.0);
}

#[test]
fn test_regression_boolean_in_variable() {
    assert!(vm_bool("let x = true; let y = false; x && !y;"));
}

#[test]
fn test_regression_string_in_array() {
    let source = r#"let arr = ["a", "b", "c"]; arr[1];"#;
    assert_eq!(vm_string(source), "b");
}

#[test]
fn test_regression_mixed_types_in_scope() {
    let result = vm_number(
        r#"
let n = 42;
var s = "hello";
let b = true;
let arr = [1, 2, 3];
n + arr[0];
"#,
    );
    assert_eq!(result, 43.0);
}

// ============================================================================

