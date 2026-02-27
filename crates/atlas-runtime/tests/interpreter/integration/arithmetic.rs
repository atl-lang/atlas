use super::*;

#[rstest]
#[case("1 + 2", 3.0)]
#[case("10 - 3", 7.0)]
#[case("4 * 5", 20.0)]
#[case("20 / 4", 5.0)]
#[case("10 % 3", 1.0)]
#[case("-42", -42.0)]
#[case("2 + 3 * 4 - 1", 13.0)]
#[case("(2 + 3) * 4", 20.0)]
fn test_arithmetic_operations(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

#[rstest]
#[case("10 / 0", "AT0005")]
#[case("10 % 0", "AT0005")]
#[case("0 / 0", "AT0005")]
#[case("-10 / 0", "AT0005")]
#[case("0 % 0", "AT0005")]
#[case("5 + (10 / 0)", "AT0005")]
fn test_divide_by_zero_errors(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[rstest]
#[case("1e308 * 2.0", "AT0007")]
#[case("1.5e308 + 1.5e308", "AT0007")]
#[case("-1.5e308 - 1.5e308", "AT0007")]
#[case("1e308 / 1e-308", "AT0007")]
fn test_numeric_overflow(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[test]
fn test_numeric_valid_large_numbers() {
    let runtime = Atlas::new();
    let code = r#"
        let x: number = 1e50;
        let y: number = 2e50;
        let z: number = x + y;
        z
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => {
            assert!(n > 0.0);
            assert!(n.is_finite());
        }
        other => panic!("Expected valid large number, got {:?}", other),
    }
}

#[test]
fn test_numeric_multiplication_by_zero_valid() {
    assert_eval_number("let large: number = 1e200; large * 0", 0.0);
}

#[test]
fn test_numeric_negative_modulo() {
    let runtime = Atlas::new();
    match runtime.eval("-10 % 3") {
        Ok(Value::Number(n)) => {
            assert!(n.is_finite());
            std::assert_eq!(n, -1.0); // Rust's % preserves sign of left operand
        }
        other => panic!("Expected valid modulo result, got {:?}", other),
    }
}

#[test]
fn test_numeric_error_in_function() {
    let code = r#"
        fn compute(a: number) -> number {
            return a * a * a;
        }
        let big: number = 1e103;
        compute(big)
    "#;
    assert_error_code(code, "AT0007");
}

#[test]
fn test_numeric_error_propagation() {
    let code = r#"
        fn bad() -> number {
            return 1 / 0;
        }
        fn caller() -> number {
            return bad() + 5;
        }
        caller()
    "#;
    assert_error_code(code, "AT0005");
}

