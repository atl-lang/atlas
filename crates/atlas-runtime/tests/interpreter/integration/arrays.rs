use super::*;

#[test]
fn test_array_literal() {
    let code = r#"
        let arr: number[] = [1, 2, 3];
        arr[1]
    "#;
    assert_eval_number(code, 2.0);
}

#[test]
fn test_array_assignment() {
    let code = r#"
        let arr: number[] = [1, 2, 3];
        arr[1] = 99;
        arr[1]
    "#;
    assert_eval_number(code, 99.0);
}

#[test]
fn test_array_reference_semantics() {
    // CoW value semantics: arr2 is a logical copy of arr1.
    // Mutating arr1[0] triggers CoW — arr2 retains the original value.
    let code = r#"
        let arr1: number[] = [1, 2, 3];
        let arr2: number[] = arr1;
        arr1[0] = 42;
        arr2[0]
    "#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_empty_array() {
    let code = r#"
        let arr: number[] = [];
        len(arr)
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_stdlib_len_array() {
    let code = r#"
        let arr: number[] = [1, 2, 3, 4];
        len(arr)
    "#;
    assert_eval_number(code, 4.0);
}

#[test]
fn test_nested_array_literal() {
    let code = r#"
        let arr: number[][] = [[1, 2], [3, 4]];
        arr[1][0]
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_nested_array_mutation() {
    let code = r#"
        let arr: number[][] = [[1, 2], [3, 4]];
        arr[0][1] = 99;
        arr[0][1]
    "#;
    assert_eval_number(code, 99.0);
}

#[test]
fn test_array_whole_number_float_index() {
    let code = r#"
        let arr: number[] = [1, 2, 3];
        arr[1.0]
    "#;
    assert_eval_number(code, 2.0);
}

#[test]
fn test_array_slice_range() {
    let code = r#"
        let arr: number[] = [1, 2, 3, 4];
        let part: number[] = arr[1..3];
        part[0] + part[1] + len(part)
    "#;
    assert_eval_number(code, 7.0);
}

#[test]
fn test_array_slice_from() {
    let code = r#"
        let arr: number[] = [1, 2, 3, 4];
        let part: number[] = arr[1..];
        part[0] + part[2] + len(part)
    "#;
    assert_eval_number(code, 9.0);
}

#[test]
fn test_array_slice_to() {
    let code = r#"
        let arr: number[] = [1, 2, 3, 4];
        let part: number[] = arr[..3];
        part[0] + part[2] + len(part)
    "#;
    assert_eval_number(code, 7.0);
}

#[test]
fn test_array_slice_full() {
    let code = r#"
        let arr: number[] = [1, 2, 3];
        let part: number[] = arr[..];
        part[0] + part[2] + len(part)
    "#;
    assert_eval_number(code, 7.0);
}

#[test]
fn test_array_slice_inclusive() {
    let code = r#"
        let arr: number[] = [1, 2, 3, 4];
        let part: number[] = arr[1..=3];
        part[0] + part[2] + len(part)
    "#;
    // elements: [2,3,4], sum = 2 + 4 + 3(len) = 9
    assert_eval_number(code, 9.0);
}

#[test]
fn test_array_slice_range_value() {
    let code = r#"
        let arr: number[] = [1, 2, 3, 4];
        let r = 1..3;
        let part: number[] = arr[r];
        part[0] + part[1] + len(part)
    "#;
    assert_eval_number(code, 7.0);
}

#[rstest]
#[case("let arr: number[] = [1, 2, 3]; arr[5]", "AT0006")]
#[case("let arr: number[] = [1, 2, 3]; arr[10] = 99; arr[0]", "AT0006")]
fn test_array_out_of_bounds(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[rstest]
#[case("let arr: number[] = [1, 2, 3]; arr[-1]", "AT0103")]
#[case("let arr: number[] = [1, 2, 3]; arr[-1] = 99; arr[0]", "AT0103")]
#[case("let arr: number[] = [1, 2, 3]; arr[1.5]", "AT0103")]
#[case("let arr: number[] = [1, 2, 3]; arr[0.5] = 99; arr[0]", "AT0103")]
fn test_array_invalid_index(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[test]
fn test_array_mutation_in_function() {
    // CoW value semantics: function receives a logical copy of the array.
    // Mutations inside the function do not affect the caller's binding.
    let code = r#"
        fn modify(arr: number[]) -> void {
            arr[0] = 999;
        }
        let numbers: number[] = [1, 2, 3];
        modify(numbers);
        numbers[0]
    "#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_array_aliasing_multiple_aliases() {
    let code = r#"
        let arr1: number[] = [1, 2, 3];
        let arr2: number[] = arr1;
        let arr3: number[] = arr2;
        arr1[0] = 100;
        arr2[1] = 200;
        arr3[2] = 300;
        arr1[0] + arr2[1] + arr3[2]
    "#;
    assert_eval_number(code, 600.0);
}

#[test]
fn test_array_aliasing_nested_arrays() {
    // CoW value semantics: `row` is a logical copy of matrix[0].
    // Mutating row[0] does not affect matrix[0][0].
    let code = r#"
        let matrix: number[][] = [[1, 2], [3, 4]];
        let row: number[] = matrix[0];
        row[0] = 99;
        matrix[0][0]
    "#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_array_aliasing_identity_equality() {
    let code = r#"
        let arr1: number[] = [1, 2, 3];
        let arr2: number[] = arr1;
        arr1 == arr2
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_array_aliasing_different_arrays_not_equal() {
    // CoW value semantics: equality is structural (same content = equal).
    // Two independently-constructed [1,2,3] arrays are equal.
    let code = r#"
        let arr1: number[] = [1, 2, 3];
        let arr2: number[] = [1, 2, 3];
        arr1 == arr2
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_array_aliasing_reassignment_breaks_link() {
    let code = r#"
        let mut arr1: number[] = [1, 2, 3];
        let mut arr2: number[] = arr1;
        arr2 = [10, 20, 30];
        arr2[0] = 99;
        arr1[0]
    "#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_array_sum_with_function() {
    let code = r#"
        fn sum_array(arr: number[]) -> number {
            let mut total: number = 0;
            let mut i: number = 0;
            while (i < len(arr)) {
                total = total + arr[i];
                i = i + 1;
            }
            return total;
        }
        let numbers: number[] = [1, 2, 3, 4, 5];
        sum_array(numbers)
    "#;
    assert_eval_number(code, 15.0);
}

// ============================================================================
