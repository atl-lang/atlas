//! B11-P05: Typechecker enforcement of the `own` ownership contract (AT3053).
//!
//! Tests that use-after-own triggers AT3053 and that valid own usage compiles clean.

use super::{assert_has_error, assert_no_errors, errors, has_error_code, typecheck};

// ============================================================================
// Valid own usage — must compile clean
// ============================================================================

#[test]
fn test_ownership_own_valid_single_use() {
    // Passing a variable to an `own` param once is fine
    let src = r#"
fn consume(own data: string) -> string { return data; }
fn main() -> void {
    let x: string = "hello";
    let result: string = consume(x);
}
"#;
    assert_no_errors(&typecheck(src));
}

#[test]
fn test_ownership_own_valid_return_value() {
    // Returning the moved value is valid; no use-after-own in caller
    let src = r#"
fn take(own val: string) -> string { return val; }
fn main() -> void {
    let s: string = "world";
    let out: string = take(s);
    let result: string = out;
}
"#;
    assert_no_errors(&typecheck(src));
}

#[test]
fn test_ownership_own_valid_multiple_different_vars() {
    // Moving two different variables is fine
    let src = r#"
fn consume(own a: string) -> void {}
fn main() -> void {
    let x: string = "a";
    let y: string = "b";
    consume(x);
    consume(y);
}
"#;
    assert_no_errors(&typecheck(src));
}

// ============================================================================
// AT3053: use-after-own must fire
// ============================================================================

#[test]
fn test_ownership_own_use_after_own_fires() {
    // Classic use-after-own: variable used after being passed to own param
    let src = r#"
fn consume(own data: string) -> void {}
fn main() -> void {
    let x: string = "hello";
    consume(x);
    let again: string = x;
}
"#;
    let diags = errors(src);
    assert_has_error(&diags, "AT3053");
}

#[test]
fn test_ownership_own_double_move_fires() {
    // Passing the same variable to own twice must fire AT3053 on the second use
    let src = r#"
fn consume(own data: string) -> void {}
fn main() -> void {
    let x: string = "hello";
    consume(x);
    consume(x);
}
"#;
    let diags = errors(src);
    assert!(
        has_error_code(&diags, "AT3053"),
        "Expected AT3053 on second use of moved variable, got: {:?}",
        diags
    );
}

#[test]
fn test_ownership_own_use_in_expression_after_move_fires() {
    // Using moved var inside an expression (not just standalone) still triggers AT3053
    let src = r#"
fn consume(own data: string) -> void {}
fn concat(borrow a: string, borrow b: string) -> string { return a; }
fn main() -> void {
    let x: string = "hello";
    consume(x);
    let result: string = concat(x, "world");
}
"#;
    let diags = errors(src);
    assert_has_error(&diags, "AT3053");
}

#[test]
fn test_ownership_own_move_in_nested_fn_isolated() {
    // A move inside a nested function should NOT affect the outer scope
    let src = r#"
fn consume(own data: string) -> void {}
fn main() -> void {
    let outer: string = "outer";
    fn inner() -> void {
        let x: string = "inner";
        consume(x);
    }
    let still_valid: string = outer;
}
"#;
    assert_no_errors(&typecheck(src));
}

#[test]
fn test_h177_rebind_after_own_clears_moved_flag() {
    // x = f(own x) — after the assignment, x is rebound and must be usable again
    let src = r#"
fn take(own val: []number) -> []number { return val; }
fn main() -> void {
    let mut nums: []number = [1, 2, 3];
    nums = take(nums);
    let n: number = nums[0];
}
"#;
    assert_no_errors(&typecheck(src));
}

#[test]
fn test_ownership_own_error_message_contains_variable_name() {
    let src = r#"
fn consume(own data: string) -> void {}
fn main() -> void {
    let my_value: string = "hello";
    consume(my_value);
    let again: string = my_value;
}
"#;
    let diags = errors(src);
    let at3053 = diags.iter().find(|d| d.code == "AT3053");
    assert!(at3053.is_some(), "Expected AT3053");
    assert!(
        at3053.unwrap().message.contains("my_value"),
        "Error message should contain the variable name, got: {}",
        at3053.unwrap().message
    );
}
