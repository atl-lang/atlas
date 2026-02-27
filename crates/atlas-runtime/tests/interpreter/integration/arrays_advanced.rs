use super::*;
use pretty_assertions::assert_eq;

fn test_cow_index_mutation_does_not_affect_original() {
    assert_eval_number(
        "var a: array = [1, 2, 3]; var b: array = a; b[0] = 99; a[0];",
        1.0,
    );
}

#[test]
fn test_cow_cloned_array_gets_mutation() {
    assert_eval_number(
        "var a: array = [1, 2, 3]; var b: array = a; b[0] = 99; b[0];",
        99.0,
    );
}

/// Compound assignment (`+=`) on array index writes back correctly.
#[test]
fn test_array_compound_assign_add() {
    assert_eval_number("var arr: array = [10, 20, 30]; arr[1] += 5; arr[1];", 25.0);
}

/// Increment (`++`) on array index writes back correctly.
#[test]
fn test_array_increment_writes_back() {
    assert_eval_number("var arr: array = [5, 6, 7]; arr[0]++; arr[0];", 6.0);
}

/// Decrement (`--`) on array index writes back correctly.
#[test]
fn test_array_decrement_writes_back() {
    assert_eval_number("var arr: array = [5, 6, 7]; arr[2]--; arr[2];", 6.0);
}

/// Multiple mutations accumulate on the same variable.
#[test]
fn test_array_multiple_mutations_accumulate() {
    assert_eval_number(
        "var arr: array = [0, 0, 0]; arr[0] = 10; arr[1] = 20; arr[2] = 30; arr[0] + arr[1] + arr[2];",
        60.0,
    );
}

/// Loop-based array mutation: each iteration writes back correctly.
#[test]
fn test_array_mutation_in_loop() {
    assert_eval_number(
        r#"
            var arr: array = [1, 2, 3, 4, 5];
            var i = 0;
            while (i < 5) {
                arr[i] = arr[i] * 2;
                i = i + 1;
            }
            arr[0] + arr[1] + arr[2] + arr[3] + arr[4];
        "#,
        30.0,
    );
}

// ============================================================================
// Phase 16: Stdlib Return Value Propagation — array method CoW write-back
// ============================================================================

/// arr.push(x) — receiver variable updated in place (CoW write-back)
#[test]
fn test_array_method_push_updates_receiver() {
    assert_eval_number(r#"var arr: array = [1, 2, 3]; arr.push(4); arr[3];"#, 4.0);
}

/// arr.push(x) — length increases
#[test]
fn test_array_method_push_increases_len() {
    assert_eval_number(r#"var arr: array = [1, 2]; arr.push(3); len(arr);"#, 3.0);
}

/// arr.push chained — multiple pushes accumulate
#[test]
fn test_array_method_push_multiple() {
    assert_eval_number(
        r#"var arr: array = []; arr.push(10); arr.push(20); arr.push(30); arr[1];"#,
        20.0,
    );
}

/// arr.pop() — returns the popped element
#[test]
fn test_array_method_pop_returns_element() {
    assert_eval_number(r#"var arr: array = [1, 2, 3]; let x = arr.pop(); x;"#, 3.0);
}

/// arr.pop() — receiver shortened by one element
#[test]
fn test_array_method_pop_shrinks_receiver() {
    assert_eval_number(r#"var arr: array = [1, 2, 3]; arr.pop(); len(arr);"#, 2.0);
}

/// arr.pop() — receiver still holds correct remaining elements
#[test]
fn test_array_method_pop_receiver_correct() {
    assert_eval_number(
        r#"var arr: array = [10, 20, 30]; arr.pop(); arr[0] + arr[1];"#,
        30.0,
    );
}

/// arr.sort() — returns a new sorted array
#[test]
fn test_array_method_sort_returns_sorted() {
    assert_eval_number(
        r#"var arr: array = [3, 1, 2]; let s = arr.sort(); s[0];"#,
        1.0,
    );
}

/// arr.sort() — does NOT mutate the receiver
#[test]
fn test_array_method_sort_non_mutating() {
    assert_eval_number(
        r#"var arr: array = [3, 1, 2]; let s = arr.sort(); arr[0];"#,
        3.0,
    );
}

/// arr.sort() — numeric sort (ascending by value)
#[test]
fn test_array_method_sort_numeric() {
    assert_eval_number(
        r#"var arr: array = [10, 2, 30, 4]; let s = arr.sort(); s[0];"#,
        2.0,
    );
}

/// arr.reverse() — receiver is updated with reversed array (mutating)
#[test]
fn test_array_method_reverse_updates_receiver() {
    assert_eval_number(r#"var arr: array = [1, 2, 3]; arr.reverse(); arr[0];"#, 3.0);
}

/// arr.reverse() — result is the reversed array
#[test]
fn test_array_method_reverse_result_correct() {
    assert_eval_number(
        r#"var arr: array = [1, 2, 3]; let r = arr.reverse(); r[0];"#,
        3.0,
    );
}

/// Free function pop(arr) CoW write-back — pop() as free function also updates receiver
#[test]
fn test_free_fn_pop_cow_writeback() {
    assert_eval_number(
        r#"var arr: array = [1, 2, 3]; let x = pop(arr); len(arr);"#,
        2.0,
    );
}

/// Free function pop(arr) — returns removed element
#[test]
fn test_free_fn_pop_returns_element() {
    assert_eval_number(r#"var arr: array = [1, 2, 3]; let x = pop(arr); x;"#, 3.0);
}

/// Free function shift(arr) — removes first element
#[test]
fn test_free_fn_shift_cow_writeback() {
    assert_eval_number(
        r#"var arr: array = [10, 20, 30]; let x = shift(arr); x;"#,
        10.0,
    );
}

/// Free function shift(arr) — receiver is updated
#[test]
fn test_free_fn_shift_receiver_updated() {
    assert_eval_number(
        r#"var arr: array = [10, 20, 30]; shift(arr); len(arr);"#,
        2.0,
    );
}

/// Free function reverse(arr) — writes new array back to receiver
#[test]
fn test_free_fn_reverse_cow_writeback() {
    assert_eval_number(r#"var arr: array = [1, 2, 3]; reverse(arr); arr[0];"#, 3.0);
}
