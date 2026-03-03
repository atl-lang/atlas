use super::*;

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_function_in_loop() {
    let source = r#"
        fn apply(f: (number) -> number, x: number) -> number {
            return f(x);
        }
        fn inc(n: number) -> number { return n + 1; }
        let mut result = 0;
        for i in [0, 1, 2] {
            result = apply(inc, result);
            let _unused = i;
        }
        result;
    "#;
    assert_eval_number(source, 3.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_map_pattern_with_function() {
    let source = r#"
        fn applyToArray(arr: number[], f: (number) -> number) -> number[] {
            let mut result: number[] = [];
            for item in arr {
                result = result + [f(item)];
            }
            return result;
        }
        fn double(x: number) -> number { return x * 2; }
        let arr = [1, 2, 3];
        let doubled = applyToArray(arr, double);
        doubled[0] + doubled[1] + doubled[2];
    "#;
    assert_eval_number(source, 12.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_filter_pattern_with_function() {
    let source = r#"
        fn filterArray(arr: number[], predicate: (number) -> bool) -> number[] {
            let mut result: number[] = [];
            for item in arr {
                if (predicate(item)) {
                    result = result + [item];
                }
            }
            return result;
        }
        fn isEven(x: number) -> bool { return x % 2 == 0; }
        let arr = [1, 2, 3, 4, 5, 6];
        let evens = filterArray(arr, isEven);
        len(evens);
    "#;
    assert_eval_number(source, 3.0);
}

#[test]
fn test_reduce_pattern_with_function() {
    let source = r#"
        fn reduceArray(
            arr: number[],
            reducer: (number, number) -> number,
            initial: number
        ) -> number {
            let mut acc = initial;
            for item in arr {
                acc = reducer(acc, item);
            }
            return acc;
        }
        fn add(a: number, b: number) -> number { return a + b; }
        let arr = [1, 2, 3, 4, 5];
        reduceArray(arr, add, 0);
    "#;
    assert_eval_number(source, 15.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_complex_function_passing() {
    let source = r#"
        fn transform(
            arr: number[],
            f1: (number) -> number,
            f2: (number) -> number
        ) -> number {
            let mut sum = 0;
            for item in arr {
                sum = sum + f1(f2(item));
            }
            return sum;
        }
        fn double(x: number) -> number { return x * 2; }
        fn square(x: number) -> number { return x * x; }
        transform([1, 2, 3], double, square);
    "#;
    assert_eval_number(source, 28.0);
}
