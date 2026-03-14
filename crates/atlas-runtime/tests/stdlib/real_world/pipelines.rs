use super::super::*;
// ============================================================================

#[test]
fn test_pipeline_map_filter_reduce() {
    let code = r#"
        fn double(borrow x: number): number { return x * 2.0; }
        fn isEven(borrow x: number): bool { return x % 2.0 == 0.0; }
        fn sum(borrow a: number, borrow b: number): number { return a + b; }

        let numbers: number[] = [1.0, 2.0, 3.0, 4.0, 5.0];
        let doubled: number[] = map(numbers, double);
        let evens: number[] = filter(doubled, isEven);
        reduce(evens, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 30.0); // doubled=[2,4,6,8,10], all even, sum=30
}

#[test]
fn test_pipeline_filter_map_join() {
    let code = r#"
        fn isLong(borrow s: string): bool { return len(s) > 3.0; }
        fn toUpper(borrow s: string): string { return s.toUpperCase(); }

        let words: string[] = ["hi", "hello", "bye", "world"];
        let long: string[] = filter(words, isLong);
        let uppered: string[] = map(long, toUpper);
        uppered.join("-")
    "#;
    assert_eval_string_with_io(code, "HELLO-WORLD");
}

#[test]
fn test_pipeline_nested_arrays() {
    let code = r#"
        let nested: number[][] = [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]];
        let flat: number[] = nested.flatten();
        fn double(borrow x: number): number { return x * 2.0; }
        let doubled: number[] = map(flat, double);
        fn sum(borrow a: number, borrow b: number): number { return a + b; }
        reduce(doubled, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 42.0); // [1..6] doubled = [2,4,6,8,10,12] sum=42
}

#[test]
fn test_pipeline_string_processing() {
    let code = r#"
        fn trimAndLower(borrow s: string): string {
            let t: string = s.trim();
            return t.toLowerCase();
        }

        let input: string[] = ["  HELLO  ", "  WORLD  ", "  TEST  "];
        let cleaned: string[] = map(input, trimAndLower);
        cleaned.join(",")
    "#;
    assert_eval_string_with_io(code, "hello,world,test");
}

#[test]
fn test_pipeline_multi_step_filter() {
    let code = r#"
        fn isPositive(borrow x: number): bool { return x > 0.0; }
        fn isSmall(borrow x: number): bool { return x < 100.0; }

        let numbers: number[] = [-5.0, 10.0, 150.0, 50.0, -20.0, 75.0];
        let positive: number[] = filter(numbers, isPositive);
        let small: number[] = filter(positive, isSmall);
        len(small)
    "#;
    assert_eval_number_with_io(code, 3.0); // [10, 50, 75]
}

#[test]
fn test_pipeline_sort_and_slice() {
    let code = r#"
        fn compare(borrow a: number, borrow b: number): number { return a - b; }

        let numbers: number[] = [5.0, 2.0, 8.0, 1.0, 9.0, 3.0];
        let sorted: number[] = sort(numbers, compare);
        let top3: number[] = sorted.slice(0.0, 3.0);
        fn sum(borrow a: number, borrow b: number): number { return a + b; }
        reduce(top3, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 6.0); // [1,2,3] sum=6
}

#[test]
fn test_pipeline_flatmap_strings() {
    let code = r#"
        fn splitWords(borrow s: string): string[] {
            return s.split(" ");
        }

        let sentences: string[] = ["hello world", "foo bar"];
        let words: string[] = flat_map(sentences, splitWords);
        len(words)
    "#;
    assert_eval_number_with_io(code, 4.0);
}

#[test]
fn test_pipeline_conditional_transform() {
    let code = r#"
        fn transform(borrow x: number): number {
            if (x < 0.0) {
                return Math.abs(x);
            }
            return x;
        }

        let numbers: number[] = [-5.0, 10.0, -3.0, 7.0];
        let transformed: number[] = map(numbers, transform);
        fn sum(borrow a: number, borrow b: number): number { return a + b; }
        reduce(transformed, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 25.0); // [5,10,3,7] sum=25
}

#[test]
fn test_pipeline_find_and_transform() {
    let code = r#"
        fn isLarge(borrow x: number): bool { return x > 50.0; }

        let numbers: number[] = [10.0, 60.0, 30.0, 80.0];
        let found: number = unwrap(find(numbers, isLarge));
        found * 2.0
    "#;
    assert_eval_number_with_io(code, 120.0); // 60 * 2
}

