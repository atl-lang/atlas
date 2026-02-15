//! Tests for nested function binding (Phase 3)
//!
//! Tests semantic analysis of nested functions including:
//! - Name resolution
//! - Shadowing behavior
//! - Forward references
//! - Error detection

use atlas_runtime::binder::Binder;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;

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
