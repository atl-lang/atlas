use super::*;

// ============================================================================
// B10-P04: Array method surface — dot-syntax for all arr.method() calls
// Both interpreter and VM parity tested throughout.
// ============================================================================

// --- arr.len() ---

#[test]
fn test_array_method_len() {
    let src = r#"let arr: number[] = [1, 2, 3]; arr.len();"#;
    assert_eval_number(src, 3.0);
    assert_parity(src);
}

#[test]
fn test_array_method_len_empty() {
    let src = r#"let arr: number[] = []; arr.len();"#;
    assert_eval_number(src, 0.0);
    assert_parity(src);
}

// --- arr.isEmpty() ---

#[test]
fn test_array_method_is_empty_true() {
    let src = r#"let arr: number[] = []; arr.isEmpty();"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_array_method_is_empty_false() {
    let src = r#"let arr: number[] = [1]; arr.isEmpty();"#;
    assert_eval_bool(src, false);
    assert_parity(src);
}

// --- arr.includes(x) ---

#[test]
fn test_array_method_includes_found() {
    let src = r#"let arr: number[] = [1, 2, 3]; arr.includes(2);"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_array_method_includes_not_found() {
    let src = r#"let arr: number[] = [1, 2, 3]; arr.includes(99);"#;
    assert_eval_bool(src, false);
    assert_parity(src);
}

// --- arr.indexOf(x) ---

#[test]
fn test_array_method_index_of_found() {
    // indexOf returns Option<number> — test parity only
    let src = r#"let arr: number[] = [10, 20, 30]; arr.indexOf(20);"#;
    assert_parity(src);
}

#[test]
fn test_array_method_index_of_not_found() {
    let src = r#"let arr: number[] = [1, 2, 3]; arr.indexOf(99);"#;
    assert_parity(src);
}

// --- arr.slice(start, end) ---

#[test]
fn test_array_method_slice_basic() {
    let src =
        r#"let arr: number[] = [1, 2, 3, 4, 5]; let s: number[] = arr.slice(1, 3); s[0] + s[1];"#;
    assert_eval_number(src, 5.0); // 2 + 3
    assert_parity(src);
}

#[test]
fn test_array_method_slice_len() {
    let src = r#"let arr: number[] = [1, 2, 3, 4, 5]; let s: number[] = arr.slice(0, 3); len(s);"#;
    assert_eval_number(src, 3.0);
    assert_parity(src);
}

// --- arr.concat(arr2) ---

#[test]
fn test_array_method_concat_basic() {
    let src = r#"
        let a: number[] = [1, 2];
        let b: number[] = [3, 4];
        let c: number[] = a.concat(b);
        len(c);
    "#;
    assert_eval_number(src, 4.0);
    assert_parity(src);
}

#[test]
fn test_array_method_concat_values() {
    let src = r#"
        let a: number[] = [1, 2];
        let b: number[] = [3, 4];
        let c: number[] = a.concat(b);
        c[0] + c[3];
    "#;
    assert_eval_number(src, 5.0); // 1 + 4
    assert_parity(src);
}

// --- arr.map(fn) ---

#[test]
fn test_h137_map_return_type_inferred_from_named_fn() {
    // H-137: arr.map(dbl) should return number[], not ?[]
    // Explicit annotation must not error.
    let src = r#"
        fn dbl(x: number) -> number { return x * 2; }
        let arr: number[] = [1, 2, 3];
        let result: number[] = arr.map(dbl);
        result[2];
    "#;
    assert_eval_number(src, 6.0);
    assert_parity(src);
}

#[test]
fn test_array_method_map_double() {
    let src = r#"
        let arr: number[] = [1, 2, 3];
        let doubled: number[] = arr.map(fn(x: number) -> number { return x * 2; });
        doubled[0] + doubled[2];
    "#;
    assert_eval_number(src, 8.0); // 2 + 6
    assert_parity(src);
}

#[test]
fn test_array_method_map_preserves_len() {
    let src = r#"
        let arr: number[] = [1, 2, 3, 4];
        let result = arr.map(fn(x: number) -> number { return x + 10; });
        len(result);
    "#;
    assert_eval_number(src, 4.0);
    assert_parity(src);
}