#[test]
fn test_pipeline_every_and_some() {
    let code = r#"
        fn isPositive(borrow x: number): bool { return x > 0.0; }

        let numbers: number[] = [1.0, 2.0, 3.0];
        let allPositive: bool = every(numbers, isPositive);
        let somePositive: bool = some(numbers, isPositive);
        allPositive && somePositive
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_pipeline_reverse_and_join() {
    let code = r#"
        let words: string[] = ["one", "two", "three"];
        let reversed: string[] = words.reverse();
        reversed.join("-")
    "#;
    assert_eval_string_with_io(code, "three-two-one");
}

#[test]
fn test_pipeline_unshift_and_concat() {
    let code = r#"
        let arr1: number[] = [2.0, 3.0];
        let arr2: number[] = [4.0, 5.0];
        let withOne: number[] = arr1.unshift(1.0);
        let combined: number[] = withOne.concat(arr2);
        len(combined)
    "#;
    assert_eval_number_with_io(code, 5.0);
}

#[test]
fn test_pipeline_multiple_maps() {
    let code = r#"
        fn add10(borrow x: number): number { return x + 10.0; }
        fn double(borrow x: number): number { return x * 2.0; }

        let numbers: number[] = [1.0, 2.0, 3.0];
        let step1: number[] = map(numbers, add10);
        let step2: number[] = map(step1, double);
        step2[0]
    "#;
    assert_eval_number_with_io(code, 22.0); // (1+10)*2 = 22
}

#[test]
fn test_pipeline_filter_reverse_first() {
    let code = r#"
        fn isEven(borrow x: number): bool { return x % 2.0 == 0.0; }

        let numbers: number[] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let evens: number[] = filter(numbers, isEven);
        let reversed: number[] = evens.reverse();
        reversed[0]
    "#;
    assert_eval_number_with_io(code, 6.0);
}

#[test]
fn test_pipeline_sortby_number() {
    let code = r#"
        fn negate(borrow x: number): number { return x * -1.0; }

        let numbers: number[] = [3.0, 1.0, 4.0, 1.0, 5.0];
        let sorted: number[] = sort_by(numbers, negate);
        sorted[0]
    "#;
    assert_eval_number_with_io(code, 5.0); // sorted descending
}

#[test]
fn test_pipeline_pop_and_process() {
    let code = r#"
        let numbers: number[] = [1.0, 2.0, 3.0];
        let last: number = numbers[len(numbers) - 1.0];
        let remaining: number[] = numbers.slice(0.0, len(numbers) - 1.0);
        last + len(remaining)
    "#;
    assert_eval_number_with_io(code, 5.0); // 3 + 2
}

#[test]
fn test_pipeline_shift_and_process() {
    let code = r#"
        let numbers: number[] = [1.0, 2.0, 3.0];
        let first: number = numbers[0];
        let remaining: number[] = numbers.slice(1.0, len(numbers));
        first + len(remaining)
    "#;
    assert_eval_number_with_io(code, 3.0); // 1 + 2
}

#[test]
fn test_pipeline_findindex_and_slice() {
    let code = r#"
        fn isLarge(borrow x: number): bool { return x > 50.0; }

        let numbers: number[] = [10.0, 20.0, 60.0, 80.0];
        let idx: number = unwrap(find_index(numbers, isLarge));
        let fromLarge: number[] = numbers.slice(idx, len(numbers));
        len(fromLarge)
    "#;
    assert_eval_number_with_io(code, 2.0); // [60, 80]
}

#[test]
fn test_pipeline_complex_aggregation() {
    let code = r#"
        fn square(borrow x: number): number { return x * x; }
        fn sum(borrow a: number, borrow b: number): number { return a + b; }

        let numbers: number[] = [1.0, 2.0, 3.0, 4.0];
        let squared: number[] = map(numbers, square);
        let total: number = reduce(squared, sum, 0.0);
        total
    "#;
    assert_eval_number_with_io(code, 30.0); // 1+4+9+16 = 30
}

#[test]
fn test_pipeline_string_filter_map() {
    let code = r#"
        fn notEmpty(borrow s: string): bool { return len(s) > 0.0; }
        fn firstChar(borrow s: string): string { return unwrap(s.charAt(0.0)); }

        let words: string[] = ["apple", "", "banana", "", "cherry"];
        let nonEmpty: string[] = filter(words, notEmpty);
        let firstChars: string[] = map(nonEmpty, firstChar);
        firstChars.join("")
    "#;
    assert_eval_string_with_io(code, "abc");
}

#[test]
fn test_pipeline_nested_operations() {
    let code = r#"
        fn process(borrow x: number): number {
            let step1: number = x + 5.0;
            let step2: number = step1 * 2.0;
            return step2;
        }
        fn sum(borrow a: number, borrow b: number): number { return a + b; }

        let numbers: number[] = [1.0, 2.0, 3.0];
        let processed: number[] = map(numbers, process);
        reduce(processed, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 42.0); // (1+5)*2=12, (2+5)*2=14, (3+5)*2=16, sum=42
}

#[test]
fn test_pipeline_includes_filter() {
    let code = r#"
        fn hasLetterA(borrow s: string): bool {
            return s.includes("a");
        }

        let words: string[] = ["apple", "berry", "apricot", "cherry"];
        let withA: string[] = filter(words, hasLetterA);
        len(withA)
    "#;
    assert_eval_number_with_io(code, 2.0); // apple, apricot
}

#[test]
fn test_pipeline_index_access_transform() {
    let code = r#"
        let matrix: number[][] = [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]];
        fn getFirst(borrow row: number[]): number { return row[0]; }

        let firstElements: number[] = map(matrix, getFirst);
        fn sum(borrow a: number, borrow b: number): number { return a + b; }
        reduce(firstElements, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 9.0); // 1+3+5 = 9
}

