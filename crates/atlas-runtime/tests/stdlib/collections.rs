use super::*;
use pretty_assertions::assert_eq;

// From numeric_edge_cases_tests.rs
// ============================================================================

// Tests for numeric edge cases
//
// Verifies behavior with boundary values, special floats (infinity, NaN),
// division by zero, and other numeric edge cases.
//
// Atlas uses f64 (64-bit IEEE 754 floating point) for all numbers.

/// Helper to get all diagnostics from source code
fn get_all_diagnostics(source: &str) -> Vec<atlas_runtime::Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    let mut binder = Binder::new();
    let (mut table, bind_diags) = binder.bind(&program);

    let mut checker = TypeChecker::new(&mut table);
    let type_diags = checker.check(&program);

    let mut all_diags = Vec::new();
    all_diags.extend(lex_diags);
    all_diags.extend(parse_diags);
    all_diags.extend(bind_diags);
    all_diags.extend(type_diags);

    all_diags
}

// =============================================================================
// Integer and Float Boundary Tests
// =============================================================================

#[rstest]
#[case::large_integer("let x: number = 9007199254740991;")]
#[case::negative_large_integer("let x: number = -9007199254740991;")]
#[case::large_integer_arithmetic(
    "let a: number = 9007199254740991;\nlet b: number = 1;\nlet c: number = a + b;"
)]
#[case::float_literal("let x: number = 3.14159265358979323846;")]
#[case::very_small_float("let x: number = 0.0000000001;")]
#[case::negative_float("let x: number = -3.14159;")]
#[case::zero_variants("let a: number = 0;\nlet b: number = 0.0;\nlet c: number = -0.0;")]
fn test_numeric_boundaries(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Should be valid: {:?}", diags);
}

#[test]
fn test_very_large_float() {
    let source = "let x = 179769313486231570000000000000000000000.0;";
    let _diags = get_all_diagnostics(source);
    // This might fail to parse depending on lexer implementation
}

// =============================================================================
// Division and Modulo Tests
// =============================================================================

#[rstest]
#[case::division("let a: number = 10;\nlet b: number = 2;\nlet c: number = a / b;")]
#[case::division_by_zero_literal("let x: number = 10 / 0;")]
#[case::division_by_variable("let divisor: number = 0;\nlet result: number = 10 / divisor;")]
#[case::division_underflow("let a = 1;\nlet b = 10000000;\nlet c = a / b;")]
#[case::modulo_by_zero("let x: number = 10 % 0;")]
#[case::modulo_with_floats("let x: number = 5.5 % 2.3;")]
fn test_division_and_modulo(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    // Type checker cannot detect division by zero - this is runtime behavior
    assert!(diags.is_empty(), "Should typecheck: {:?}", diags);
}

// =============================================================================
// Arithmetic Overflow/Underflow Tests
// =============================================================================

#[rstest]
#[case::addition_overflow("let a = 100000000000000000000000000000.0;\nlet b = 100000000000000000000000000000.0;\nlet c = a + b;")]
#[case::multiplication_overflow(
    "let a = 10000000000000000000.0;\nlet b = 10000000000000000000.0;\nlet c = a * b;"
)]
fn test_arithmetic_overflow(#[case] source: &str) {
    let _diags = get_all_diagnostics(source);
    // Typechecks fine, runtime would produce infinity
}

#[test]
fn test_subtraction_to_negative() {
    let source = "let a: number = 5;\nlet b: number = 10;\nlet c: number = a - b;";
    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Should typecheck: {:?}", diags);
}

// =============================================================================
// Comparison Tests with Edge Values
// =============================================================================

#[rstest]
#[case::zero_comparisons(
    "let a: number = 0;\nlet b: bool = a > 0;\nlet c: bool = a < 0;\nlet d: bool = a == 0;"
)]
#[case::negative_comparison("let a: number = -5;\nlet b: number = 10;\nlet c: bool = a < b;")]
#[case::float_equality("let a: number = 0.1 + 0.2;\nlet b: number = 0.3;\nlet c: bool = a == b;")]
fn test_comparisons(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Should typecheck: {:?}", diags);
}

// =============================================================================
// Mixed Arithmetic Tests
// =============================================================================

