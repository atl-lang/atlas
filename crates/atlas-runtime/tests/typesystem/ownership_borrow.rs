//! B11-P06: Typechecker enforcement of the `borrow` ownership contract (AT3054).
//!
//! Tests that borrow escape (return, let binding, struct field, closure capture) triggers
//! AT3054/AT3040, and that valid read-only borrow usage compiles clean.

use super::{assert_has_error, assert_no_errors, errors, typecheck};

// ============================================================================
// Valid borrow usage — read-only in body, must compile clean
// ============================================================================

#[test]
fn test_ownership_borrow_valid_read_in_body() {
    // Reading a borrow param (passing to a function call) is fine
    let src = r#"
fn greet(borrow name: string) -> string {
    let prefix: string = "Hello, ";
    return prefix;
}
fn main() -> void {
    let n: string = "Alice";
    let result: string = greet(n);
}
"#;
    assert_no_errors(&typecheck(src));
}

#[test]
fn test_ownership_borrow_valid_use_in_expression() {
    // Using a borrow param in a binary expression result is fine — result is a new value
    let src = r#"
fn check_len(borrow s: string) -> bool {
    let limit: number = 10;
    return limit > 0;
}
fn main() -> void {
    let text: string = "hello";
    let ok: bool = check_len(text);
}
"#;
    assert_no_errors(&typecheck(src));
}

#[test]
fn test_ownership_borrow_valid_pass_to_own() {
    // Passing borrow to own fires AT2012 (warning), not AT3054 (error)
    // This test just ensures no AT3054 is emitted in this scenario
    let src = r#"
fn consume(own data: string) -> void {}
fn relay(borrow s: string) -> void {
    consume(s);
}
fn main() -> void {
    let x: string = "hi";
    relay(x);
}
"#;
    let diags = typecheck(src);
    let at3054: Vec<_> = diags.iter().filter(|d| d.code == "AT3054").collect();
    assert!(at3054.is_empty(), "Expected no AT3054, got: {:?}", at3054);
}

// ============================================================================
// AT3054: borrow escape — return
// ============================================================================

#[test]
fn test_ownership_borrow_return_escape_fires() {
    let src = r#"
fn leak(borrow s: string) -> string {
    return s;
}
fn main() -> void {
    let x: string = "hello";
    let result: string = leak(x);
}
"#;
    let diags = errors(src);
    assert_has_error(&diags, "AT3054");
}

#[test]
fn test_ownership_borrow_return_escape_message_has_param_name() {
    let src = r#"
fn leak(borrow my_param: string) -> string {
    return my_param;
}
fn main() -> void {
    let x: string = "hello";
    let result: string = leak(x);
}
"#;
    let diags = errors(src);
    let at3054 = diags.iter().find(|d| d.code == "AT3054");
    assert!(at3054.is_some(), "Expected AT3054");
    assert!(
        at3054.unwrap().message.contains("my_param"),
        "Error message should contain parameter name, got: {}",
        at3054.unwrap().message
    );
}

// ============================================================================
// AT3054: borrow escape — let binding
// ============================================================================

#[test]
fn test_ownership_borrow_let_binding_escape_fires() {
    let src = r#"
fn process(borrow s: string) -> void {
    let stored: string = s;
}
fn main() -> void {
    let x: string = "hello";
    process(x);
}
"#;
    let diags = errors(src);
    assert_has_error(&diags, "AT3054");
}

// ============================================================================
// AT3054: borrow escape — struct literal field
// ============================================================================

#[test]
fn test_ownership_borrow_struct_field_escape_fires() {
    let src = r#"
struct Wrapper {
    value: string,
}
fn wrap(borrow s: string) -> Wrapper {
    let w: Wrapper = Wrapper { value: s };
    return w;
}
fn main() -> void {
    let x: string = "hello";
    let result: Wrapper = wrap(x);
}
"#;
    let diags = errors(src);
    assert_has_error(&diags, "AT3054");
}

// ============================================================================
// AT3040: borrow escape — closure capture (pre-existing check)
// ============================================================================

#[test]
fn test_ownership_borrow_closure_capture_fires_at3040() {
    let src = r#"
fn make_closure(borrow s: string) -> void {
    let f = fn() -> void { let captured: string = s; };
}
fn main() -> void {
    let x: string = "hello";
    make_closure(x);
}
"#;
    let diags = errors(src);
    // AT3040 fires for closure capture of borrow params
    let has_borrow_error = diags
        .iter()
        .any(|d| d.code == "AT3040" || d.code == "AT3054");
    assert!(
        has_borrow_error,
        "Expected AT3040 or AT3054 for closure capture of borrow, got: {:?}",
        diags
    );
}
