use super::common::*;

// ============================================================================
// H-384: TupleGet error on mixed literal/wildcard tuple patterns
// ============================================================================

#[test]
fn test_h384_tuple_match_literal_wildcard_mixed() {
    // Regression: (0, y) arm after (x, 0) arm crashed with
    // "TupleGet applied to non-tuple value" because SetLocal for pattern
    // variable x overwrote the scrutinee stack slot.
    let code = r#"
fn classify(own pair: (number, number)): string {
    match pair {
        (0, 0) => "origin",
        (x, 0) => "x-axis",
        (0, y) => "y-axis",
        (x, y) => "quadrant"
    }
}
classify((0, 3))
"#;
    assert_eval_string(code, "y-axis");
}

#[test]
fn test_h384_tuple_match_all_arms() {
    let code = r#"
fn classify(own pair: (number, number)): string {
    match pair {
        (0, 0) => "origin",
        (x, 0) => "x-axis",
        (0, y) => "y-axis",
        (x, y) => "quadrant"
    }
}
let a = classify((0, 0));
let b = classify((5, 0));
let c = classify((0, 3));
let d = classify((2, 4));
d
"#;
    assert_eval_string(code, "quadrant");
}

#[test]
fn test_h384_tuple_match_two_vars_single_arm() {
    // Regression for secondary bug: (x, y) as last arm with two variable
    // bindings caused SetLocal stack underflow when local_count > index + 1.
    let code = r#"
fn swap(own p: (number, number)): (number, number) {
    match p {
        (x, y) => (y, x)
    }
}
let r = swap((3, 7));
r.0
"#;
    assert_eval_number(code, 7.0);
}

#[test]
fn test_h384_tuple_match_second_arm_variable() {
    let code = r#"
fn classify(own pair: (number, number)): string {
    match pair {
        (0, 0) => "origin",
        (x, 0) => "x-axis",
        (0, y) => "y-axis",
        (x, y) => "quadrant"
    }
}
classify((5, 0))
"#;
    assert_eval_string(code, "x-axis");
}
