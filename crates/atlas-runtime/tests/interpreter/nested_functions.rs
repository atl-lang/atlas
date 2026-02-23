use super::*;
use pretty_assertions::assert_eq;

// From nested_function_binding_tests.rs
// ============================================================================

fn parse_and_bind(source: &str) -> (Vec<String>, Vec<String>) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diagnostics) = parser.parse();

    let mut binder = Binder::new();
    let (_symbol_table, bind_diagnostics) = binder.bind(&program);

    let parse_errors: Vec<String> = parse_diagnostics
        .iter()
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect();

    let bind_errors: Vec<String> = bind_diagnostics
        .iter()
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect();

    (parse_errors, bind_errors)
}

// ============================================================================
// Basic Nested Function Binding
// ============================================================================

#[test]
fn test_bind_nested_function_basic() {
    let source = r#"
        fn outer() -> number {
            fn helper(x: number) -> number {
                return x * 2;
            }
            return helper(21);
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

#[test]
fn test_bind_multiple_nested_functions() {
    let source = r#"
        fn outer() -> number {
            fn add(a: number, b: number) -> number {
                return a + b;
            }
            fn multiply(a: number, b: number) -> number {
                return a * b;
            }
            return add(2, multiply(3, 4));
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

#[test]
fn test_bind_deeply_nested_functions() {
    let source = r#"
        fn level1() -> number {
            fn level2() -> number {
                fn level3() -> number {
                    return 42;
                }
                return level3();
            }
            return level2();
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

// ============================================================================
// Forward References
// ============================================================================

#[test]
fn test_bind_forward_reference_same_scope() {
    let source = r#"
        fn outer() -> number {
            fn first() -> number {
                return second();
            }
            fn second() -> number {
                return 42;
            }
            return first();
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    // Binding should succeed - forward reference in same scope is allowed
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

#[test]
fn test_bind_mutual_recursion_same_scope() {
    let source = r#"
        fn outer() -> number {
            fn isEven(n: number) -> bool {
                if (n == 0) {
                    return true;
                }
                return isOdd(n - 1);
            }
            fn isOdd(n: number) -> bool {
                if (n == 0) {
                    return false;
                }
                return isEven(n - 1);
            }
            return 0;
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

// ============================================================================
// Shadowing
// ============================================================================

#[test]
fn test_bind_nested_function_shadows_global() {
    let source = r#"
        fn foo() -> number {
            return 1;
        }
        
        fn outer() -> number {
            fn foo() -> number {
                return 2;
            }
            return foo();
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

#[test]
fn test_bind_nested_function_shadows_builtin() {
    let source = r#"
        fn outer() -> number {
            fn print(x: number) -> number {
                return x;
            }
            return print(42);
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    // Nested functions CAN shadow builtins (unlike top-level functions)
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

#[test]
fn test_bind_nested_function_shadows_outer_nested() {
    let source = r#"
        fn level1() -> number {
            fn helper() -> number {
                return 1;
            }
            fn level2() -> number {
                fn helper() -> number {
                    return 2;
                }
                return helper();
            }
            return level2();
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_bind_redeclaration_same_scope() {
    let source = r#"
        fn outer() -> number {
            fn helper() -> number {
                return 1;
            }
            fn helper() -> number {
                return 2;
            }
            return 0;
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    // Should have redeclaration error
    assert!(!bind_errors.is_empty(), "Expected redeclaration error");
    assert!(
        bind_errors.iter().any(|e| e.contains("AT2003")),
        "Expected AT2003 error, got: {:?}",
        bind_errors
    );
}

#[test]
fn test_bind_nested_function_in_if_block() {
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
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

#[test]
fn test_bind_nested_function_in_while_block() {
    let source = r#"
        fn outer() -> number {
            var i: number = 0;
            while (i < 1) {
                fn helper() -> number {
                    return 42;
                }
                i++;
            }
            return 0;
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

#[test]
fn test_bind_nested_function_in_for_block() {
    let source = r#"
        fn outer() -> number {
            for (var i: number = 0; i < 5; i++) {
                fn helper(x: number) -> number {
                    return x;
                }
            }
            return 0;
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

#[test]
fn test_bind_nested_function_with_type_params() {
    let source = r#"
        fn outer<T>() -> number {
            fn inner<E>(x: E) -> number {
                return 42;
            }
            return inner(5);
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

// ============================================================================

// From nested_function_interpreter_tests.rs
// ============================================================================

fn nested_run_interpreter(source: &str) -> Result<Value, String> {
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

// ============================================================================
// Basic Nested Function Calls
// ============================================================================

#[test]
fn test_interp_nested_function_basic() {
    let source = r#"
        fn outer() -> number {
            fn helper(x: number) -> number {
                return x * 2;
            }
            return helper(21);
        }
        outer();
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_interp_nested_function_multiple_params() {
    let source = r#"
        fn outer() -> number {
            fn add(a: number, b: number) -> number {
                return a + b;
            }
            return add(10, 32);
        }
        outer();
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_interp_nested_function_string() {
    let source = r#"
        fn outer() -> string {
            fn greet(name: string) -> string {
                return "Hello, " + name;
            }
            return greet("World");
        }
        outer();
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::string("Hello, World"));
}

// ============================================================================
// Parameter Access
// ============================================================================

#[test]
fn test_interp_nested_function_params() {
    let source = r#"
        fn outer(x: number) -> number {
            fn double(y: number) -> number {
                return y * 2;
            }
            return double(x);
        }
        outer(21);
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Shadowing
// ============================================================================

#[test]
fn test_interp_nested_function_shadows_global() {
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

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_interp_nested_function_shadows_outer_nested() {
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

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Multiple Nesting Levels
// ============================================================================

#[test]
fn test_interp_deeply_nested_functions() {
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

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Nested Functions Calling Each Other
// ============================================================================

#[test]
fn test_interp_nested_function_calling_nested() {
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

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_interp_nested_function_calling_outer() {
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

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Void Functions
// ============================================================================

#[test]
fn test_interp_nested_function_void() {
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

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Arrays
// ============================================================================

#[test]
fn test_interp_nested_function_array_param() {
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

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_interp_nested_function_array_return() {
    let source = r#"
        fn outer() -> number[] {
            fn makeArray() -> number[] {
                return [42, 100];
            }
            return makeArray();
        }
        outer()[0];
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Control Flow
// ============================================================================

#[test]
fn test_interp_nested_function_conditional() {
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

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Nested Functions in Different Block Types
// ============================================================================

#[test]
fn test_interp_nested_function_in_if_block() {
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

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================

// From nested_function_typecheck_tests.rs
// ============================================================================

fn nested_typecheck_source(source: &str) -> Vec<String> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);

    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let diagnostics = typechecker.check(&program);

    diagnostics
        .iter()
        .filter(|d| d.level == atlas_runtime::diagnostic::DiagnosticLevel::Error)
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect()
}

fn nested_typecheck_source_with_warnings(source: &str) -> Vec<String> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);

    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let diagnostics = typechecker.check(&program);

    diagnostics
        .iter()
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect()
}

// ============================================================================
// Basic Type Checking
// ============================================================================

#[test]
fn test_typecheck_nested_function_basic() {
    let source = r#"
        fn outer() -> number {
            fn helper(x: number) -> number {
                return x * 2;
            }
            return helper(21);
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

#[test]
fn test_typecheck_nested_function_multiple_params() {
    let source = r#"
        fn outer() -> number {
            fn add(a: number, b: number) -> number {
                return a + b;
            }
            return add(10, 20);
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

#[test]
fn test_typecheck_nested_function_different_types() {
    let source = r#"
        fn outer() -> string {
            fn greet(name: string) -> string {
                return "Hello, " + name;
            }
            return greet("World");
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

// ============================================================================
// Return Path Analysis
// ============================================================================

#[test]
fn test_typecheck_nested_function_missing_return() {
    let source = r#"
        fn outer() -> number {
            fn helper(x: number) -> number {
                let y: number = x * 2;
            }
            return 0;
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert!(!errors.is_empty(), "Expected missing return error");
    assert!(
        errors.iter().any(|e| e.contains("AT3004")),
        "Expected AT3004 error, got: {:?}",
        errors
    );
}

#[test]
fn test_typecheck_nested_function_conditional_return() {
    let source = r#"
        fn outer() -> number {
            fn abs(x: number) -> number {
                if (x < 0) {
                    return -x;
                } else {
                    return x;
                }
            }
            return abs(-5);
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

#[test]
fn test_typecheck_nested_function_incomplete_return_paths() {
    let source = r#"
        fn outer() -> number {
            fn test(x: number) -> number {
                if (x > 0) {
                    return x;
                }
            }
            return 0;
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert!(!errors.is_empty(), "Expected incomplete return paths error");
    assert!(
        errors.iter().any(|e| e.contains("AT3004")),
        "Expected AT3004 error, got: {:?}",
        errors
    );
}

// ============================================================================
// Type Errors
// ============================================================================

#[test]
fn test_typecheck_nested_function_wrong_param_type() {
    let source = r#"
        fn outer() -> number {
            fn double(x: number) -> number {
                return x * 2;
            }
            return double("not a number");
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert!(!errors.is_empty(), "Expected type mismatch error");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("AT3001") || e.contains("type")),
        "Expected type error, got: {:?}",
        errors
    );
}

#[test]
fn test_typecheck_nested_function_wrong_return_type() {
    let source = r#"
        fn outer() -> number {
            fn bad() -> number {
                return "wrong type";
            }
            return bad();
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert!(!errors.is_empty(), "Expected type mismatch error");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("AT3001") || e.contains("type")),
        "Expected type error, got: {:?}",
        errors
    );
}

#[test]
fn test_typecheck_nested_function_param_type_mismatch() {
    let source = r#"
        fn outer() -> void {
            fn process(x: number, y: string) -> void {
                print(str(x) + y);
            }
            process(42, 100);
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert!(!errors.is_empty(), "Expected type mismatch error");
}

// ============================================================================
// Multiple Nesting Levels
// ============================================================================

#[test]
fn test_typecheck_deeply_nested_functions() {
    let source = r#"
        fn level1() -> number {
            fn level2() -> number {
                fn level3() -> number {
                    return 42;
                }
                return level3() + 1;
            }
            return level2() + 1;
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

#[test]
fn test_typecheck_nested_function_type_error_in_deep_nesting() {
    let source = r#"
        fn level1() -> number {
            fn level2() -> number {
                fn level3() -> number {
                    return "wrong";
                }
                return level3();
            }
            return level2();
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert!(!errors.is_empty(), "Expected type error");
}

// ============================================================================
// Function Calls
// ============================================================================

#[test]
fn test_typecheck_nested_function_calling_nested() {
    let source = r#"
        fn outer() -> number {
            fn helper1() -> number {
                return 10;
            }
            fn helper2() -> number {
                return helper1() + 20;
            }
            return helper2();
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

#[test]
fn test_typecheck_nested_function_calling_outer() {
    let source = r#"
        fn global() -> number {
            return 100;
        }
        
        fn outer() -> number {
            fn nested() -> number {
                return global() + 1;
            }
            return nested();
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

// ============================================================================
// Type Parameters
// ============================================================================

#[test]
fn test_typecheck_nested_function_with_type_params() {
    let source = r#"
        fn outer<T>() -> number {
            fn inner<E>(x: E) -> number {
                return 42;
            }
            return inner(100);
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

// ============================================================================
// Void Return Type
// ============================================================================

#[test]
fn test_typecheck_nested_function_void_return() {
    let source = r#"
        fn outer() -> void {
            fn helper() -> void {
                print("test");
            }
            helper();
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

#[test]
fn test_typecheck_nested_function_void_no_return_required() {
    let source = r#"
        fn outer() -> void {
            fn helper() -> void {
                let x: number = 42;
            }
            helper();
        }
    "#;

    let errors = nested_typecheck_source(source);
    // Void functions don't require explicit return
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

// ============================================================================
// Array and Complex Types
// ============================================================================

#[test]
fn test_typecheck_nested_function_array_param() {
    let source = r#"
        fn outer() -> number {
            fn sum(arr: number[]) -> number {
                return arr[0] + arr[1];
            }
            let nums: number[] = [10, 20];
            return sum(nums);
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

#[test]
fn test_typecheck_nested_function_array_return() {
    let source = r#"
        fn outer() -> number[] {
            fn makeArray() -> number[] {
                return [1, 2, 3];
            }
            return makeArray();
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

// ============================================================================
// Unused Parameter Warnings (Should Work)
// ============================================================================

#[test]
fn test_typecheck_nested_function_unused_param_warning() {
    let source = r#"
        fn outer() -> number {
            fn helper(x: number, y: number) -> number {
                return x;
            }
            return helper(10, 20);
        }
    "#;

    let diagnostics = nested_typecheck_source_with_warnings(source);
    // Should have warning for unused parameter 'y'
    assert!(
        diagnostics.iter().any(|d| d.contains("AT2001")),
        "Expected unused parameter warning, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_typecheck_nested_function_underscore_suppresses_warning() {
    let source = r#"
        fn outer() -> number {
            fn helper(_x: number) -> number {
                return 42;
            }
            return helper(10);
        }
    "#;

    let diagnostics = nested_typecheck_source_with_warnings(source);
    // Should NOT have warning for _x (underscore prefix suppresses)
    let unused_warnings = diagnostics.iter().filter(|d| d.contains("AT2001")).count();
    assert_eq!(
        unused_warnings, 0,
        "Should not warn about _x: {:?}",
        diagnostics
    );
}

// ============================================================================

