use super::*;

// B41-P04: Variadic (rest) parameters

#[test]
fn test_variadic_sum_basic() {
    let source = r#"
        fn sum(...nums: number[]): number {
            let mut t = 0;
            for n in nums {
                t = t + n;
            }
            return t;
        }
        sum(1, 2, 3, 4);
    "#;
    assert_eval_number(source, 10.0);
}

#[test]
fn test_variadic_zero_args() {
    let source = r#"
        fn sum(...nums: number[]): number {
            let mut t = 0;
            for n in nums {
                t = t + n;
            }
            return t;
        }
        sum();
    "#;
    assert_eval_number(source, 0.0);
}

#[test]
fn test_variadic_with_fixed_params() {
    let source = r#"
        fn greet(prefix: string, ...names: string[]): string {
            let mut result = prefix;
            for name in names {
                result = result + " " + name;
            }
            return result;
        }
        greet("Hello", "Alice", "Bob");
    "#;
    assert_eval_string(source, "Hello Alice Bob");
}

#[test]
fn test_variadic_fixed_only_no_rest() {
    let source = r#"
        fn greet(prefix: string, ...names: string[]): string {
            let mut result = prefix;
            for name in names {
                result = result + " " + name;
            }
            return result;
        }
        greet("Hi");
    "#;
    assert_eval_string(source, "Hi");
}

#[test]
fn test_function_in_loop() {
    let source = r#"
        fn apply(borrow f: (number): number, x: number): number {
            return f(x);
        }
        fn inc(borrow n: number): number { return n + 1; }
        let mut result = 0;
        for i in [0, 1, 2] {
            result = apply(inc, result);
            let _unused = i;
        }
        result;
    "#;
    assert_eval_number(source, 3.0);
}

#[test]
fn test_map_pattern_with_function() {
    let source = r#"
        fn applyToArray(borrow arr: number[], borrow f: (number): number): number[] {
            let mut result: number[] = [];
            for item in arr {
                result = result + [f(item)];
            }
            return result;
        }
        fn double(borrow x: number): number { return x * 2; }
        let arr = [1, 2, 3];
        let doubled = applyToArray(arr, double);
        doubled[0] + doubled[1] + doubled[2];
    "#;
    assert_eval_number(source, 12.0);
}

#[test]
fn test_filter_pattern_with_function() {
    let source = r#"
        fn filterArray(borrow arr: number[], borrow predicate: (number): bool): number[] {
            let mut result: number[] = [];
            for item in arr {
                if (predicate(item)) {
                    result = result + [item];
                }
            }
            return result;
        }
        fn isEven(borrow x: number): bool { return x % 2 == 0; }
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
            borrow arr: number[],
            borrow reducer: (number, number): number,
            initial: number
        ): number {
            let mut acc = initial;
            for item in arr {
                acc = reducer(acc, item);
            }
            return acc;
        }
        fn add(borrow a: number, borrow b: number): number { return a + b; }
        let arr = [1, 2, 3, 4, 5];
        reduceArray(arr, add, 0);
    "#;
    assert_eval_number(source, 15.0);
}

#[test]
fn test_complex_function_passing() {
    let source = r#"
        fn transform(
            borrow arr: number[],
            borrow f1: (number): number,
            f2: (number): number
        ): number {
            let mut sum = 0;
            for item in arr {
                sum = sum + f1(f2(item));
            }
            return sum;
        }
        fn double(borrow x: number): number { return x * 2; }
        fn square(borrow x: number): number { return x * x; }
        transform([1, 2, 3], double, square);
    "#;
    assert_eval_number(source, 28.0);
}
