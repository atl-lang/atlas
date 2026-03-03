use super::*;

#[test]
fn test_if_then() {
    let code = r#"
        let mut x: number = 0;
        if (true) {
            x = 42;
        }
        x
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_if_else() {
    let code = r#"
        let mut x: number = 0;
        if (false) {
            x = 10;
        } else {
            x = 20;
        }
        x
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn test_if_with_comparison() {
    let code = r#"
        let x: number = 5;
        let mut result: number = 0;
        if (x > 3) {
            result = 1;
        } else {
            result = 2;
        }
        result
    "#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_while_loop() {
    let code = r#"
        let mut i: number = 0;
        let mut sum: number = 0;
        while (i < 5) {
            sum = sum + i;
            i = i + 1;
        }
        sum
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_while_loop_with_break() {
    let code = r#"
        let mut i: number = 0;
        while (i < 10) {
            if (i == 5) {
                break;
            }
            i = i + 1;
        }
        i
    "#;
    assert_eval_number(code, 5.0);
}

#[test]
fn test_while_loop_with_continue() {
    let code = r#"
        let mut i: number = 0;
        let mut sum: number = 0;
        while (i < 5) {
            i = i + 1;
            if (i == 3) {
                continue;
            }
            sum = sum + i;
        }
        sum
    "#;
    assert_eval_number(code, 12.0);
}

#[test]
fn test_for_loop() {
    let code = r#"
        let mut sum: number = 0;
        for i in [0, 1, 2, 3, 4] {
            sum = sum + i;
        }
        sum
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_for_loop_with_break() {
    let code = r#"
        let mut result: number = 0;
        for i in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] {
            if (i == 5) {
                break;
            }
            result = i;
        }
        result
    "#;
    assert_eval_number(code, 4.0);
}

#[test]
fn test_for_loop_with_continue() {
    let code = r#"
        let mut sum: number = 0;
        for i in [0, 1, 2, 3, 4] {
            if (i == 2) {
                continue;
            }
            sum = sum + i;
        }
        sum
    "#;
    assert_eval_number(code, 8.0);
}

#[test]
fn test_for_loop_with_increment() {
    let code = r#"
        let mut sum: number = 0;
        for i in [0, 1, 2, 3, 4] {
            sum += i;
        }
        sum
    "#;
    assert_eval_number(code, 10.0);
}

// ============================================================================