// --- arr.filter(fn) ---

#[test]
fn test_array_method_filter_gt() {
    let src = r#"
        let arr: number[] = [1, 2, 3, 4, 5];
        let big = arr.filter(fn(x: number) -> bool { return x > 3; });
        len(big);
    "#;
    assert_eval_number(src, 2.0);
    assert_parity(src);
}

#[test]
fn test_array_method_filter_values() {
    let src = r#"
        let arr: number[] = [10, 20, 30, 40];
        let big = arr.filter(fn(x: number) -> bool { return x > 15; });
        big[0];
    "#;
    assert_eval_number(src, 20.0);
    assert_parity(src);
}

// --- arr.reduce(fn, init) ---

#[test]
fn test_array_method_reduce_sum() {
    let src = r#"
        let arr: number[] = [1, 2, 3, 4, 5];
        let total = arr.reduce(fn(acc: number, x: number) -> number { return acc + x; }, 0);
        total;
    "#;
    assert_eval_number(src, 15.0);
    assert_parity(src);
}

#[test]
fn test_array_method_reduce_product() {
    let src = r#"
        let arr: number[] = [1, 2, 3, 4];
        let product = arr.reduce(fn(acc: number, x: number) -> number { return acc * x; }, 1);
        product;
    "#;
    assert_eval_number(src, 24.0);
    assert_parity(src);
}

// --- arr.find(fn) ---

#[test]
fn test_array_method_find_match() {
    // find returns an optional value — test parity only
    let src = r#"
        let arr: number[] = [1, 2, 3, 4, 5];
        arr.find(fn(x: number) -> bool { return x > 3; });
    "#;
    assert_parity(src);
}

// --- arr.findIndex(fn) ---

#[test]
fn test_array_method_find_index_match() {
    // findIndex returns an optional index — test parity only
    let src = r#"
        let arr: number[] = [10, 20, 30, 40];
        arr.findIndex(fn(x: number) -> bool { return x == 30; });
    "#;
    assert_parity(src);
}

// --- arr.some(fn) ---

#[test]
fn test_array_method_some_true() {
    let src = r#"
        let arr: number[] = [1, 2, 3];
        arr.some(fn(x: number) -> bool { return x > 2; });
    "#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_array_method_some_false() {
    let src = r#"
        let arr: number[] = [1, 2, 3];
        arr.some(fn(x: number) -> bool { return x > 10; });
    "#;
    assert_eval_bool(src, false);
    assert_parity(src);
}

// --- arr.every(fn) ---

#[test]
fn test_array_method_every_true() {
    let src = r#"
        let arr: number[] = [2, 4, 6];
        arr.every(fn(x: number) -> bool { return x > 0; });
    "#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_array_method_every_false() {
    let src = r#"
        let arr: number[] = [2, 3, 6];
        arr.every(fn(x: number) -> bool { return x > 5; });
    "#;
    assert_eval_bool(src, false);
    assert_parity(src);
}

// --- arr.forEach(fn) ---

#[test]
fn test_array_method_for_each_side_effect() {
    let src = r#"
        let arr: number[] = [1, 2, 3];
        let mut total: number = 0;
        arr.forEach(fn(x: number) -> void { total = total + x; });
        total;
    "#;
    assert_eval_number(src, 6.0);
    assert_parity(src);
}

// --- Multi-step pipeline (via intermediate vars to avoid chained-call VM bug) ---

#[test]
fn test_array_method_pipeline_filter_then_map() {
    let src = r#"
        let arr: number[] = [1, 2, 3, 4, 5, 6];
        let filtered = arr.filter(fn(x: number) -> bool { return x > 3; });
        let result = filtered.map(fn(x: number) -> number { return x * 10; });
        result[0];
    "#;
    assert_eval_number(src, 40.0);
    assert_parity(src);
}

#[test]
fn test_array_method_pipeline_map_then_reduce() {
    let src = r#"
        let arr: number[] = [1, 2, 3];
        let squares = arr.map(fn(x: number) -> number { return x * x; });
        let result = squares.reduce(fn(acc: number, x: number) -> number { return acc + x; }, 0);
        result;
    "#;
    assert_eval_number(src, 14.0); // 1 + 4 + 9
    assert_parity(src);
}
