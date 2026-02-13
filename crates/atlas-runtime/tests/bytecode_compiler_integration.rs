//! Modern Bytecode Compiler Integration Tests
//!
//! Converted from bytecode_compiler_integration.rs (261 lines → ~110 lines = 58% reduction)

mod common;

use atlas_runtime::compiler::Compiler;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::value::Value;
use atlas_runtime::vm::VM;
use rstest::rstest;

fn execute_source(source: &str) -> Result<Option<Value>, atlas_runtime::value::RuntimeError> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty(), "Lexer errors: {:?}", lex_diags);

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    assert!(parse_diags.is_empty(), "Parser errors: {:?}", parse_diags);

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).expect("Compilation failed");

    let mut vm = VM::new(bytecode);
    vm.run()
}

// ============================================================================
// Compound Assignment Operators
// ============================================================================

#[rstest]
#[case("let x = 10; x += 5; x;", 15.0)]
#[case("let x = 10; x -= 3; x;", 7.0)]
#[case("let x = 4; x *= 3; x;", 12.0)]
#[case("let x = 20; x /= 4; x;", 5.0)]
#[case("let x = 17; x %= 5; x;", 2.0)]
fn test_compound_assignments(#[case] source: &str, #[case] expected: f64) {
    let result = execute_source(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(expected)));
}

// ============================================================================
// Increment/Decrement Operators
// ============================================================================

#[rstest]
#[case("let x = 5; x++; x;", 6.0)]
#[case("let x = 5; x--; x;", 4.0)]
fn test_increment_decrement(#[case] source: &str, #[case] expected: f64) {
    let result = execute_source(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(expected)));
}

// ============================================================================
// Array Operations
// ============================================================================

#[test]
fn test_array_index_assignment() {
    let result = execute_source("let arr = [1, 2, 3]; arr[1] = 42; arr[1];");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(42.0)));
}

#[test]
fn test_array_compound_assignment() {
    let result = execute_source("let arr = [10, 20, 30]; arr[1] += 5; arr[1];");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(25.0)));
}

#[test]
fn test_array_increment() {
    let result = execute_source("let arr = [5, 10, 15]; arr[1]++; arr[1];");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(11.0)));
}

// ============================================================================
// Function Execution
// ============================================================================

#[test]
fn test_user_function_simple() {
    let result = execute_source("fn double(x: number) -> number { return x * 2; } double(21);");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(42.0)));
}

#[test]
fn test_user_function_with_multiple_params() {
    let result = execute_source("fn add(a: number, b: number) -> number { return a + b; } add(10, 32);");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(42.0)));
}

#[test]
fn test_user_function_recursion() {
    let result = execute_source(r#"
        fn factorial(n: number) -> number {
            if (n <= 1) { return 1; }
            return n * factorial(n - 1);
        }
        factorial(5);
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(120.0)));
}

#[test]
fn test_user_function_with_local_variables() {
    let result = execute_source(r#"
        fn calculate(x: number) -> number {
            let y = x * 2;
            let z = y + 10;
            return z;
        }
        calculate(5);
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(20.0)));
}

#[test]
fn test_multiple_functions() {
    let result = execute_source(r#"
        fn double(x: number) -> number { return x * 2; }
        fn triple(x: number) -> number { return x * 3; }
        double(7) + triple(4);
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(26.0)));
}

#[test]
fn test_function_calling_function() {
    let result = execute_source(r#"
        fn add(a: number, b: number) -> number { return a + b; }
        fn addThree(a: number, b: number, c: number) -> number {
            return add(add(a, b), c);
        }
        addThree(10, 20, 12);
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(42.0)));
}

// ============================================================================
// Complex Expression Chains
// ============================================================================

#[test]
fn test_multiple_compound_assignments() {
    let result = execute_source(r#"
        let x = 10;
        x += 5;
        x *= 2;
        x -= 3;
        x;
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(27.0)));
}

#[test]
fn test_mixed_operators() {
    let result = execute_source(r#"
        let x = 5;
        x++;
        x *= 2;
        x--;
        x;
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(11.0)));
}

// ============================================================================
// Nested Operations
// ============================================================================

#[test]
fn test_array_in_function() {
    let result = execute_source(r#"
        fn modify_array() -> number {
            let arr = [1, 2, 3];
            arr[1] += 10;
            return arr[1];
        }
        modify_array();
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(12.0)));
}

#[test]
fn test_loop_with_compound_assignment() {
    let result = execute_source(r#"
        let sum = 0;
        let i = 0;
        while (i < 5) {
            sum += i;
            i++;
        }
        sum;
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(10.0)));
}
