use super::*;
use pretty_assertions::assert_eq;
use std::io::Write;
use tempfile::NamedTempFile;

// From nested_function_vm_tests.rs
// ============================================================================

// Tests for nested function execution in the VM (Phase 6)
//
// Tests runtime behavior of nested functions in the VM including:
// - Basic nested function calls
// - Parameter access
// - Shadowing at runtime
// - Multiple nesting levels
// - Nested functions calling each other

fn nested_run_vm(source: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    let (_symbol_table, _) = binder.bind(&program);

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).map_err(|e| format!("{:?}", e))?;

    let mut vm = VM::new(bytecode);
    vm.run(&SecurityContext::allow_all())
        .map(|opt| opt.unwrap_or(Value::Null))
        .map_err(|e| format!("{:?}", e))
}

// ============================================================================
// Basic Nested Function Calls
// ============================================================================

#[test]
fn test_vm_nested_function_basic() {
    let source = r#"
        fn outer() -> number {
            fn helper(borrow x: number) -> number {
                return x * 2;
            }
            return helper(21);
        }
        outer();
    "#;

    let result = nested_run_vm(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_vm_nested_function_multiple_params() {
    let source = r#"
        fn outer() -> number {
            fn add(borrow a: number, borrow b: number) -> number {
                return a + b;
            }
            return add(10, 32);
        }
        outer();
    "#;

    let result = nested_run_vm(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_vm_nested_function_string() {
    let source = r#"
        fn outer() -> string {
            fn greet(borrow name: string) -> string {
                return "Hello, " + name;
            }
            return greet("World");
        }
        outer();
    "#;

    let result = nested_run_vm(source).unwrap();
    assert_eq!(result, Value::string("Hello, World"));
}

#[test]
fn test_vm_runtime_error_stack_trace() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let source = r#"fn level1() -> void {
    level2();
}
fn level2() -> void {
    level3();
}
fn level3() -> void {
    let arr = [1, 2];
    arr[5];
}
level1();
"#;
    write!(temp_file, "{}", source).unwrap();
    let file_path = temp_file.path().to_str().unwrap();

    let mut lexer = Lexer::new(source.to_string()).with_file(file_path);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (_symbol_table, _) = binder.bind(&program);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).expect("Compilation failed");

    let mut vm = VM::new(bytecode);
    let err = vm
        .run(&SecurityContext::allow_all())
        .expect_err("Expected runtime error");
    let stack_trace = vm.stack_trace(err.span());

    assert_eq!(stack_trace.len(), 3);
    assert_eq!(stack_trace[0].function, "level3");
    assert_eq!(stack_trace[0].line, 9);
    assert_eq!(stack_trace[0].column, 5);
    assert_eq!(stack_trace[1].function, "level2");
    assert_eq!(stack_trace[1].line, 2);
    assert_eq!(stack_trace[1].column, 5);
    assert_eq!(stack_trace[2].function, "level1");
    assert_eq!(stack_trace[2].line, 11);
    assert_eq!(stack_trace[2].column, 1);
}

// ============================================================================
// Parameter Access
// ============================================================================

#[test]
fn test_vm_nested_function_params() {
    let source = r#"
        fn outer(borrow x: number) -> number {
            fn double(borrow y: number) -> number {
                return y * 2;
            }
            return double(x);
        }
        outer(21);
    "#;

    let result = nested_run_vm(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Shadowing
// ============================================================================

#[test]
fn test_vm_nested_function_shadows_global() {
    let source = r#"
        fn foo() -> number {
            return 1;
        }

        fn outer() -> number {
            fn foo() -> number {
                return 42;
            }
            return foo();
        }
        outer();
    "#;

    let result = nested_run_vm(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_vm_nested_function_shadows_outer_nested() {
    let source = r#"
        fn level1() -> number {
            fn helper() -> number {
                return 1;
            }
            fn level2() -> number {
                fn helper() -> number {
                    return 42;
                }
                return helper();
            }
            return level2();
        }
        level1();
    "#;

    let result = nested_run_vm(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Multiple Nesting Levels
// ============================================================================

#[test]
fn test_vm_deeply_nested_functions() {
    let source = r#"
        fn level1() -> number {
            fn level2() -> number {
                fn level3() -> number {
                    fn level4() -> number {
                        return 42;
                    }
                    return level4();
                }
                return level3();
            }
            return level2();
        }
        level1();
    "#;

    let result = nested_run_vm(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Nested Functions Calling Each Other
// ============================================================================

#[test]
fn test_vm_nested_function_calling_nested() {
    let source = r#"
        fn outer() -> number {
            fn helper1() -> number {
                return 10;
            }
            fn helper2() -> number {
                return helper1() + 32;
            }
            return helper2();
        }
        outer();
    "#;

    let result = nested_run_vm(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_vm_nested_function_calling_outer() {
    let source = r#"
        fn global() -> number {
            return 40;
        }

        fn outer() -> number {
            fn nested() -> number {
                return global() + 2;
            }
            return nested();
        }
        outer();
    "#;

    let result = nested_run_vm(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Void Functions
// ============================================================================

#[test]
fn test_vm_nested_function_void() {
    let source = r#"
        let mut result: number = 0;

        fn outer() -> void {
            fn setResult() -> void {
                result = 42;
            }
            setResult();
        }

        outer();
        result;
    "#;

    let result = nested_run_vm(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Arrays
// ============================================================================

#[test]
fn test_vm_nested_function_array_param() {
    let source = r#"
        fn outer() -> number {
            fn sum(borrow arr: number[]) -> number {
                return arr[0] + arr[1];
            }
            let nums: number[] = [10, 32];
            return sum(nums);
        }
        outer();
    "#;

    let result = nested_run_vm(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_vm_nested_function_array_return() {
    let source = r#"
        fn outer() -> number[] {
            fn makeArray() -> number[] {
                return [42, 100];
            }
            return makeArray();
        }
        outer()[0];
    "#;

    let result = nested_run_vm(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Control Flow
// ============================================================================

#[test]
fn test_vm_nested_function_conditional() {
    let source = r#"
        fn outer() -> number {
            fn abs(borrow x: number) -> number {
                if (x < 0) {
                    return -x;
                } else {
                    return x;
                }
            }
            return abs(-42);
        }
        outer();
    "#;

    let result = nested_run_vm(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Nested Functions in Different Block Types
// ============================================================================

#[test]
fn test_vm_nested_function_in_if_block() {
    let source = r#"
        fn outer() -> number {
            if (true) {
                fn helper() -> number {
                    return 42;
                }
                return helper();
            }
            return 0;
        }
        outer();
    "#;

    let result = nested_run_vm(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
