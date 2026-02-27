//! Type inference error message tests (Block 5 Phase 6)

use super::super::*;
#[allow(unused_imports)]
use super::helpers::*;

// ============================================================================
// Inference error messages (Block 5 Phase 6)
// ============================================================================

#[test]
fn test_at3050_includes_function_name() {
    let diags = errors(
        r#"
fn confused(x: number) {
    if (x > 0) { return 1; } else { return "bad"; }
}
"#,
    );
    let at3050 = diags.iter().find(|d| d.code == "AT3050");
    assert!(
        at3050.is_some(),
        "Expected AT3050, got: {:?}",
        diags.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
    let msg = &at3050.unwrap().message;
    assert!(
        msg.contains("confused"),
        "AT3050 message should include function name 'confused', got: {}",
        msg
    );
}

#[test]
fn test_at3051_includes_type_param_name() {
    let diags = errors(
        r#"
fn make<T>() -> T { return 42; }
make();
"#,
    );
    let at3051 = diags.iter().find(|d| d.code == "AT3051");
    assert!(
        at3051.is_some(),
        "Expected AT3051, got: {:?}",
        diags.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
    let msg = &at3051.unwrap().message;
    assert!(
        msg.contains('T'),
        "AT3051 message should include type param name 'T', got: {}",
        msg
    );
}

#[test]
fn test_at3052_fires_for_identifier_binary_op_mismatch() {
    let diags = errors(r#"let x = 42; x + "string";"#);
    assert!(
        diags.iter().any(|d| d.code == "AT3052"),
        "Expected AT3052 for identifier in binary mismatch, got: {:?}",
        diags.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn test_at3052_message_mentions_inferred_type() {
    let diags = errors(r#"let x = 42; x + "string";"#);
    let at3052 = diags.iter().find(|d| d.code == "AT3052");
    assert!(at3052.is_some(), "Expected AT3052");
    let msg = &at3052.unwrap().message;
    assert!(
        msg.contains("number"),
        "AT3052 message should mention the inferred type 'number', got: {}",
        msg
    );
}

#[test]
fn test_at3050_not_fired_for_consistent_returns() {
    // Consistent return type should NOT trigger AT3050
    let diags = errors(
        r#"
fn consistent(x: number) {
    if (x > 0) { return 1; } else { return 2; }
}
"#,
    );
    assert!(
        !diags.iter().any(|d| d.code == "AT3050"),
        "AT3050 should not fire for consistent returns, got: {:?}",
        diags.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn test_at3051_not_fired_when_type_param_inferrable() {
    // identity(42) can infer T=number → no AT3051
    let diags = errors(
        r#"
fn identity<T>(x: T) -> T { return x; }
identity(42);
"#,
    );
    assert!(
        !diags.iter().any(|d| d.code == "AT3051"),
        "AT3051 should not fire when type param is inferrable, got: {:?}",
        diags.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