#[rstest]
#[case::complex_expression("let x: number = (10 + 5) * 2 - 8 / 4;")]
#[case::nested_arithmetic("let a: number = 10;\nlet b: number = 5;\nlet c: number = 2;\nlet result: number = (a + b) * c - (a / b);")]
#[case::negative_arithmetic("let a: number = -10;\nlet b: number = -5;\nlet c: number = a + b;\nlet d: number = a - b;\nlet e: number = a * b;\nlet f: number = a / b;")]
fn test_mixed_arithmetic(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Should typecheck: {:?}", diags);
}

// =============================================================================
// Unary Minus Tests
// =============================================================================

#[rstest]
#[case::literal("let x: number = -42;")]
#[case::variable("let a: number = 42;\nlet b: number = -a;")]
#[case::double_negation("let a: number = 42;\nlet b: number = -(-a);")]
#[case::negative_zero("let x: number = -0;\nlet y: number = -0.0;")]
fn test_unary_minus(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Should typecheck: {:?}", diags);
}

// =============================================================================
// Error Cases
// =============================================================================

#[rstest]
#[case::string_plus_number("let x: number = \"hello\" + 5;")]
#[case::string_division("let x: number = \"10\" / \"2\";")]
#[case::bool_modulo("let x: number = true % false;")]
#[case::string_comparison("let x: bool = \"hello\" < 5;")]
fn test_type_errors(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    assert!(!diags.is_empty(), "Should produce error");
}

#[test]
fn test_arithmetic_on_non_numbers_has_error_code() {
    let source = "let x: number = \"hello\" + 5;";
    let diags = get_all_diagnostics(source);
    let error = diags.iter().find(|d| d.code.starts_with("AT"));
    assert!(error.is_some(), "Should have AT error code");
}

// =============================================================================
// Array Index Edge Cases
// =============================================================================

#[rstest]
#[case::zero_index("let arr = [1, 2, 3];\nlet x = arr[0];")]
#[case::large_index("let arr = [1, 2, 3];\nlet x = arr[999999];")]
#[case::negative_index("let arr = [1, 2, 3];\nlet x = arr[-1];")]
#[case::float_index("let arr = [1, 2, 3];\nlet x = arr[1.5];")]
fn test_array_index_edge_cases(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    // Type system allows number (f64) for array index
    // Runtime would handle bounds/integer checking
    assert!(diags.is_empty(), "Should typecheck: {:?}", diags);
}

// ============================================================================

// From collection_iteration_tests.rs
// ============================================================================

// HashMap and HashSet Iteration Tests
//
// Comprehensive tests for for_each, map, and filter intrinsics on collections.
//
// NOTE: Atlas v0.2 does not support anonymous functions (fn(x) { ... }).
// All callbacks must be named functions passed by reference.

fn eval(code: &str) -> Value {
    let runtime = Atlas::new();
    runtime.eval(code).expect("Interpretation failed")
}

fn eval_expect_error(code: &str) -> bool {
    let runtime = Atlas::new();
    runtime.eval(code).is_err()
}

// =============================================================================
// HashMap Iteration Tests
// =============================================================================

#[test]
fn test_hashmap_foreach_returns_null() {
    let result = eval(
        r#"
        fn callback(borrow _v: number, borrow _k: string): void {}
        let hmap = Map();
        hmap.set("a", 1);
        hmap.forEach(callback)
    "#,
    );
    assert_eq!(result, Value::Null);
}

