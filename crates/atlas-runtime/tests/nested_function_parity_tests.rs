//! Parity tests for nested functions (Phase 7)
//!
//! Ensures 100% interpreter/VM parity for nested function execution.
//! Every test runs the same source code in both engines and verifies
//! identical output.

use atlas_runtime::binder::Binder;
use atlas_runtime::compiler::Compiler;
use atlas_runtime::interpreter::Interpreter;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::value::Value;
use atlas_runtime::vm::VM;

fn run_interpreter(source: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    let (_symbol_table, _) = binder.bind(&program);

    let mut interpreter = Interpreter::new();
    interpreter
        .eval(&program, &SecurityContext::allow_all())
        .map_err(|e| format!("{:?}", e))
}

fn run_vm(source: &str) -> Result<Value, String> {
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
fn parity_nested_function_basic() {
    let source = r#"
        fn outer() -> number {
            fn helper(x: number) -> number {
                return x * 2;
            }
            return helper(21);
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

#[test]
fn parity_nested_function_multiple_params() {
    let source = r#"
        fn outer() -> number {
            fn add(a: number, b: number) -> number {
                return a + b;
            }
            return add(10, 32);
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

#[test]
fn parity_nested_function_string() {
    let source = r#"
        fn outer() -> string {
            fn greet(name: string) -> string {
                return "Hello, " + name;
            }
            return greet("World");
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::string("Hello, World"));
}

// ============================================================================
// Parameter Access
// ============================================================================

#[test]
fn parity_nested_function_params() {
    let source = r#"
        fn outer(x: number) -> number {
            fn double(y: number) -> number {
                return y * 2;
            }
            return double(x);
        }
        outer(21);
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

// ============================================================================
// Shadowing
// ============================================================================

#[test]
fn parity_nested_function_shadows_global() {
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

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

#[test]
fn parity_nested_function_shadows_outer_nested() {
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

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

// ============================================================================
// Multiple Nesting Levels
// ============================================================================

#[test]
fn parity_deeply_nested_functions() {
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

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

// ============================================================================
// Nested Functions Calling Each Other
// ============================================================================

#[test]
fn parity_nested_function_calling_nested() {
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

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

#[test]
fn parity_nested_function_calling_outer() {
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

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

// ============================================================================
// Void Functions
// ============================================================================

#[test]
fn parity_nested_function_void() {
    let source = r#"
        var result: number = 0;

        fn outer() -> void {
            fn setResult() -> void {
                result = 42;
            }
            setResult();
        }

        outer();
        result;
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

// ============================================================================
// Arrays
// ============================================================================

#[test]
fn parity_nested_function_array_param() {
    let source = r#"
        fn outer() -> number {
            fn sum(arr: number[]) -> number {
                return arr[0] + arr[1];
            }
            let nums: number[] = [10, 32];
            return sum(nums);
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

#[test]
fn parity_nested_function_array_return() {
    let source = r#"
        fn outer() -> number[] {
            fn makeArray() -> number[] {
                return [42, 100];
            }
            return makeArray();
        }
        outer()[0];
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

// ============================================================================
// Control Flow
// ============================================================================

#[test]
fn parity_nested_function_conditional() {
    let source = r#"
        fn outer() -> number {
            fn abs(x: number) -> number {
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

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

// ============================================================================
// Nested Functions in Different Block Types
// ============================================================================

#[test]
fn parity_nested_function_in_if_block() {
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

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

// ============================================================================
// Additional Parity Tests (to reach 20+)
// ============================================================================

#[test]
fn parity_nested_function_no_params() {
    let source = r#"
        fn outer() -> number {
            fn getValue() -> number {
                return 42;
            }
            return getValue();
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

#[test]
fn parity_multiple_nested_same_level() {
    let source = r#"
        fn outer() -> number {
            fn a() -> number { return 10; }
            fn b() -> number { return 20; }
            fn c() -> number { return 12; }
            return a() + b() + c();
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

#[test]
fn parity_nested_with_local_variables() {
    let source = r#"
        fn outer() -> number {
            fn compute() -> number {
                let x = 21;
                let y = x * 2;
                return y;
            }
            return compute();
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

#[test]
fn parity_nested_returning_bool() {
    let source = r#"
        fn outer() -> bool {
            fn isTrue() -> bool {
                return true;
            }
            return isTrue();
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Bool(true));
}

#[test]
fn parity_nested_with_arithmetic() {
    let source = r#"
        fn outer() -> number {
            fn calculate(a: number, b: number, c: number) -> number {
                return (a + b) * c;
            }
            return calculate(5, 9, 3);
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

#[test]
fn parity_nested_with_string_concat() {
    let source = r#"
        fn outer() -> string {
            fn combine(a: string, b: string) -> string {
                return a + b;
            }
            return combine("Hello", "World");
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::string("HelloWorld"));
}

#[test]
fn parity_nested_calling_multiple_siblings() {
    let source = r#"
        fn outer() -> number {
            fn getBase() -> number {
                return 20;
            }
            fn getBonus() -> number {
                return 22;
            }
            fn total() -> number {
                return getBase() + getBonus();
            }
            return total();
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}
