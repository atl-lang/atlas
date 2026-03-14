use super::{vm_eval, Value};
use pretty_assertions::assert_eq;

#[test]
fn test_vm_for_loop() {
    assert_eq!(
        vm_eval("let mut sum = 0; for i in [0, 1, 2, 3, 4] { sum = sum + i; } sum;"),
        Some(Value::Number(10.0))
    );
}

/// H-303: tail_expr in while/for-in/if blocks was being ignored.
/// The parser puts the last expression in `block.tail_expr`, but `compile_block`
/// only compiled `block.statements`. This test ensures tail expressions execute.
#[test]
fn test_h303_tail_expr_in_while_body() {
    // Block expression as last statement in while body must execute
    let code = r#"
        let mut count = 0;
        let mut i = 0;
        while i < 3 {
            i = i + 1;
            { count = count + 1; }
        }
        count;
    "#;
    assert_eq!(vm_eval(code), Some(Value::Number(3.0)));
}

#[test]
fn test_h303_match_as_last_statement_in_while() {
    // Match expression as last statement in while body must execute arms
    let code = r#"
        let mut found = false;
        let mut i = 0;
        while i < 3 {
            i = i + 1;
            match i {
                2 => { found = true; },
                _ => {}
            }
        }
        found;
    "#;
    assert_eq!(vm_eval(code), Some(Value::Bool(true)));
}

#[test]
fn test_h303_tail_expr_in_for_body() {
    // Tail expression in for-in body must execute
    let code = r#"
        let mut sum = 0;
        for x in [1, 2, 3] {
            { sum = sum + x; }
        }
        sum;
    "#;
    assert_eq!(vm_eval(code), Some(Value::Number(6.0)));
}

#[test]
fn test_h303_tail_expr_in_if_then() {
    // Tail expression in if-then block must execute
    let code = r#"
        let mut result = 0;
        if true {
            { result = 42; }
        }
        result;
    "#;
    assert_eq!(vm_eval(code), Some(Value::Number(42.0)));
}

#[test]
fn test_h303_tail_expr_in_if_else() {
    // Tail expression in else block must execute
    let code = r#"
        let mut result = 0;
        if false {
            result = 1;
        } else {
            { result = 99; }
        }
        result;
    "#;
    assert_eq!(vm_eval(code), Some(Value::Number(99.0)));
}

/// H-407: `continue` inside a match arm within a for loop skips remaining iterations.
/// Root cause: compile_continue jumped back to loop increment without popping
/// body locals (e.g. the `opt` variable and the $match_scrutinee phantom slot).
#[test]
fn test_h407_continue_in_match_arm_for_loop() {
    // None => continue: scrutinee consumed by pattern check ($match_scrutinee is phantom)
    let code = r#"
        let items: string[] = ["a", "b", "c"];
        let mut found = "";
        for item in items {
            let opt: Option<string> = if item == "b" { Some(item) } else { None };
            match opt {
                Some(v) => { found = v; },
                None => continue,
            }
        }
        found;
    "#;
    assert_eq!(vm_eval(code), Some(Value::string("b")));
}

/// H-407 variant: continue in Some arm (pattern var present)
#[test]
fn test_h407_continue_in_some_arm() {
    let code = r#"
        let mut count = 0;
        for i in [1, 2, 3, 4, 5] {
            let opt: Option<number> = if i % 2 == 0 { Some(i) } else { None };
            match opt {
                None => { count = count + 10; },
                Some(_) => continue,
            }
        }
        count;
    "#;
    // Odd numbers (1,3,5) hit None arm: count += 10 each = 30
    assert_eq!(vm_eval(code), Some(Value::Number(30.0)));
}
