//! For-in loop edge case tests (Phase-20d)
//!
//! Comprehensive edge case testing for for-in loops.

use atlas_runtime::{Atlas, Value};

#[test]
fn test_for_in_large_array() {
    // Simplified: Use a smaller array to test iteration stability
    let source = r#"
        let arr: array = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
            10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
            20, 21, 22, 23, 24, 25, 26, 27, 28, 29
        ];

        var sum: number = 0;
        for item in arr {
            sum = sum + item;
        }

        sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    // Sum of 0..29 = 29 * 30 / 2 = 435
    assert_eq!(result.unwrap(), Value::Number(435.0));
}

#[test]
fn test_for_in_deeply_nested() {
    let source = r#"
        let arr3d: array = [
            [[1, 2], [3, 4]],
            [[5, 6], [7, 8]]
        ];

        var sum: number = 0;
        for layer in arr3d {
            for row in layer {
                for item in row {
                    sum = sum + item;
                }
            }
        }

        sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(result.unwrap(), Value::Number(36.0), "Sum 1+2+..+8=36");
}

#[test]
fn test_for_in_with_array_iteration_count() {
    // Test that iteration count is correct
    let source = r#"
        let arr: array = [1, 2, 3, 4, 5];
        var count: number = 0;

        for item in arr {
            count = count + 1;
        }

        count
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(
        result.unwrap(),
        Value::Number(5.0),
        "Should iterate 5 times"
    );
}

#[test]
fn test_for_in_with_early_return() {
    let source = r#"
        fn find_first_even(arr: array) -> number {
            for item in arr {
                if (item % 2 == 0) {
                    return item;
                }
            }
            return -1;
        }

        find_first_even([1, 3, 5, 8, 10])
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(
        result.unwrap(),
        Value::Number(8.0),
        "Should return first even number"
    );
}

#[test]
fn test_for_in_with_complex_expressions() {
    let source = r#"
        let arr: array = [1, 2, 3, 4, 5];
        var sum_even: number = 0;
        var sum_odd: number = 0;

        for item in arr {
            if (item % 2 == 0) {
                sum_even = sum_even + item;
            } else {
                sum_odd = sum_odd + item;
            }
        }

        sum_even + sum_odd
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(
        result.unwrap(),
        Value::Number(15.0),
        "Sum of all items = 15"
    );
}

#[test]
fn test_for_in_break_in_nested_loop() {
    let source = r#"
        let matrix: array = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        var found: bool = false;

        for row in matrix {
            for item in row {
                if (item == 5) {
                    found = true;
                    break;
                }
            }
            if (found) {
                break;
            }
        }

        found
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
fn test_for_in_multiple_sequential() {
    let source = r#"
        let arr1: array = [1, 2, 3];
        let arr2: array = [4, 5, 6];
        var sum: number = 0;

        for item in arr1 {
            sum = sum + item;
        }

        for item in arr2 {
            sum = sum + item;
        }

        sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(result.unwrap(), Value::Number(21.0), "Sum of 1..6 = 21");
}

#[test]
fn test_for_in_with_function_calls() {
    let source = r#"
        fn double(x: number) -> number {
            return x * 2;
        }

        let arr: array = [1, 2, 3];
        var sum: number = 0;

        for item in arr {
            sum = sum + double(item);
        }

        sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(
        result.unwrap(),
        Value::Number(12.0),
        "Sum of doubled items = 12"
    );
}

#[test]
fn test_for_in_with_hashmap_keys() {
    let source = r#"
        let hmap: HashMap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);

        let keys: array = hashMapKeys(hmap);
        var count: number = 0;

        for key in keys {
            count = count + 1;
        }

        count
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(result.unwrap(), Value::Number(3.0));
}

#[test]
fn test_for_in_with_hashset() {
    let source = r#"
        let set: HashSet = hashSetNew();
        hashSetAdd(set, 10);
        hashSetAdd(set, 20);
        hashSetAdd(set, 30);

        let arr: array = hashSetToArray(set);
        var sum: number = 0;

        for item in arr {
            sum = sum + item;
        }

        sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(result.unwrap(), Value::Number(60.0));
}

#[test]
fn test_for_in_with_result_early_return() {
    let source = r#"
        fn process(arr: array) -> number {
            var sum: number = 0;
            for item in arr {
                if (item < 0) {
                    return -999;
                }
                sum = sum + item;
            }
            return sum;
        }

        process([1, 2, 3])
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(
        result.unwrap(),
        Value::Number(6.0),
        "Should return sum of positive numbers"
    );
}

#[test]
fn test_for_in_with_conditional_sum() {
    let source = r#"
        let arr: array = [1, -1, 2, -2, 3, -3];
        var pos_sum: number = 0;
        var neg_sum: number = 0;

        for num in arr {
            if (num > 0) {
                pos_sum = pos_sum + num;
            } else {
                neg_sum = neg_sum + num;
            }
        }

        pos_sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(
        result.unwrap(),
        Value::Number(6.0),
        "Sum of positive values"
    );
}

#[test]
fn test_for_in_performance() {
    // Build a large array literal for performance testing
    let mut array_elements = Vec::new();
    for i in 0..1000 {
        array_elements.push(i.to_string());
    }
    let array_literal = format!("[{}]", array_elements.join(", "));

    let source = format!(
        r#"
        let arr: array = {};

        var sum: number = 0;
        for item in arr {{
            sum = sum + item;
        }}

        sum
    "#,
        array_literal
    );

    let start = std::time::Instant::now();
    let runtime = Atlas::new();
    let result = runtime.eval(&source);
    let duration = start.elapsed();

    assert!(result.is_ok());
    // Sum of 0..999 = 999 * 1000 / 2 = 499500
    assert_eq!(result.unwrap(), Value::Number(499500.0));
    assert!(
        duration.as_millis() < 2000,
        "Should complete in < 2s, took {}ms",
        duration.as_millis()
    );
}