#[test]
fn test_pipeline_replace_map() {
    let code = r#"
        fn removeSpaces(borrow s: string): string {
            return s.replace(" ", "_");
        }

        let phrases: string[] = ["hello world", "foo bar"];
        let replaced: string[] = map(phrases, removeSpaces);
        replaced.join("|")
    "#;
    assert_eval_string_with_io(code, "hello_world|foo_bar");
}

#[test]
fn test_pipeline_padstart_map() {
    let code = r#"
        fn pad(borrow s: string): string {
            return s.padStart(5.0, "0");
        }

        let numbers: string[] = ["1", "22", "333"];
        let padded: string[] = map(numbers, pad);
        padded.join(",")
    "#;
    assert_eval_string_with_io(code, "00001,00022,00333");
}

#[test]
fn test_pipeline_substring_filter_map() {
    let code = r#"
        fn getPrefix(borrow s: string): string {
            return s.substring(0.0, 3.0);
        }

        let words: string[] = ["apple", "application", "appropriate"];
        let prefixes: string[] = map(words, getPrefix);
        fn isApp(borrow s: string): bool { return s == "app"; }
        let appPrefixes: string[] = filter(prefixes, isApp);
        len(appPrefixes)
    "#;
    assert_eval_number_with_io(code, 3.0);
}

#[test]
fn test_pipeline_min_max_aggregation() {
    let code = r#"
        fn findMin(borrow current: number, borrow x: number): number {
            if (current == 0.0) { return x; }
            return Math.min(current, x);
        }
        fn findMax(borrow current: number, borrow x: number): number {
            return Math.max(current, x);
        }

        let numbers: number[] = [5.0, 2.0, 8.0, 1.0, 9.0];
        let minVal: number = reduce(numbers, findMin, 0.0);
        let maxVal: number = reduce(numbers, findMax, 0.0);
        maxVal - minVal
    "#;
    assert_eval_number_with_io(code, 8.0); // 9 - 1
}

#[test]
fn test_pipeline_array_building() {
    let code = r#"
        let arr1: number[] = [1.0];
        let arr2: number[] = arr1.unshift(0.0);
        let arr3: number[] = arr2.concat([2.0, 3.0]);
        fn sum(borrow a: number, borrow b: number): number { return a + b; }
        reduce(arr3, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 6.0); // [0,1,2,3] sum=6
}

#[test]
fn test_pipeline_foreach_side_effects() {
    let code = r#"
        fn noop(borrow _x: number): void { return; }

        let numbers: number[] = [1.0, 2.0, 3.0];
        for_each(numbers, noop);
        // for_each returns null, verify it doesn't crash
        true
    "#;
    assert_eval_bool_with_io(code, true);
}
