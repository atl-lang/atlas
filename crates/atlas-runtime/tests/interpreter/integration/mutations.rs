use super::*;

#[rstest]
#[case(r#"let mut x: number = 5; x++; x"#, 6.0)]
#[case(r#"let mut x: number = 10; x--; x"#, 9.0)]
#[case(r#"let mut x: number = 0; x++; x++; x++; x"#, 3.0)]
#[case(r#"let mut x: number = 10; x--; x--; x"#, 8.0)]
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
        let mut sum: number = 0;
        let mut i: number = 0;
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
#[case("let x: number = 1; x = 2; x", "AT3003")] // Basic assignment to let
#[case("let x: number = 5; x--; x", "AT3003")] // Decrement
fn test_immutable_mutation_errors(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[rstest]
#[case("let mut x: number = 10; x += 5; x", 15.0)]
#[case("let mut x: number = 20; x -= 8; x", 12.0)]
#[case("let mut x: number = 7; x *= 3; x", 21.0)]
#[case("let mut x: number = 50; x /= 5; x", 10.0)]
#[case("let mut x: number = 17; x %= 5; x", 2.0)]
#[case("let mut x: number = 1; x = 2; x", 2.0)] // Basic assignment to var
#[case("let mut x: number = 5; x++; x", 6.0)] // Increment
#[case("let mut x: number = 5; x--; x", 4.0)] // Decrement
fn test_mutable_var_assignments(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

// ============================================================================
// let mut - Rust-style mutable bindings (recommended over deprecated `var`)
// ============================================================================

#[rstest]
#[case("let mut x: number = 10; x += 5; x", 15.0)]
#[case("let mut x: number = 20; x -= 8; x", 12.0)]
#[case("let mut x: number = 7; x *= 3; x", 21.0)]
#[case("let mut x: number = 50; x /= 5; x", 10.0)]
#[case("let mut x: number = 17; x %= 5; x", 2.0)]
#[case("let mut x: number = 1; x = 2; x", 2.0)] // Basic assignment
#[case("let mut x: number = 5; x++; x", 6.0)] // Increment
#[case("let mut x: number = 5; x--; x", 4.0)] // Decrement
fn test_mutable_let_mut_assignments(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

#[test]
fn test_let_mut_in_loop() {
    let code = r#"
        let mut sum: number = 0;
        let mut i: number = 0;
        while (i < 5) {
            sum += i;
            i++;
        }
        sum
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_let_mut_chained_operations() {
    let code = r#"
        let mut x: number = 10;
        x += 5;
        x *= 2;
        x -= 10;
        x
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn test_compound_chained() {
    let code = r#"
        let mut x: number = 10;
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
        let mut x: number = 10;
        x /= 0;
        x
    "#;
    assert_error_code(code, "AT0005");
}

// ============================================================================
// Phase interpreter-02: Interpreter-VM Parity Tests
// ============================================================================
