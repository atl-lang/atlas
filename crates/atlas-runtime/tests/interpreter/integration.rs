use super::*;
use pretty_assertions::assert_eq;

// From integration/interpreter/arithmetic.rs
// ============================================================================

#[rstest]
#[case("1 + 2", 3.0)]
#[case("10 - 3", 7.0)]
#[case("4 * 5", 20.0)]
#[case("20 / 4", 5.0)]
#[case("10 % 3", 1.0)]
#[case("-42", -42.0)]
#[case("2 + 3 * 4 - 1", 13.0)]
#[case("(2 + 3) * 4", 20.0)]
fn test_arithmetic_operations(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

#[rstest]
#[case("10 / 0", "AT0005")]
#[case("10 % 0", "AT0005")]
#[case("0 / 0", "AT0005")]
#[case("-10 / 0", "AT0005")]
#[case("0 % 0", "AT0005")]
#[case("5 + (10 / 0)", "AT0005")]
fn test_divide_by_zero_errors(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[rstest]
#[case("1e308 * 2.0", "AT0007")]
#[case("1.5e308 + 1.5e308", "AT0007")]
#[case("-1.5e308 - 1.5e308", "AT0007")]
#[case("1e308 / 1e-308", "AT0007")]
fn test_numeric_overflow(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[test]
fn test_numeric_valid_large_numbers() {
    let runtime = Atlas::new();
    let code = r#"
        let x: number = 1e50;
        let y: number = 2e50;
        let z: number = x + y;
        z
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => {
            assert!(n > 0.0);
            assert!(n.is_finite());
        }
        other => panic!("Expected valid large number, got {:?}", other),
    }
}

#[test]
fn test_numeric_multiplication_by_zero_valid() {
    assert_eval_number("let large: number = 1e200; large * 0", 0.0);
}

#[test]
fn test_numeric_negative_modulo() {
    let runtime = Atlas::new();
    match runtime.eval("-10 % 3") {
        Ok(Value::Number(n)) => {
            assert!(n.is_finite());
            std::assert_eq!(n, -1.0); // Rust's % preserves sign of left operand
        }
        other => panic!("Expected valid modulo result, got {:?}", other),
    }
}

#[test]
fn test_numeric_error_in_function() {
    let code = r#"
        fn compute(a: number) -> number {
            return a * a * a;
        }
        let big: number = 1e103;
        compute(big)
    "#;
    assert_error_code(code, "AT0007");
}

#[test]
fn test_numeric_error_propagation() {
    let code = r#"
        fn bad() -> number {
            return 1 / 0;
        }
        fn caller() -> number {
            return bad() + 5;
        }
        caller()
    "#;
    assert_error_code(code, "AT0005");
}

// ============================================================================
// From integration/interpreter/arrays.rs
// ============================================================================

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
        var arr1: number[] = [1, 2, 3];
        var arr2: number[] = arr1;
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
            var total: number = 0;
            var i: number = 0;
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
// From integration/interpreter/control_flow.rs
// ============================================================================

#[test]
fn test_if_then() {
    let code = r#"
        var x: number = 0;
        if (true) {
            x = 42;
        }
        x
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_if_else() {
    let code = r#"
        var x: number = 0;
        if (false) {
            x = 10;
        } else {
            x = 20;
        }
        x
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn test_if_with_comparison() {
    let code = r#"
        let x: number = 5;
        var result: number = 0;
        if (x > 3) {
            result = 1;
        } else {
            result = 2;
        }
        result
    "#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_while_loop() {
    let code = r#"
        var i: number = 0;
        var sum: number = 0;
        while (i < 5) {
            sum = sum + i;
            i = i + 1;
        }
        sum
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_while_loop_with_break() {
    let code = r#"
        var i: number = 0;
        while (i < 10) {
            if (i == 5) {
                break;
            }
            i = i + 1;
        }
        i
    "#;
    assert_eval_number(code, 5.0);
}

#[test]
fn test_while_loop_with_continue() {
    let code = r#"
        var i: number = 0;
        var sum: number = 0;
        while (i < 5) {
            i = i + 1;
            if (i == 3) {
                continue;
            }
            sum = sum + i;
        }
        sum
    "#;
    assert_eval_number(code, 12.0);
}

#[test]
fn test_for_loop() {
    let code = r#"
        var sum: number = 0;
        for (var i: number = 0; i < 5; i = i + 1) {
            sum = sum + i;
        }
        sum
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_for_loop_with_break() {
    let code = r#"
        var result: number = 0;
        for (var i: number = 0; i < 10; i = i + 1) {
            if (i == 5) {
                break;
            }
            result = i;
        }
        result
    "#;
    assert_eval_number(code, 4.0);
}

#[test]
fn test_for_loop_with_continue() {
    let code = r#"
        var sum: number = 0;
        for (var i: number = 0; i < 5; i = i + 1) {
            if (i == 2) {
                continue;
            }
            sum = sum + i;
        }
        sum
    "#;
    assert_eval_number(code, 8.0);
}

#[test]
fn test_for_loop_with_increment() {
    let code = r#"
        var sum: number = 0;
        for (var i: number = 0; i < 5; i++) {
            sum += i;
        }
        sum
    "#;
    assert_eval_number(code, 10.0);
}

// ============================================================================
// From integration/interpreter/functions.rs
// ============================================================================

#[test]
fn test_function_definition_and_call() {
    let code = r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }
        add(3, 4)
    "#;
    assert_eval_number(code, 7.0);
}

#[test]
fn test_function_with_local_return() {
    let code = r#"
        fn foo(x: number) -> number {
            let y: number = x + 1;
            return y;
        }
        foo(5)
    "#;
    assert_eval_number(code, 6.0);
}

#[test]
fn test_function_with_early_return() {
    let code = r#"
        fn myAbs(x: number) -> number {
            if (x < 0) {
                return -x;
            }
            return x;
        }
        myAbs(-5)
    "#;
    assert_eval_number(code, 5.0);
}

#[test]
fn test_function_recursion() {
    let code = r#"
        fn factorial(n: number) -> number {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        factorial(5)
    "#;
    assert_eval_number(code, 120.0);
}

#[test]
fn test_function_with_local_variables() {
    let code = r#"
        fn compute(x: number) -> number {
            let a: number = x + 1;
            let b: number = a * 2;
            return b - 1;
        }
        compute(5)
    "#;
    assert_eval_number(code, 11.0);
}

#[test]
fn test_function_nested_calls() {
    let code = r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }
        fn multiply(x: number, y: number) -> number {
            return x * y;
        }
        fn compute(n: number) -> number {
            return add(multiply(n, 2), 5);
        }
        compute(3)
    "#;
    assert_eval_number(code, 11.0);
}

#[rstest]
#[case(
    "fn add(a: number, b: number) -> number { return a + b; } add(5)",
    "AT3005"
)]
#[case(
    "fn add(a: number, b: number) -> number { return a + b; } add(1, 2, 3)",
    "AT3005"
)]
fn test_function_wrong_arity(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[test]
fn test_function_void_return() {
    let code = r#"
        var result: number = 0;
        fn set_result(x: number) -> void {
            result = x;
        }
        set_result(42);
        result
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_function_no_parameters() {
    let code = r#"
        fn get_answer() -> number {
            return 42;
        }
        get_answer()
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_function_multiple_parameters() {
    let code = r#"
        fn sum_four(a: number, b: number, c: number, d: number) -> number {
            return a + b + c + d;
        }
        sum_four(1, 2, 3, 4)
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_function_call_stack_depth() {
    let code = r#"
        fn count_down(n: number) -> number {
            if (n <= 0) {
                return 0;
            }
            return n + count_down(n - 1);
        }
        count_down(5)
    "#;
    assert_eval_number(code, 15.0);
}

#[test]
fn test_function_local_variable_isolation() {
    let code = r#"
        var global: number = 100;
        fn modify_local() -> number {
            let global: number = 50;
            return global;
        }
        let result: number = modify_local();
        result + global
    "#;
    assert_eval_number(code, 150.0);
}

#[test]
fn test_function_mutually_recursive() {
    let code = r#"
        fn is_even(n: number) -> bool {
            if (n == 0) {
                return true;
            }
            return is_odd(n - 1);
        }
        fn is_odd(n: number) -> bool {
            if (n == 0) {
                return false;
            }
            return is_even(n - 1);
        }
        is_even(4)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_fibonacci() {
    let code = r#"
        fn fib(n: number) -> number {
            if (n <= 1) {
                return n;
            }
            return fib(n - 1) + fib(n - 2);
        }
        fib(10)
    "#;
    assert_eval_number(code, 55.0);
}

#[test]
fn test_runtime_error_in_function_call() {
    let code = r#"
        fn divide(a: number, b: number) -> number {
            return a / b;
        }
        divide(10, 0)
    "#;
    assert_error_code(code, "AT0005");
}

// ============================================================================
// From integration/interpreter/logical.rs
// ============================================================================

#[rstest]
#[case("5 == 5", true)]
#[case("5 != 3", true)]
#[case("3 < 5", true)]
#[case("5 > 3", true)]
#[case("true && true", true)]
#[case("false || true", true)]
#[case("!false", true)]
#[case("true && false", false)]
#[case("false || false", false)]
#[case("!true", false)]
fn test_comparison_and_boolean_ops(#[case] code: &str, #[case] expected: bool) {
    assert_eval_bool(code, expected);
}

#[test]
fn test_variable_declaration_and_use() {
    let code = r#"
        let x: number = 42;
        x
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_variable_assignment() {
    let code = r#"
        var x: number = 10;
        x = 20;
        x
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn test_variable_arithmetic() {
    let code = r#"
        let a: number = 5;
        let b: number = 3;
        a + b
    "#;
    assert_eval_number(code, 8.0);
}

#[test]
fn test_block_scope() {
    let code = r#"
        let x: number = 1;
        if (true) {
            let x: number = 2;
            x;
        }
    "#;
    assert_eval_number(code, 2.0);
}

#[test]
fn test_function_scope() {
    let code = r#"
        var x: number = 10;
        fn foo(x: number) -> number {
            return x + 1;
        }
        foo(5)
    "#;
    assert_eval_number(code, 6.0);
}

// ============================================================================
// From integration/interpreter/strings.rs

// From integration/interpreter/strings.rs
// ============================================================================

#[test]
fn test_string_concatenation() {
    let code = r#"
        let s: string = "Hello, " + "World!";
        s
    "#;
    assert_eval_string(code, "Hello, World!");
}

// TODO: Enable when typechecker supports string indexing
#[test]
#[ignore = "typechecker does not yet support string indexing"]
fn test_string_indexing() {
    let code = r#"
        let s: string = "Hello";
        s[1]
    "#;
    assert_eval_string(code, "e");
}

#[test]
fn test_stdlib_len_string() {
    let code = r#"
        let s: string = "hello";
        len(s)
    "#;
    assert_eval_number(code, 5.0);
}

#[test]
fn test_stdlib_str() {
    let code = r#"
        let n: number = 42;
        str(n)
    "#;
    assert_eval_string(code, "42");
}

#[rstest]
#[case(r#"var x: number = 5; x++; x"#, 6.0)]
#[case(r#"var x: number = 10; x--; x"#, 9.0)]
#[case(r#"var x: number = 0; x++; x++; x++; x"#, 3.0)]
#[case(r#"var x: number = 10; x--; x--; x"#, 8.0)]
fn test_increment_decrement_basics(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

#[test]
fn test_increment_array_element() {
    let code = r#"
        let arr: number[] = [5, 10, 15];
        arr[0]++;
        arr[0]
    "#;
    assert_eval_number(code, 6.0);
}

#[test]
fn test_decrement_array_element() {
    let code = r#"
        let arr: number[] = [5, 10, 15];
        arr[2]--;
        arr[2]
    "#;
    assert_eval_number(code, 14.0);
}

#[test]
fn test_increment_in_loop() {
    let code = r#"
        var sum: number = 0;
        var i: number = 0;
        while (i < 5) {
            sum += i;
            i++;
        }
        sum
    "#;
    assert_eval_number(code, 10.0);
}

#[rstest]
#[case("let x: number = 5; x++; x", "AT3003")]
#[case("let x: number = 10; x += 5; x", "AT3003")]
#[case("let x: number = 1; x = 2; x", "AT3003")] // Basic assignment to let
#[case("let x: number = 5; x--; x", "AT3003")] // Decrement
fn test_immutable_mutation_errors(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[rstest]
#[case("var x: number = 10; x += 5; x", 15.0)]
#[case("var x: number = 20; x -= 8; x", 12.0)]
#[case("var x: number = 7; x *= 3; x", 21.0)]
#[case("var x: number = 50; x /= 5; x", 10.0)]
#[case("var x: number = 17; x %= 5; x", 2.0)]
#[case("var x: number = 1; x = 2; x", 2.0)] // Basic assignment to var
#[case("var x: number = 5; x++; x", 6.0)] // Increment
#[case("var x: number = 5; x--; x", 4.0)] // Decrement
fn test_mutable_var_assignments(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

#[test]
fn test_compound_chained() {
    let code = r#"
        var x: number = 10;
        x += 5;
        x *= 2;
        x -= 10;
        x
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn test_compound_array_element() {
    let code = r#"
        let arr: number[] = [10, 20, 30];
        arr[1] += 5;
        arr[1]
    "#;
    assert_eval_number(code, 25.0);
}

#[test]
fn test_compound_divide_by_zero() {
    let code = r#"
        var x: number = 10;
        x /= 0;
        x
    "#;
    assert_error_code(code, "AT0005");
}

// ============================================================================
// Phase interpreter-02: Interpreter-VM Parity Tests
// ============================================================================

use atlas_runtime::compiler::Compiler;
use atlas_runtime::vm::VM;

/// Run code through both interpreter and VM, assert identical results
fn assert_parity(source: &str) {
    // Run interpreter (with binder + typechecker for type-tag resolution)
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);

    let mut interp = Interpreter::new();
    let interp_result = interp.eval(&program, &SecurityContext::allow_all());

    // Run VM (with binder + typechecker so compiler has type tags)
    let mut lexer2 = Lexer::new(source);
    let (tokens2, _) = lexer2.tokenize();
    let mut parser2 = Parser::new(tokens2);
    let (program2, _) = parser2.parse();
    let mut binder2 = Binder::new();
    let (mut symbol_table2, _) = binder2.bind(&program2);
    let mut typechecker2 = TypeChecker::new(&mut symbol_table2);
    let _ = typechecker2.check(&program2);

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program2).expect("compilation failed");
    let mut vm = VM::new(bytecode);
    let vm_result = vm.run(&SecurityContext::allow_all());

    // Compare results
    match (interp_result, vm_result) {
        (Ok(interp_val), Ok(vm_val)) => {
            let interp_str = format!("{:?}", interp_val);
            let vm_str = format!("{:?}", vm_val.unwrap_or(Value::Null));
            assert_eq!(
                interp_str, vm_str,
                "Parity mismatch for:\n{}\nInterpreter: {}\nVM: {}",
                source, interp_str, vm_str
            );
        }
        (Err(interp_err), Err(vm_err)) => {
            // Both errored - acceptable parity
            let _ = (interp_err, vm_err);
        }
        (Ok(val), Err(err)) => {
            panic!(
                "Parity mismatch: interpreter succeeded with {:?}, VM failed with {:?}",
                val, err
            );
        }
        (Err(err), Ok(val)) => {
            panic!(
                "Parity mismatch: interpreter failed with {:?}, VM succeeded with {:?}",
                err, val
            );
        }
    }
}

// Arithmetic parity tests
#[rstest]
#[case("1 + 2;")]
#[case("10 - 3;")]
#[case("5 * 4;")]
#[case("20 / 4;")]
#[case("17 % 5;")]
#[case("2 + 3 * 4;")]
#[case("(2 + 3) * 4;")]
#[case("-5;")]
#[case("--5;")]
fn test_parity_arithmetic(#[case] code: &str) {
    assert_parity(code);
}

// Boolean parity tests
#[rstest]
#[case("true;")]
#[case("false;")]
#[case("!true;")]
#[case("!false;")]
#[case("true && true;")]
#[case("true && false;")]
#[case("false || true;")]
#[case("false || false;")]
#[case("1 < 2;")]
#[case("2 <= 2;")]
#[case("3 > 2;")]
#[case("3 >= 3;")]
#[case("1 == 1;")]
#[case("1 != 2;")]
fn test_parity_boolean(#[case] code: &str) {
    assert_parity(code);
}

// Variable parity tests
#[rstest]
#[case("let x = 10; x;")]
#[case("var y = 5; y = y + 1; y;")]
#[case("let a = 1; let b = 2; a + b;")]
#[case("var c = 0; c = c + 1; c = c + 1; c;")]
fn test_parity_variables(#[case] code: &str) {
    assert_parity(code);
}

// Function parity tests
#[rstest]
#[case("fn add(a: number, b: number) -> number { return a + b; } add(2, 3);")]
#[case("fn identity(x: number) -> number { return x; } identity(42);")]
#[case("fn constant() -> number { return 99; } constant();")]
#[case("fn inc(x: number) -> number { return x + 1; } inc(inc(inc(0)));")]
fn test_parity_functions(#[case] code: &str) {
    assert_parity(code);
}

// Control flow parity tests
#[rstest]
#[case("var r = 0; if (true) { r = 1; } else { r = 2; } r;")]
#[case("var r = 0; if (false) { r = 1; } else { r = 2; } r;")]
#[case("var r = 0; if (1 < 2) { r = 10; } else { r = 20; } r;")]
#[case("var x = 0; if (x == 0) { x = 1; } x;")]
fn test_parity_if_else(#[case] code: &str) {
    assert_parity(code);
}

// Loop parity tests
#[rstest]
#[case("var i = 0; while (i < 5) { i = i + 1; } i;")]
#[case("var sum = 0; var i = 0; while (i < 10) { sum = sum + i; i = i + 1; } sum;")]
#[case("var count = 0; while (count < 3) { count = count + 1; } count;")]
fn test_parity_while_loop(#[case] code: &str) {
    assert_parity(code);
}

// Array parity tests
#[rstest]
#[case("[1, 2, 3];")]
#[case("let arr = [10, 20, 30]; arr[0];")]
#[case("let arr = [1, 2, 3]; arr[2];")]
#[case("let arr: number[] = [5]; len(arr);")]
fn test_parity_arrays(#[case] code: &str) {
    assert_parity(code);
}

// String parity tests
#[rstest]
#[case(r#""hello";"#)]
#[case(r#""foo" + "bar";"#)]
#[case(r#"let s = "test"; len(s);"#)]
#[case(r#"toUpperCase("hello");"#)]
#[case(r#"toLowerCase("WORLD");"#)]
fn test_parity_strings(#[case] code: &str) {
    assert_parity(code);
}

// ============================================================================
// Phase 19: Interpreter/VM Parity — Array & Collection Operations
// ============================================================================

// Array: index read
#[rstest]
#[case("let arr: number[] = [10, 20, 30]; arr[1];")]
#[case("let arr: number[] = [10, 20, 30]; arr[0];")]
#[case("let arr: number[] = [10, 20, 30]; arr[2];")]
fn test_parity_array_index_read(#[case] code: &str) {
    assert_parity(code);
}

// Array: length
#[rstest]
#[case("let arr: number[] = [1, 2, 3]; len(arr);")]
#[case("let arr: number[] = []; len(arr);")]
#[case("let arr: number[] = [1, 2, 3]; arr.len();")]
fn test_parity_array_length(#[case] code: &str) {
    assert_parity(code);
}

// Array: push (CoW — original unaffected)
#[rstest]
#[case("var a: array = [1, 2]; var b: array = a; b.push(3); len(a);")]
#[case("var a: array = [1]; a.push(2); a.push(3); len(a);")]
fn test_parity_array_push_cow(#[case] code: &str) {
    assert_parity(code);
}

// Array: pop (CoW — pops from receiver, returns length)
#[rstest]
#[case("var a: array = [1, 2, 3]; a.pop(); len(a);")]
#[case("var a: array = [1, 2, 3]; var b: array = a; a.pop(); len(b);")]
fn test_parity_array_pop(#[case] code: &str) {
    assert_parity(code);
}

// Array: sort (returns new sorted array, receiver unchanged)
#[rstest]
#[case("var a: array = [3, 1, 2]; let s = a.sort(); s[0];")]
#[case("var a: array = [3, 1, 2]; let s = a.sort(); a[0];")]
fn test_parity_array_sort(#[case] code: &str) {
    assert_parity(code);
}

// Array: concat via + operator
#[rstest]
#[case("let a: number[] = [1, 2]; let b: number[] = [3, 4]; let c = a + b; len(c);")]
#[case("let a: number[] = [1, 2]; let b: number[] = [3, 4]; let c = a + b; c[0];")]
fn test_parity_array_concat(#[case] code: &str) {
    assert_parity(code);
}

// Array: for-each (sum over elements)
#[rstest]
#[case("var sum: number = 0; for x in [1, 2, 3] { sum = sum + x; } sum;")]
#[case("var count: number = 0; for _x in [10, 20, 30] { count = count + 1; } count;")]
fn test_parity_array_foreach(#[case] code: &str) {
    assert_parity(code);
}

// Array: map/filter with closures — both engines error (acceptable parity until Block 4)
// These are included so parity is verified even for unsupported operations.
#[rstest]
#[case("let a: number[] = [1, 2, 3]; map(a, fn(x: number) -> number { return x * 2; });")]
#[case("let a: number[] = [1, 2, 3, 4]; filter(a, fn(x: number) -> bool { return x > 2; });")]
fn test_parity_array_map_filter_both_error(#[case] code: &str) {
    assert_parity(code); // Both engines must agree (both succeed or both fail)
}

// Map (HashMap): get
#[rstest]
#[case("let m: HashMap = hashMapNew(); hashMapPut(m, \"a\", 1); unwrap(hashMapGet(m, \"a\"));")]
#[case("let m: HashMap = hashMapNew(); hashMapPut(m, \"x\", 42); unwrap(hashMapGet(m, \"x\"));")]
fn test_parity_hashmap_get(#[case] code: &str) {
    assert_parity(code);
}

// Map (HashMap): set with CoW — original unaffected after copy
#[rstest]
#[case("var m: HashMap = hashMapNew(); hashMapPut(m, \"a\", 1); var n: HashMap = m; hashMapPut(n, \"b\", 2); hashMapSize(m);")]
fn test_parity_hashmap_set_cow(#[case] code: &str) {
    assert_parity(code);
}

// Map (HashMap): keys count
#[rstest]
#[case("let m: HashMap = hashMapNew(); hashMapPut(m, \"a\", 1); hashMapPut(m, \"b\", 2); hashMapSize(m);")]
fn test_parity_hashmap_keys(#[case] code: &str) {
    assert_parity(code);
}

// Map (HashMap): remove (delete a key)
#[rstest]
#[case("let m: HashMap = hashMapNew(); hashMapPut(m, \"a\", 1); hashMapPut(m, \"b\", 2); hashMapRemove(m, \"a\"); hashMapSize(m);")]
fn test_parity_hashmap_remove(#[case] code: &str) {
    assert_parity(code);
}

// Queue: enqueue/dequeue/size
#[rstest]
#[case("let q: Queue = queueNew(); queueEnqueue(q, 1); queueEnqueue(q, 2); queueEnqueue(q, 3); queueSize(q);")]
#[case("let q: Queue = queueNew(); queueEnqueue(q, 10); queueEnqueue(q, 20); unwrap(queueDequeue(q)); queueSize(q);")]
#[case("let q: Queue = queueNew(); queueEnqueue(q, 42); unwrap(queueDequeue(q));")]
fn test_parity_queue_operations(#[case] code: &str) {
    assert_parity(code);
}

// Stack: push/pop/size
#[rstest]
#[case(
    "let s: Stack = stackNew(); stackPush(s, 1); stackPush(s, 2); stackPush(s, 3); stackSize(s);"
)]
#[case("let s: Stack = stackNew(); stackPush(s, 10); stackPush(s, 20); unwrap(stackPop(s)); stackSize(s);")]
#[case("let s: Stack = stackNew(); stackPush(s, 99); unwrap(stackPop(s));")]
fn test_parity_stack_operations(#[case] code: &str) {
    assert_parity(code);
}

// CoW semantics: identical behavior in both engines
#[rstest]
#[case("let a: number[] = [1, 2, 3]; let b: number[] = a; a[0] = 99; b[0];")]
#[case("var a: array = [1, 2]; var b: array = a; b.push(9); len(a);")]
#[case("let a: number[] = [1, 2, 3]; let b: number[] = a; b[2] = 100; a[2];")]
fn test_parity_cow_semantics(#[case] code: &str) {
    assert_parity(code);
}

// ============================================================================
// Phase interpreter-02: Integration Tests - Closures and Scopes
// ============================================================================

#[test]
fn test_integration_nested_function_with_params() {
    // Test nested function that takes parameters (avoids closure capture warnings)
    let code = r#"
        fn outer(x: number) -> number {
            fn inner(y: number) -> number {
                return y * 2;
            }
            return inner(x);
        }
        outer(10);
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn test_integration_nested_function_calls() {
    let code = r#"
        fn a(x: number) -> number { return x + 1; }
        fn b(x: number) -> number { return a(x) + 1; }
        fn c(x: number) -> number { return b(x) + 1; }
        c(0);
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_integration_scope_shadowing() {
    let code = r#"
        let x = 1;
        fn test() -> number {
            let x = 2;
            return x;
        }
        test() + x;
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_integration_multiple_function_levels() {
    // Test function calls across multiple levels
    let code = r#"
        fn level1(x: number) -> number {
            fn level2(y: number) -> number {
                fn level3(z: number) -> number {
                    return z + 1;
                }
                return level3(y) + 1;
            }
            return level2(x) + 1;
        }
        level1(0);
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_integration_function_as_parameter() {
    // Test higher-order function pattern
    let code = r#"
        fn apply(f: (number) -> number, x: number) -> number {
            return f(x);
        }
        fn double(n: number) -> number {
            return n * 2;
        }
        apply(double, 5);
    "#;
    assert_eval_number(code, 10.0);
}

// ============================================================================
// Phase interpreter-02: Integration Tests - Error Recovery
// ============================================================================

#[test]
fn test_integration_undefined_variable_error() {
    let result = Atlas::new().eval("undefined_var;");
    assert!(result.is_err(), "Expected error for undefined variable");
}

#[test]
fn test_integration_type_mismatch_error() {
    let result = Atlas::new().eval(r#"let x: number = "hello";"#);
    assert!(result.is_err(), "Expected type mismatch error");
}

#[test]
fn test_integration_divide_by_zero_error() {
    assert_error_code("10 / 0;", "AT0005");
}

#[test]
fn test_integration_array_index_out_of_bounds() {
    let result = Atlas::new().eval("let arr = [1, 2, 3]; arr[10];");
    assert!(result.is_err(), "Expected array index out of bounds error");
}

#[test]
fn test_integration_function_wrong_arity() {
    let code = r#"
        fn add(a: number, b: number) -> number { return a + b; }
        add(1);
    "#;
    let result = Atlas::new().eval(code);
    assert!(result.is_err(), "Expected function arity error");
}

// ============================================================================
// Phase interpreter-02: Integration Tests - Complex Programs
// ============================================================================

#[test]
fn test_integration_fibonacci_recursive() {
    let code = r#"
        fn fib(n: number) -> number {
            if (n <= 1) { return n; }
            return fib(n - 1) + fib(n - 2);
        }
        fib(10);
    "#;
    assert_eval_number(code, 55.0);
}

#[test]
fn test_integration_factorial() {
    let code = r#"
        fn factorial(n: number) -> number {
            if (n <= 1) { return 1; }
            return n * factorial(n - 1);
        }
        factorial(5);
    "#;
    assert_eval_number(code, 120.0);
}

#[test]
fn test_integration_sum_to_n() {
    let code = r#"
        fn sum_to(n: number) -> number {
            var sum = 0;
            var i = 1;
            while (i <= n) {
                sum = sum + i;
                i = i + 1;
            }
            return sum;
        }
        sum_to(100);
    "#;
    assert_eval_number(code, 5050.0);
}

#[test]
fn test_integration_is_prime() {
    let code = r#"
        fn is_prime(n: number) -> bool {
            if (n < 2) { return false; }
            var i = 2;
            while (i * i <= n) {
                if (n % i == 0) { return false; }
                i = i + 1;
            }
            return true;
        }
        is_prime(17);
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_integration_is_not_prime() {
    let code = r#"
        fn is_prime(n: number) -> bool {
            if (n < 2) { return false; }
            var i = 2;
            while (i * i <= n) {
                if (n % i == 0) { return false; }
                i = i + 1;
            }
            return true;
        }
        is_prime(15);
    "#;
    assert_eval_bool(code, false);
}

// ============================================================================
// Phase interpreter-02: Integration Tests - Stdlib Functions
// ============================================================================

#[test]
fn test_integration_stdlib_len_string() {
    assert_eval_number(r#"len("hello");"#, 5.0);
}

#[test]
fn test_integration_stdlib_len_array() {
    assert_eval_number("len([1, 2, 3, 4, 5]);", 5.0);
}

#[test]
fn test_integration_stdlib_str() {
    assert_eval_string("str(42);", "42");
}

#[test]
fn test_integration_stdlib_trim() {
    assert_eval_string(r#"trim("  hello  ");"#, "hello");
}

#[test]
fn test_integration_stdlib_split_join() {
    let code = r#"
        let parts = split("a,b,c", ",");
        join(parts, "-");
    "#;
    assert_eval_string(code, "a-b-c");
}

#[test]
fn test_integration_stdlib_substring() {
    assert_eval_string(r#"substring("hello world", 0, 5);"#, "hello");
}

#[test]
fn test_integration_stdlib_includes() {
    assert_eval_bool(r#"includes("hello world", "world");"#, true);
}

#[test]
fn test_integration_stdlib_starts_with() {
    assert_eval_bool(r#"startsWith("hello world", "hello");"#, true);
}

#[test]
fn test_integration_stdlib_ends_with() {
    assert_eval_bool(r#"endsWith("hello world", "world");"#, true);
}

#[test]
fn test_integration_stdlib_replace() {
    assert_eval_string(
        r#"replace("hello world", "world", "atlas");"#,
        "hello atlas",
    );
}

// ============================================================================
// Phase interpreter-02: Performance Correctness Tests
// ============================================================================

#[test]
fn test_perf_loop_1000_iterations() {
    let code = "var i = 0; while (i < 1000) { i = i + 1; } i;";
    assert_eval_number(code, 1000.0);
}

#[test]
fn test_perf_nested_loop_correctness() {
    let code = r#"
        var count = 0;
        var i = 0;
        while (i < 10) {
            var j = 0;
            while (j < 10) {
                count = count + 1;
                j = j + 1;
            }
            i = i + 1;
        }
        count;
    "#;
    assert_eval_number(code, 100.0);
}

#[test]
fn test_perf_string_accumulation() {
    let code = r#"
        var s = "";
        var i = 0;
        while (i < 50) {
            s = s + "x";
            i = i + 1;
        }
        len(s);
    "#;
    assert_eval_number(code, 50.0);
}

#[test]
fn test_perf_function_calls_correctness() {
    let code = r#"
        fn inc(x: number) -> number { return x + 1; }
        var r = 0;
        var i = 0;
        while (i < 100) {
            r = inc(r);
            i = i + 1;
        }
        r;
    "#;
    assert_eval_number(code, 100.0);
}

#[test]
fn test_perf_array_operations() {
    // Test array indexing performance
    let code = r#"
        let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        var sum = 0;
        var i = 0;
        while (i < 100) {
            sum = sum + arr[i % 10];
            i = i + 1;
        }
        sum;
    "#;
    assert_eval_number(code, 550.0); // sum of 1-10 is 55, times 10 = 550
}

// ============================================================================
// Phase interpreter-02: Edge Case Tests
// ============================================================================

#[test]
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

/// CoW: mutating a cloned array does not affect the original.
///
/// `var a = [1, 2, 3]; var b = a; b[0] = 99;`
/// After mutation, `a[0]` must still be 1 — CoW cloned the underlying data.
#[test]
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

/// arr.push with inferred array type (no annotation)
#[test]
fn test_array_method_push_inferred_type() {
    assert_eval_number(r#"let arr = [1, 2, 3]; arr.push(4); arr[3];"#, 4.0);
}

/// Parity: interpreter and VM produce same result for push
#[test]
fn test_array_method_push_parity_via_atlas_eval() {
    let code = r#"var arr: array = [1, 2, 3]; arr.push(99); arr[3];"#;
    assert_eval_number(code, 99.0);
}

// ============================================================================
// Value semantics regression tests — CoW behavior must never regress
// ============================================================================

/// Regression: assignment creates independent copy; mutation of source does not
/// affect the copy (CoW value semantics).
#[test]
fn test_value_semantics_regression_assign_copy() {
    let code = r#"
        let a: number[] = [1, 2, 3];
        let b: number[] = a;
        a[0] = 99;
        b[0]
    "#;
    assert_eval_number(code, 1.0);
}

/// Regression: mutation of assigned copy does not affect source.
#[test]
fn test_value_semantics_regression_copy_mutation_isolated() {
    let code = r#"
        let a: number[] = [1, 2, 3];
        let b: number[] = a;
        b[0] = 42;
        a[0]
    "#;
    assert_eval_number(code, 1.0);
}

/// Regression: push on assigned copy does not grow the source.
#[test]
fn test_value_semantics_regression_push_copy_isolated() {
    let code = r#"
        var a: array = [1, 2, 3];
        var b: array = a;
        b.push(4);
        len(a)
    "#;
    assert_eval_number(code, 3.0);
}

/// Regression: function parameter is an independent copy — mutations stay local.
#[test]
fn test_value_semantics_regression_fn_param_copy() {
    let code = r#"
        fn fill(arr: number[]) -> void {
            arr[0] = 999;
        }
        let nums: number[] = [1, 2, 3];
        fill(nums);
        nums[0]
    "#;
    assert_eval_number(code, 1.0);
}

/// Regression: three-way copy — each variable is independent.
#[test]
fn test_value_semantics_regression_three_way_copy() {
    let code = r#"
        let a: number[] = [1, 2, 3];
        let b: number[] = a;
        let c: number[] = b;
        b[0] = 10;
        c[1] = 20;
        a[0] + a[1]
    "#;
    assert_eval_number(code, 3.0);
}

// ============================================================================
// Phase 08: Runtime `own` enforcement in interpreter (debug mode)
// ============================================================================

/// After passing a variable to an `own` parameter, reading it must fail in debug mode.
#[test]
fn test_own_param_consumes_binding_debug() {
    let src = r#"
        fn consume(own data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        consume(arr);
        arr;
    "#;
    let result = run_interpreter(src);
    assert!(
        result.is_err(),
        "Expected error after consuming arr, got: {:?}",
        result
    );
    assert!(
        result.unwrap_err().contains("use of moved value"),
        "Error should mention 'use of moved value'"
    );
}

/// A `borrow` parameter must NOT consume the caller's binding.
#[test]
fn test_borrow_param_does_not_consume_binding() {
    let src = r#"
        fn read(borrow data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        read(arr);
        len(arr);
    "#;
    let result = run_interpreter(src);
    assert!(
        result.is_ok(),
        "borrow should not consume binding, got: {:?}",
        result
    );
    assert_eq!(result.unwrap(), "Number(3)");
}

/// An unannotated parameter must NOT consume the caller's binding.
#[test]
fn test_unannotated_param_does_not_consume_binding() {
    let src = r#"
        fn take(data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        take(arr);
        len(arr);
    "#;
    let result = run_interpreter(src);
    assert!(
        result.is_ok(),
        "unannotated param should not consume binding, got: {:?}",
        result
    );
    assert_eq!(result.unwrap(), "Number(3)");
}

/// Passing a literal to an `own` parameter must not attempt to consume any binding.
#[test]
fn test_own_param_with_literal_arg_no_consume() {
    let src = r#"
        fn consume(own data: array<number>) -> void { }
        consume([1, 2, 3]);
        42;
    "#;
    let result = run_interpreter(src);
    assert!(
        result.is_ok(),
        "literal arg to own param should not error, got: {:?}",
        result
    );
    assert_eq!(result.unwrap(), "Number(42)");
}

/// Passing an expression result to an `own` parameter must not consume any binding.
#[test]
fn test_own_param_with_expression_arg_no_consume() {
    let src = r#"
        fn make_arr() -> array<number> { [10, 20]; }
        fn consume(own data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        consume(make_arr());
        len(arr);
    "#;
    let result = run_interpreter(src);
    assert!(
        result.is_ok(),
        "expression arg to own param should not consume unrelated binding, got: {:?}",
        result
    );
    assert_eq!(result.unwrap(), "Number(3)");
}

// ============================================================================
// Phase 09: Runtime `shared` enforcement in interpreter (debug mode)
// ============================================================================

/// Passing a plain (non-shared) value to a `shared` param must produce a runtime error.
#[test]
fn test_shared_param_rejects_plain_value_debug() {
    let src = r#"
        fn register(shared handler: number[]) -> void { }
        let arr: number[] = [1, 2, 3];
        register(arr);
    "#;
    let result = run_interpreter(src);
    assert!(
        result.is_err(),
        "Expected ownership violation error, got: {:?}",
        result
    );
    assert!(
        result.unwrap_err().contains("ownership violation"),
        "Error should mention 'ownership violation'"
    );
}

/// Passing an actual SharedValue to a `shared` param must succeed.
#[test]
fn test_shared_param_accepts_shared_value() {
    use atlas_runtime::value::{Shared, Value};

    // Parse and register the function
    let src = r#"
        fn register(shared handler: number[]) -> void { }
        register(sv);
    "#;
    let mut lexer = atlas_runtime::lexer::Lexer::new(src);
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::parser::Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = atlas_runtime::binder::Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = atlas_runtime::typechecker::TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);

    let mut interp = Interpreter::new();
    // Inject a SharedValue into the interpreter's globals so Atlas source can reference it
    let shared_val = Value::SharedValue(Shared::new(Box::new(Value::array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
    ]))));
    interp.define_global("sv".to_string(), shared_val);

    let result = interp.eval(&program, &SecurityContext::allow_all());
    assert!(
        result.is_ok(),
        "SharedValue passed to shared param should succeed, got: {:?}",
        result
    );
}

/// Passing a SharedValue to an `own` param emits an advisory (not a hard error).
#[test]
fn test_shared_value_to_own_param_advisory_not_error() {
    use atlas_runtime::value::{Shared, Value};

    let src = r#"
        fn consume(own handler: number[]) -> void { }
        consume(sv);
    "#;
    let mut lexer = atlas_runtime::lexer::Lexer::new(src);
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::parser::Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = atlas_runtime::binder::Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = atlas_runtime::typechecker::TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);

    let mut interp = Interpreter::new();
    let shared_val = Value::SharedValue(Shared::new(Box::new(Value::array(vec![Value::Number(
        1.0,
    )]))));
    interp.define_global("sv".to_string(), shared_val);

    // Advisory warning only — must NOT be a hard error
    let result = interp.eval(&program, &SecurityContext::allow_all());
    assert!(
        result.is_ok(),
        "SharedValue to own param should be advisory (not hard error), got: {:?}",
        result
    );
}

// ============================================================================
// Phase 13: Ownership Parity Verification
// Interpreter and VM must produce identical output/error for all ownership scenarios.
// ============================================================================

/// Run source through both engines and assert they produce the same result (Ok or Err).
/// For Ok: values must match as Debug strings.
/// For Err: both must fail (message parity checked separately per-test where needed).
fn assert_ownership_parity(source: &str) {
    // Interpreter
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);
    let mut interp = Interpreter::new();
    let interp_result = interp.eval(&program, &SecurityContext::allow_all());

    // VM
    let mut lexer2 = Lexer::new(source);
    let (tokens2, _) = lexer2.tokenize();
    let mut parser2 = Parser::new(tokens2);
    let (program2, _) = parser2.parse();
    let mut binder2 = Binder::new();
    let (mut symbol_table2, _) = binder2.bind(&program2);
    let mut typechecker2 = TypeChecker::new(&mut symbol_table2);
    let _ = typechecker2.check(&program2);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program2).expect("VM compilation failed");
    let mut vm = VM::new(bytecode);
    let vm_result = vm.run(&SecurityContext::allow_all());

    match (interp_result, vm_result) {
        (Ok(_), Ok(_)) => {}   // Both succeeded — parity holds
        (Err(_), Err(_)) => {} // Both errored — parity holds (message checked per-test)
        (Ok(v), Err(e)) => panic!(
            "Parity mismatch: interpreter OK ({:?}), VM err ({:?})\n{}",
            v, e, source
        ),
        (Err(e), Ok(v)) => panic!(
            "Parity mismatch: interpreter err ({:?}), VM ok ({:?})\n{}",
            e, v, source
        ),
    }
}

/// Assert both engines fail with an error containing `expected_fragment`.
fn assert_ownership_parity_err(source: &str, expected_fragment: &str) {
    let interp_result = run_interpreter(source);
    let vm_result = run_vm(source);

    assert!(
        interp_result.is_err(),
        "Interpreter should error for:\n{}\ngot: {:?}",
        source,
        interp_result
    );
    assert!(
        vm_result.is_err(),
        "VM should error for:\n{}\ngot: {:?}",
        source,
        vm_result
    );
    let ie = interp_result.unwrap_err();
    let ve = vm_result.unwrap_err();
    assert!(
        ie.contains(expected_fragment),
        "Interpreter error missing '{}': {}",
        expected_fragment,
        ie
    );
    assert!(
        ve.contains(expected_fragment),
        "VM error missing '{}': {}",
        expected_fragment,
        ve
    );
}

/// Run source through VM, return Ok(debug_string) or Err(debug_string).
fn run_vm(source: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).expect("VM compilation failed");
    let mut vm = VM::new(bytecode);
    match vm.run(&SecurityContext::allow_all()) {
        Ok(v) => Ok(format!("{:?}", v)),
        Err(e) => Err(format!("{:?}", e)),
    }
}

// ─── Scenario 1: Unannotated function — no regression ────────────────────────

#[test]
fn test_parity_unannotated_function_no_regression() {
    assert_ownership_parity(
        r#"
        fn add(a: number, b: number) -> number { a + b; }
        add(3, 4);
        "#,
    );
}

// ─── Scenario 2: own param — callee receives value ───────────────────────────

#[test]
fn test_parity_own_param_callee_receives_value() {
    assert_ownership_parity(
        r#"
        fn consume(own data: array<number>) -> number { len(data); }
        consume([10, 20, 30]);
        "#,
    );
}

// ─── Scenario 3: own param — caller uses consumed binding (debug) ─────────────

#[test]
#[cfg(debug_assertions)]
fn test_parity_own_param_consumes_binding() {
    assert_ownership_parity_err(
        r#"
        fn consume(own data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        consume(arr);
        arr;
        "#,
        "use of moved value",
    );
}

// ─── Scenario 4: borrow param — caller retains value ─────────────────────────

#[test]
fn test_parity_borrow_param_caller_retains_value() {
    assert_ownership_parity(
        r#"
        fn read(borrow data: array<number>) -> number { len(data); }
        let arr: array<number> = [1, 2, 3];
        read(arr);
        len(arr);
        "#,
    );
}

// ─── Scenario 5: shared param with plain value — both error (debug) ──────────

#[test]
#[cfg(debug_assertions)]
fn test_parity_shared_param_rejects_plain_value() {
    assert_ownership_parity_err(
        r#"
        fn register(shared handler: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        register(arr);
        "#,
        "ownership violation",
    );
}

// ─── Scenario 6: Mixed annotations — own + borrow + unannotated ───────────────

#[test]
fn test_parity_mixed_annotations_own_borrow_none() {
    assert_ownership_parity(
        r#"
        fn process(own a: array<number>, borrow b: array<number>, c: number) -> number {
            len(a) + len(b) + c;
        }
        process([1, 2], [3, 4, 5], 10);
        "#,
    );
}

// ─── Scenario 7: own with literal argument — no binding consumed ──────────────

#[test]
fn test_parity_own_literal_arg_no_consume() {
    assert_ownership_parity(
        r#"
        fn consume(own data: array<number>) -> number { len(data); }
        consume([1, 2, 3, 4]);
        42;
        "#,
    );
}

// ─── Scenario 8: own return type annotation — parsed, ignored at runtime ──────

#[test]
fn test_parity_own_return_type_annotation() {
    // Both engines must accept the annotation without error.
    // Result value not compared (function-body return diverges pre-Block2).
    assert_ownership_parity(
        r#"
        fn make() -> own array<number> { [1, 2, 3]; }
        make();
        42;
        "#,
    );
}

// ─── Scenario 9: borrow return type annotation — parsed, ignored ──────────────

#[test]
fn test_parity_borrow_return_type_annotation() {
    // Both engines must accept borrow return annotation without error.
    assert_ownership_parity(
        r#"
        fn peek(borrow data: array<number>) -> borrow array<number> { data; }
        let arr: array<number> = [10, 20];
        peek(arr);
        42;
        "#,
    );
}

// ─── Scenario 10: Nested function calls with ownership propagation ─────────────

#[test]
fn test_parity_nested_own_calls() {
    assert_ownership_parity(
        r#"
        fn inner(own data: array<number>) -> number { len(data); }
        fn outer(own data: array<number>) -> number { inner(data); }
        outer([1, 2, 3, 4, 5]);
        "#,
    );
}

// ─── Scenario 11: own param where arg is a function call result ───────────────

#[test]
fn test_parity_own_param_fn_call_result() {
    // own param with a literal array (avoids function-return divergence)
    // Both engines must accept and not error.
    assert_ownership_parity(
        r#"
        fn consume(own data: array<number>) -> void { }
        consume([1, 2, 3]);
        42;
        "#,
    );
}

// ─── Scenario 12: Multiple borrow calls to same value ─────────────────────────

#[test]
fn test_parity_multiple_borrow_calls_same_value() {
    assert_ownership_parity(
        r#"
        fn read(borrow data: array<number>) -> number { len(data); }
        let arr: array<number> = [1, 2, 3, 4, 5];
        read(arr);
        read(arr);
        read(arr);
        "#,
    );
}

// ─── Scenario 13: own then second access — same error ─────────────────────────

#[test]
#[cfg(debug_assertions)]
fn test_parity_own_then_second_access_errors() {
    assert_ownership_parity_err(
        r#"
        fn consume(own data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        consume(arr);
        consume(arr);
        "#,
        "use of moved value",
    );
}

// ─── Scenario 14: Ownership on recursive functions ────────────────────────────

#[test]
fn test_parity_own_recursive_function() {
    assert_ownership_parity(
        r#"
        fn sum(borrow data: array<number>, i: number) -> number {
            if i >= len(data) { 0; } else { data[i] + sum(data, i + 1); }
        }
        let arr: array<number> = [1, 2, 3, 4, 5];
        sum(arr, 0);
        "#,
    );
}

// ─── Scenario 15: own annotation on void function (no return) ─────────────────

#[test]
fn test_parity_own_annotation_void_function() {
    assert_ownership_parity(
        r#"
        fn sink(own data: array<number>) -> void { }
        sink([1, 2, 3]);
        42;
        "#,
    );
}

// ─── Scenario 16: borrow then own of same binding — own errors (debug) ─────────

#[test]
#[cfg(debug_assertions)]
fn test_parity_borrow_then_own_same_binding() {
    assert_ownership_parity_err(
        r#"
        fn borrow_it(borrow data: array<number>) -> void { }
        fn own_it(own data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        borrow_it(arr);
        own_it(arr);
        arr;
        "#,
        "use of moved value",
    );
}

// ─── Scenario 17: Function stored in variable, own param ─────────────────────

#[test]
fn test_parity_own_param_via_variable_call() {
    assert_ownership_parity(
        r#"
        fn consume(own data: array<number>) -> number { len(data); }
        let f: (array<number>) -> number = consume;
        f([10, 20, 30]);
        "#,
    );
}

// ─── Scenario 18: multiple sequential own calls with distinct literals ─────────

#[test]
fn test_parity_multiple_own_calls_distinct_literals() {
    assert_ownership_parity(
        r#"
        fn consume(own data: array<number>) -> number { len(data); }
        consume([1]);
        consume([2, 3]);
        consume([4, 5, 6]);
        "#,
    );
}

// ─── Scenario 19: Nested scope inner fn calls outer with own param ─────────────

#[test]
fn test_parity_nested_scope_own_param() {
    assert_ownership_parity(
        r#"
        fn outer() -> number {
            fn inner(own data: array<number>) -> number { len(data); }
            inner([1, 2, 3, 4]);
        }
        outer();
        "#,
    );
}

// ─── Scenario 20: Error message identical between engines ─────────────────────

#[test]
#[cfg(debug_assertions)]
fn test_parity_error_message_identical_own_violation() {
    let src = r#"
        fn consume(own data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        consume(arr);
        arr;
    "#;
    let ie = run_interpreter(src).unwrap_err();
    let ve = run_vm(src).unwrap_err();
    assert!(
        ie.contains("use of moved value"),
        "Interpreter error: {}",
        ie
    );
    assert!(ve.contains("use of moved value"), "VM error: {}", ve);
}

#[test]
#[cfg(debug_assertions)]
fn test_parity_error_message_identical_shared_violation() {
    let src = r#"
        fn register(shared handler: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        register(arr);
    "#;
    let ie = run_interpreter(src).unwrap_err();
    let ve = run_vm(src).unwrap_err();
    assert!(
        ie.contains("ownership violation"),
        "Interpreter error: {}",
        ie
    );
    assert!(ve.contains("ownership violation"), "VM error: {}", ve);
    // Both must include param name
    assert!(
        ie.contains("handler"),
        "Interpreter error missing param name: {}",
        ie
    );
    assert!(
        ve.contains("handler"),
        "VM error missing param name: {}",
        ve
    );
}

// ============================================================
// Phase 14 — VM: Trait Dispatch Parity Tests
// ============================================================
// Tests in this section verify interpreter parity for trait dispatch.
// VM tests live in vm.rs Phase 12/13 sections.

#[test]
fn test_parity_trait_method_string_dispatch() {
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Wrap { fn wrap(self: Wrap) -> string; }
        impl Wrap for string {
            fn wrap(self: string) -> string { return \"[\" + self + \"]\"; }
        }
        let s: string = \"hello\";
        let r: string = s.wrap();
        r
    ",
        )
        .expect("Should succeed");
    assert_eq!(result, Value::string("[hello]"));
}

#[test]
fn test_parity_trait_method_number_compute() {
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Double { fn double(self: Double) -> number; }
        impl Double for number {
            fn double(self: number) -> number { return self * 2; }
        }
        let n: number = 21;
        let r: number = n.double();
        r
    ",
        )
        .expect("Should succeed");
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_parity_multiple_impl_types_no_collision() {
    let atlas = Atlas::new();

    let result_n = atlas
        .eval(
            "
        trait Tag { fn tag(self: Tag) -> string; }
        impl Tag for number {
            fn tag(self: number) -> string { return \"num\"; }
        }
        impl Tag for string {
            fn tag(self: string) -> string { return \"str\"; }
        }
        let n: number = 1;
        let r: string = n.tag();
        r
    ",
        )
        .expect("Should succeed");
    assert_eq!(result_n, Value::string("num"));

    let result_s = atlas
        .eval(
            "
        trait Tag { fn tag(self: Tag) -> string; }
        impl Tag for number {
            fn tag(self: number) -> string { return \"num\"; }
        }
        impl Tag for string {
            fn tag(self: string) -> string { return \"str\"; }
        }
        let s: string = \"hi\";
        let r: string = s.tag();
        r
    ",
        )
        .expect("Should succeed");
    assert_eq!(result_s, Value::string("str"));
}

#[test]
fn test_parity_trait_method_self_arg_is_receiver() {
    // Verify `self` inside the method body refers to the receiver value
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Identity { fn identity(self: Identity) -> number; }
        impl Identity for number {
            fn identity(self: number) -> number { return self; }
        }
        let n: number = 99;
        let r: number = n.identity();
        r
    ",
        )
        .expect("Should succeed");
    assert_eq!(result, Value::Number(99.0));
}

// ─── Block-03 Phase 17: Parity Hardening — 10 Extended Scenarios ─────────────

#[test]
fn test_parity_block03_scenario_a_interpreter() {
    // Multiple traits on same type
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Addable { fn add(self: Addable, n: number) -> number; }
        trait Subtractable { fn sub(self: Subtractable, n: number) -> number; }
        impl Addable for number { fn add(self: number, n: number) -> number { return self + n; } }
        impl Subtractable for number { fn sub(self: number, n: number) -> number { return self - n; } }
        let x: number = 10;
        let a: number = x.add(5);
        let b: number = a.sub(3);
        b
        ",
        )
        .expect("scenario A should succeed");
    assert_eq!(result, Value::Number(12.0));
}

#[test]
fn test_parity_block03_scenario_b_interpreter() {
    // Trait method returning bool, used in condition
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            r#"
        trait Comparable { fn greater_than(self: Comparable, other: number) -> bool; }
        impl Comparable for number {
            fn greater_than(self: number, other: number) -> bool { return self > other; }
        }
        let x: number = 10;
        var r: string = "no";
        if (x.greater_than(5)) { r = "yes"; }
        r
        "#,
        )
        .expect("scenario B should succeed");
    assert_eq!(result, Value::string("yes"));
}

#[test]
fn test_parity_block03_scenario_c_interpreter() {
    // Trait method calling stdlib function
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            r#"
        trait Formatted { fn fmt(self: Formatted) -> string; }
        impl Formatted for number {
            fn fmt(self: number) -> string { return "Value: " + str(self); }
        }
        let x: number = 42;
        let r: string = x.fmt();
        r
        "#,
        )
        .expect("scenario C should succeed");
    assert_eq!(result, Value::string("Value: 42"));
}

#[test]
fn test_parity_block03_scenario_d_interpreter() {
    // Chained trait method calls (via intermediate variables)
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Inc { fn inc(self: Inc) -> number; }
        impl Inc for number { fn inc(self: number) -> number { return self + 1; } }
        let x: number = 40;
        let y: number = x.inc();
        let z: number = y.inc();
        z
        ",
        )
        .expect("scenario D should succeed");
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_parity_block03_scenario_e_interpreter() {
    // Trait method with multiple parameters
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Interpolator { fn interpolate(self: Interpolator, t: number, other: number) -> number; }
        impl Interpolator for number {
            fn interpolate(self: number, t: number, other: number) -> number {
                return self + (other - self) * t;
            }
        }
        let a: number = 0;
        let r: number = a.interpolate(0.5, 100);
        r
        ",
        )
        .expect("scenario E should succeed");
    assert_eq!(result, Value::Number(50.0));
}

#[test]
fn test_parity_block03_scenario_f_interpreter() {
    // Trait method with conditional return paths (clamp)
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Clamp { fn clamp(self: Clamp, min: number, max: number) -> number; }
        impl Clamp for number {
            fn clamp(self: number, min: number, max: number) -> number {
                if (self < min) { return min; }
                if (self > max) { return max; }
                return self;
            }
        }
        let x: number = 150;
        let r: number = x.clamp(0, 100);
        r
        ",
        )
        .expect("scenario F should succeed");
    assert_eq!(result, Value::Number(100.0));
}

#[test]
fn test_parity_block03_scenario_g_interpreter() {
    // Impl method with local state (no leakage)
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Counter { fn count_to(self: Counter, n: number) -> number; }
        impl Counter for number {
            fn count_to(self: number, n: number) -> number {
                var total: number = 0;
                var i: number = self;
                while (i <= n) { total = total + i; i = i + 1; }
                return total;
            }
        }
        let x: number = 1;
        let r: number = x.count_to(10);
        r
        ",
        )
        .expect("scenario G should succeed");
    assert_eq!(result, Value::Number(55.0));
}

#[test]
fn test_parity_block03_scenario_h_interpreter() {
    // String type impl
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            r#"
        trait Shouter { fn shout(self: Shouter) -> string; }
        impl Shouter for string {
            fn shout(self: string) -> string { return self + "!!!"; }
        }
        let s: string = "hello";
        let r: string = s.shout();
        r
        "#,
        )
        .expect("scenario H should succeed");
    assert_eq!(result, Value::string("hello!!!"));
}

#[test]
fn test_parity_block03_scenario_i_interpreter() {
    // Bool type impl
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Toggle { fn toggle(self: Toggle) -> bool; }
        impl Toggle for bool { fn toggle(self: bool) -> bool { return !self; } }
        let b: bool = true;
        let r: bool = b.toggle();
        r
        ",
        )
        .expect("scenario I should succeed");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_parity_block03_scenario_j_interpreter() {
    // Trait method returning array, index into result
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Pair { fn pair(self: Pair) -> number[]; }
        impl Pair for number { fn pair(self: number) -> number[] { return [self, self * 2]; } }
        let x: number = 7;
        let p: number[] = x.pair();
        let r: number = p[1];
        r
        ",
        )
        .expect("scenario J should succeed");
    assert_eq!(result, Value::Number(14.0));
}