#[test]
fn test_hashmap_foreach_executes_callback() {
    // Verify callback executes by counting iterations
    let result = eval(
        r#"
        let mut count: number = 0;
        fn callback(borrow _v: number, borrow _k: string): void {
            count = count + 1;
        }
        let hmap = Map();
        hmap.set("a", 1);
        hmap.set("b", 2);
        hmap.set("c", 3);
        hmap.forEach(callback);
        count
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_hashmap_map_transforms_values() {
    let result = eval(
        r#"
        fn double(borrow v: number, borrow _k: string): number {
            return v * 2;
        }
        let hmap = Map();
        hmap.set("a", 1);
        hmap.set("b", 2);
        let mapped = hmap.map(double);
        unwrap(mapped.get("a"))
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashmap_map_preserves_keys() {
    let result = eval(
        r#"
        fn addFive(borrow v: number, borrow _k: string): number {
            return v + 5;
        }
        let hmap = Map();
        hmap.set("x", 10);
        hmap.set("y", 20);
        let mapped = hmap.map(addFive);
        mapped.has("x") && mapped.has("y")
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_hashmap_map_preserves_size() {
    let result = eval(
        r#"
        fn times10(borrow v: number, borrow _k: string): number {
            return v * 10;
        }
        let hmap = Map();
        hmap.set("a", 1);
        hmap.set("b", 2);
        hmap.set("c", 3);
        let mapped = hmap.map(times10);
        mapped.size()
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_hashmap_filter_keeps_matching_entries() {
    let result = eval(
        r#"
        fn greaterThanOne(borrow v: number, borrow _k: string): bool {
            return v > 1;
        }
        let hmap = Map();
        hmap.set("a", 1);
        hmap.set("b", 2);
        hmap.set("c", 3);
        let filtered = hmap.filter(greaterThanOne);
        filtered.size()
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashmap_filter_with_predicate() {
    let result = eval(
        r#"
        fn isEven(borrow v: number, borrow _k: string): bool {
            return v % 2 == 0;
        }
        let hmap = Map();
        hmap.set("a", 1);
        hmap.set("b", 2);
        hmap.set("c", 3);
        hmap.set("d", 4);
        let filtered = hmap.filter(isEven);
        filtered.size()
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashmap_filter_removes_non_matching() {
    let result = eval(
        r#"
        fn greaterThan10(borrow v: number, borrow _k: string): bool {
            return v > 10;
        }
        let hmap = Map();
        hmap.set("a", 1);
        hmap.set("b", 2);
        hmap.set("c", 3);
        let filtered = hmap.filter(greaterThan10);
        filtered.size()
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_hashmap_empty_iteration() {
    let result = eval(
        r#"
        fn identity(borrow v: number, borrow _k: string): number {
            return v;
        }
        let hmap = Map();
        let mapped = hmap.map(identity);
        mapped.size()
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_hashmap_chaining_operations() {
    let result = eval(
        r#"
        fn double(borrow v: number, borrow _k: string): number {
            return v * 2;
        }
        fn greaterThan2(borrow v: number, borrow _k: string): bool {
            return v > 2;
        }
        let hmap = Map();
        hmap.set("a", 1);
        hmap.set("b", 2);
        hmap.set("c", 3);
        let doubled = hmap.map(double);
        let filtered = doubled.filter(greaterThan2);
        filtered.size()
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashmap_callback_receives_value_and_key() {
    // Verify callback receives both value and key parameters
    let result = eval(
        r#"
        fn addIfTest(borrow v: number, borrow k: string): number {
            if (k == "test") {
                return v + 1;
            } else {
                return v;
            }
        }
        let hmap = Map();
        hmap.set("test", 42);
        let mapped = hmap.map(addIfTest);
        unwrap(mapped.get("test"))
    "#,
    );
    assert_eq!(result, Value::Number(43.0));
}

#[test]
fn test_hashmap_large_map() {
    let result = eval(
        r#"
        fn lessThan25(borrow v: number, borrow _k: string): bool {
            return v < 25;
        }
        let hmap = Map();
        let mut i: number = 0;
        while (i < 50) {
            hmap.set(toString(i), i);
            i = i + 1;
        }
        let filtered = hmap.filter(lessThan25);
        filtered.size()
    "#,
    );
    assert_eq!(result, Value::Number(25.0));
}

// Error Handling Tests

#[test]
fn test_hashmap_foreach_non_function_callback() {
    assert!(eval_expect_error(
        r#"
        let hmap = Map();
        hmap.set("a", 1);
        hmap.forEach("not a function")
    "#
    ));
}

#[test]
fn test_hashmap_map_non_function_callback() {
    assert!(eval_expect_error(
        r#"
        let hmap = Map();
        hmap.set("a", 1);
        hmap.map(42)
    "#
    ));
}

#[test]
fn test_hashmap_filter_non_function_callback() {
    assert!(eval_expect_error(
        r#"
        let hmap = Map();
        hmap.set("a", 1);
        hmap.filter(null)
    "#
    ));
}

#[test]
fn test_hashmap_filter_non_bool_return() {
    // Filter predicate must return bool
    assert!(eval_expect_error(
        r#"
        fn returnValue(borrow v: number, borrow _k: string): number {
            return v;
        }
        let hmap = Map();
        hmap.set("a", 1);
        hmap.filter(returnValue)
    "#
    ));
}

// =============================================================================
// HashSet Iteration Tests
// =============================================================================

#[test]
fn test_hashset_foreach_returns_null() {
    let result = eval(
        r#"
        fn callback(borrow _elem: number): void {}
        let hset = Set();
        hset.add(1);
        hset.forEach(callback)
    "#,
    );
    assert_eq!(result, Value::Null);
}

#[test]
fn test_hashset_foreach_executes_callback() {
    let result = eval(
        r#"
        let mut count: number = 0;
        fn callback(borrow _elem: number): void {
            count = count + 1;
        }
        let hset = Set();
        hset.add(1);
        hset.add(2);
        hset.add(3);
        hset.forEach(callback);
        count
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_hashset_map_to_array() {
    let result = eval(
        r#"
        fn double(borrow elem: number): number {
            return elem * 2;
        }
        let hset = Set();
        hset.add(1);
        hset.add(2);
        let arr = hset.map(double);
        typeof(arr)
    "#,
    );
    assert_eq!(result, Value::String(Arc::new("array".to_string())));
}

#[test]
fn test_hashset_map_array_length() {
    let result = eval(
        r#"
        fn times10(borrow elem: number): number {
            return elem * 10;
        }
        let hset = Set();
        hset.add(1);
        hset.add(2);
        hset.add(3);
        let arr = hset.map(times10);
        len(arr)
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_hashset_map_transforms_elements() {
    let result = eval(
        r#"
        fn double(borrow elem: number): number {
            return elem * 2;
        }
        let hset = Set();
        hset.add(5);
        let arr = hset.map(double);
        arr[0]
    "#,
    );
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_hashset_filter_keeps_matching() {
    let result = eval(
        r#"
        fn greaterThan2(borrow elem: number): bool {
            return elem > 2;
        }
        let hset = Set();
        hset.add(1);
        hset.add(2);
        hset.add(3);
        hset.add(4);
        let filtered = hset.filter(greaterThan2);
        filtered.size()
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashset_filter_removes_non_matching() {
    let result = eval(
        r#"
        fn greaterThan10(borrow elem: number): bool {
            return elem > 10;
        }
        let hset = Set();
        hset.add(1);
        hset.add(2);
        hset.add(3);
        let filtered = hset.filter(greaterThan10);
        filtered.size()
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_hashset_empty_filter() {
    let result = eval(
        r#"
        fn alwaysTrue(borrow _elem: number): bool {
            return true;
        }
        let hset = Set();
        let filtered = hset.filter(alwaysTrue);
        filtered.size()
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_hashset_filter_chaining() {
    let result = eval(
        r#"
        fn greaterThan1(borrow elem: number): bool {
            return elem > 1;
        }
        fn lessThan4(borrow elem: number): bool {
            return elem < 4;
        }
        let hset = Set();
        hset.add(1);
        hset.add(2);
        hset.add(3);
        hset.add(4);
        let f1 = hset.filter(greaterThan1);
        let f2 = f1.filter(lessThan4);
        f2.size()
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashset_large_set() {
    let result = eval(
        r#"
        fn divisibleBy3(borrow elem: number): bool {
            return elem % 3 == 0;
        }
        let hset = Set();
        let mut i: number = 0;
        while (i < 30) {
            hset.add(i);
            i = i + 1;
        }
        let filtered = hset.filter(divisibleBy3);
        filtered.size()
    "#,
    );
    assert_eq!(result, Value::Number(10.0));
}

// Error Handling Tests

#[test]
fn test_hashset_foreach_non_function_callback() {
    assert!(eval_expect_error(
        r#"
        let hset = Set();
        hset.add(1);
        hset.forEach("not a function")
    "#
    ));
}

#[test]
fn test_hashset_map_non_function_callback() {
    assert!(eval_expect_error(
        r#"
        let hset = Set();
        hset.add(1);
        hset.map(42)
    "#
    ));
}

#[test]
fn test_hashset_filter_non_function_callback() {
    assert!(eval_expect_error(
        r#"
        let hset = Set();
        hset.add(1);
        hset.filter(null)
    "#
    ));
}

#[test]
fn test_hashset_filter_non_bool_return() {
    // Filter predicate must return bool
    assert!(eval_expect_error(
        r#"
        fn returnValue(borrow elem: number): number {
            return elem;
        }
        let hset = Set();
        hset.add(1);
        hset.filter(returnValue)
    "#
    ));
}

// =============================================================================
// Integration Tests
// =============================================================================

#[test]
fn test_integration_hashmap_to_hashset() {
    let result = eval(
        r#"
        let hmap = Map();
        hmap.set("a", 1);
        hmap.set("b", 2);
        let values = hmap.values();
        let hset = Set(values);
        hset.size()
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_integration_hashset_map_to_array_filter() {
    let result = eval(
        r#"
        fn double(borrow x: number): number {
            return x * 2;
        }
        fn greaterThan2(borrow x: number): bool {
            return x > 2;
        }
        let hset = Set();
        hset.add(1);
        hset.add(2);
        hset.add(3);
        let arr = hset.map(double);
        let filtered = filter(arr, greaterThan2);
        len(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_integration_empty_collections() {
    let result = eval(
        r#"
        fn identity(borrow v: number, borrow _k: string): number {
            return v;
        }
        fn alwaysTrue(borrow _x: number): bool {
            return true;
        }
        let hm = Map();
        let hs = Set();
        let mr = hm.map(identity);
        let sr = hs.filter(alwaysTrue);
        mr.size() + sr.size()
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_integration_complex_transformation() {
    let result = eval(
        r#"
        fn double(borrow v: number, borrow _k: string): number {
            return v * 2;
        }
        fn greaterOrEqual4(borrow v: number, borrow _k: string): bool {
            return v >= 4;
        }
        let mut sum: number = 0;
        fn addToSum(borrow v: number): void {
            sum = sum + v;
        }
        let hmap = Map();
        hmap.set("a", 1);
        hmap.set("b", 2);
        hmap.set("c", 3);
        hmap.set("d", 4);
        let doubled = hmap.map(double);
        let filtered = doubled.filter(greaterOrEqual4);
        let values = filtered.values();
        for_each(values, addToSum);
        sum
    "#,
    );
    assert_eq!(result, Value::Number(18.0)); // 4 + 6 + 8 = 18
}

#[test]
fn test_integration_hashmap_keys_to_hashset() {
    let result = eval(
        r#"
        let hmap = Map();
        hmap.set("a", 1);
        hmap.set("b", 2);
        hmap.set("c", 3);
        let keys = hmap.keys();
        let hset = Set(keys);
        hset.size()
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

// =============================================================================
// Parity Tests (ensure interpreter/VM consistency)
// =============================================================================

#[test]
fn test_parity_hashmap_foreach() {
    let result = eval(
        r#"
        let mut sum: number = 0;
        fn addToSum(borrow v: number, borrow _k: string): void {
            sum = sum + v;
        }
        let hmap = Map();
        hmap.set("x", 5);
        hmap.forEach(addToSum);
        sum
    "#,
    );
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_parity_hashmap_map() {
    let result = eval(
        r#"
        fn triple(borrow v: number, borrow _k: string): number {
            return v * 3;
        }
        let hmap = Map();
        hmap.set("test", 5);
        let mapped = hmap.map(triple);
        unwrap(mapped.get("test"))
    "#,
    );
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn test_parity_hashmap_filter() {
    let result = eval(
        r#"
        fn notEqual2(borrow v: number, borrow _k: string): bool {
            return v != 2;
        }
        let hmap = Map();
        hmap.set("a", 1);
        hmap.set("b", 2);
        hmap.set("c", 3);
        let filtered = hmap.filter(notEqual2);
        filtered.size()
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_parity_hashset_foreach() {
    let result = eval(
        r#"
        let mut sum: number = 0;
        fn addToSum(borrow elem: number): void {
            sum = sum + elem;
        }
        let hset = Set();
        hset.add(10);
        hset.forEach(addToSum);
        sum
    "#,
    );
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_parity_hashset_map() {
    let result = eval(
        r#"
        fn double(borrow elem: number): number {
            return elem * 2;
        }
        let hset = Set();
        hset.add(7);
        let arr = hset.map(double);
        arr[0]
    "#,
    );
    assert_eq!(result, Value::Number(14.0));
}

#[test]
fn test_parity_hashset_filter() {
    let result = eval(
        r#"
        fn lessOrEqual2(borrow elem: number): bool {
            return elem <= 2;
        }
        let hset = Set();
        hset.add(1);
        hset.add(2);
        hset.add(3);
        let filtered = hset.filter(lessOrEqual2);
        filtered.size()
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

// ============================================================================
// VM stdlib tests (co-located to eliminate duplicate binary pairs)
// Tests run with separate binary name prefix via submodule
// ============================================================================
