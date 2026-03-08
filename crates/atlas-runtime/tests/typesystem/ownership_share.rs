//! B11-P07: Typechecker enforcement of the `share` ownership contract (AT3055).
//!
//! Share params are immutable from the callee's perspective. Tests that mutation
//! and own-transfer of share params trigger AT3055, and valid share usage compiles clean.

use super::{assert_has_error, assert_no_errors, errors, typecheck};

// ============================================================================
// Valid share usage — must compile clean
// ============================================================================

#[test]
fn test_ownership_share_valid_no_mutation_in_body() {
    // Just declaring and reading a share param without mutation is fine.
    // No main call avoids AT3028 (caller-side plain-value-to-share check).
    let src = r#"
fn inspect(share val: string) -> number {
    let limit: number = 5;
    return limit;
}
"#;
    assert_no_errors(&typecheck(src));
}

#[test]
fn test_ownership_share_valid_pass_as_borrow() {
    // Passing a share param to a borrow param within the body is valid
    let src = r#"
fn read_only(borrow s: string) -> void {}
fn shared_relay(share s: string) -> void {
    read_only(s);
}
"#;
    assert_no_errors(&typecheck(src));
}

#[test]
fn test_ownership_share_valid_pass_as_share() {
    // Passing a share param to another share param is valid
    let src = r#"
fn downstream(share s: string) -> void {}
fn upstream(share s: string) -> void {
    downstream(s);
}
"#;
    assert_no_errors(&typecheck(src));
}

// ============================================================================
// AT3055: share param mutation — assignment
// ============================================================================

#[test]
fn test_ownership_share_assignment_fires() {
    // Assigning to a share param fires AT3055
    let src = r#"
fn mutate(share val: string) -> void {
    val = "new value";
}
"#;
    let diags = errors(src);
    assert_has_error(&diags, "AT3055");
}

#[test]
fn test_ownership_share_assignment_message_has_param_name() {
    let src = r#"
fn mutate(share my_shared: string) -> void {
    my_shared = "new";
}
"#;
    let diags = errors(src);
    let at3055 = diags.iter().find(|d| d.code == "AT3055");
    assert!(at3055.is_some(), "Expected AT3055");
    assert!(
        at3055.unwrap().message.contains("my_shared"),
        "Error message should contain param name, got: {}",
        at3055.unwrap().message
    );
}

// ============================================================================
// AT3055: share param own-transfer
// ============================================================================

#[test]
fn test_ownership_share_own_transfer_fires() {
    // Passing a share param to an own parameter fires AT3055
    let src = r#"
fn consume(own data: string) -> void {}
fn relay(share s: string) -> void {
    consume(s);
}
"#;
    let diags = errors(src);
    assert_has_error(&diags, "AT3055");
}

#[test]
fn test_ownership_share_own_transfer_message_has_param_name() {
    let src = r#"
fn consume(own data: string) -> void {}
fn relay(share my_share: string) -> void {
    consume(my_share);
}
"#;
    let diags = errors(src);
    let at3055 = diags.iter().find(|d| d.code == "AT3055");
    assert!(at3055.is_some(), "Expected AT3055");
    assert!(
        at3055.unwrap().message.contains("my_share"),
        "Error message should contain param name, got: {}",
        at3055.unwrap().message
    );
}
