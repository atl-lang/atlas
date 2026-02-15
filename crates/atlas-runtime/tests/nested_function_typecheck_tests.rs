//! Tests for nested function type checking (Phase 4)
//!
//! Tests type safety of nested functions including:
//! - Parameter type checking
//! - Return type checking
//! - Return path analysis
//! - Type errors in function calls

use atlas_runtime::binder::Binder;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::typechecker::TypeChecker;

fn typecheck_source(source: &str) -> Vec<String> {
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

fn typecheck_source_with_warnings(source: &str) -> Vec<String> {
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

    let errors = typecheck_source(source);
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

    let errors = typecheck_source(source);
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

    let errors = typecheck_source(source);
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

    let errors = typecheck_source(source);
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

    let errors = typecheck_source(source);
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

    let errors = typecheck_source(source);
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

    let errors = typecheck_source(source);
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

    let errors = typecheck_source(source);
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

    let errors = typecheck_source(source);
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

    let errors = typecheck_source(source);
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

    let errors = typecheck_source(source);
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

    let errors = typecheck_source(source);
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

    let errors = typecheck_source(source);
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

    let errors = typecheck_source(source);
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

    let errors = typecheck_source(source);
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

    let errors = typecheck_source(source);
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

    let errors = typecheck_source(source);
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

    let errors = typecheck_source(source);
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

    let diagnostics = typecheck_source_with_warnings(source);
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

    let diagnostics = typecheck_source_with_warnings(source);
    // Should NOT have warning for _x (underscore prefix suppresses)
    let unused_warnings = diagnostics.iter().filter(|d| d.contains("AT2001")).count();
    assert_eq!(
        unused_warnings, 0,
        "Should not warn about _x: {:?}",
        diagnostics
    );
}
