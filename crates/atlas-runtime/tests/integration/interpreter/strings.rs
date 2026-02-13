//! String operations and concatenation tests

use rstest::rstest;

use crate::common::*;

#[test]
fn test_string_concatenation() {
    let code = r#"
        let s: string = "Hello, " + "World!";
        s
    "#;
    assert_eval_string(code, "Hello, World!");
}

// TODO: Enable when typechecker supports string indexing
#[test]
#[ignore]
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

#[rstest]
#[case(r#"var x: number = 5; x++; x"#, 6.0)]
#[case(r#"var x: number = 10; x--; x"#, 9.0)]
#[case(r#"var x: number = 0; x++; x++; x++; x"#, 3.0)]
#[case(r#"var x: number = 10; x--; x--; x"#, 8.0)]
fn test_increment_decrement_basics(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

#[test]
fn test_increment_array_element() {
    let code = r#"
        let arr: number[] = [5, 10, 15];
        arr[0]++;
        arr[0]
    "#;
    assert_eval_number(code, 6.0);
}

#[test]
fn test_decrement_array_element() {
    let code = r#"
        let arr: number[] = [5, 10, 15];
        arr[2]--;
        arr[2]
    "#;
    assert_eval_number(code, 14.0);
}

#[test]
fn test_increment_in_loop() {
    let code = r#"
        var sum: number = 0;
        var i: number = 0;
        while (i < 5) {
            sum += i;
            i++;
        }
        sum
    "#;
    assert_eval_number(code, 10.0);
}

#[rstest]
#[case("let x: number = 5; x++; x", "AT3003")]
#[case("let x: number = 10; x += 5; x", "AT3003")]
fn test_immutable_mutation_errors(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[rstest]
#[case("var x: number = 10; x += 5; x", 15.0)]
#[case("var x: number = 20; x -= 8; x", 12.0)]
#[case("var x: number = 7; x *= 3; x", 21.0)]
#[case("var x: number = 50; x /= 5; x", 10.0)]
#[case("var x: number = 17; x %= 5; x", 2.0)]
fn test_compound_assignments(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

#[test]
fn test_compound_chained() {
    let code = r#"
        var x: number = 10;
        x += 5;
        x *= 2;
        x -= 10;
        x
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn test_compound_array_element() {
    let code = r#"
        let arr: number[] = [10, 20, 30];
        arr[1] += 5;
        arr[1]
    "#;
    assert_eval_number(code, 25.0);
}

#[test]
fn test_compound_divide_by_zero() {
    let code = r#"
        var x: number = 10;
        x /= 0;
        x
    "#;
    assert_error_code(code, "AT0005");
}
