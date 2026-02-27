use super::*;

fn test_edge_empty_function() {
    let code = "fn noop() { } noop();";
    assert_no_error(code);
}

#[test]
fn test_edge_deeply_nested_if() {
    let code = r#"
        var x = 0;
        if (true) {
            if (true) {
                if (true) {
                    x = 1;
                }
            }
        }
        x;
    "#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_edge_boolean_short_circuit_and() {
    // If short-circuit works, second function should not be called
    let code = r#"
        var called = 0;
        fn side_effect() -> bool {
            called = called + 1;
            return true;
        }
        let result = false && side_effect();
        called;
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_edge_boolean_short_circuit_or() {
    // If short-circuit works, second function should not be called
    let code = r#"
        var called = 0;
        fn side_effect() -> bool {
            called = called + 1;
            return false;
        }
        let result = true || side_effect();
        called;
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_edge_return_from_nested_block() {
    let code = r#"
        fn test() -> number {
            if (true) {
                if (true) {
                    return 42;
                }
            }
            return 0;
        }
        test();
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_edge_while_loop_early_break() {
    // Note: Atlas may not have break keyword - if it does, test it
    // Otherwise test early return from function containing loop
    let code = r#"
        fn first_over_5() -> number {
            var i = 0;
            while (i < 100) {
                if (i > 5) { return i; }
                i = i + 1;
            }
            return -1;
        }
        first_over_5();
    "#;
    assert_eval_number(code, 6.0);
}

// ============================================================================
// Phase 07: Array Mutation CoW Semantics (Interpreter)
// ============================================================================

/// Index assignment writes back to the variable in the environment.
///
/// Previously, `set_array_element` mutated a local copy and discarded it.
/// Now `assign_at_index` clones the container, mutates via CoW, and writes back.
#[test]
fn test_array_index_assignment_write_back() {
    assert_eval_number("var arr: array = [10, 20, 30]; arr[1] = 99; arr[1];", 99.0);
}

#[test]
fn test_array_index_assignment_first_element() {
    assert_eval_number("var arr: array = [1, 2, 3]; arr[0] = 42; arr[0];", 42.0);
}

#[test]
fn test_array_index_assignment_last_element() {
    assert_eval_number("var arr: array = [1, 2, 3]; arr[2] = 77; arr[2];", 77.0);
}
