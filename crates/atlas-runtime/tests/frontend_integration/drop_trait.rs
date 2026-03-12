//! Drop trait foundation tests (B37-P02)
//!
//! Tests for the Drop trait: deterministic cleanup when variables go out of scope.

use super::*;
use atlas_runtime::compiler::Compiler;
use atlas_runtime::typechecker::TypeChecker;
use atlas_runtime::vm::VM;
use atlas_runtime::{Binder, SecurityContext};

/// Helper to run full pipeline: lex -> parse -> bind -> typecheck -> compile -> run
fn run_source(source: &str) -> Result<Option<Value>, String> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        return Err(format!("Lexer errors: {:?}", lex_diags));
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    if parse_diags
        .iter()
        .any(|d| d.level == DiagnosticLevel::Error)
    {
        return Err(format!("Parser errors: {:?}", parse_diags));
    }

    let mut binder = Binder::new();
    let (mut table, bind_diags) = binder.bind(&program);
    if bind_diags.iter().any(|d| d.level == DiagnosticLevel::Error) {
        return Err(format!("Binder errors: {:?}", bind_diags));
    }

    let mut checker = TypeChecker::new(&mut table);
    let type_diags = checker.check(&program);
    if type_diags.iter().any(|d| d.level == DiagnosticLevel::Error) {
        return Err(format!("Type errors: {:?}", type_diags));
    }

    let mut compiler = Compiler::new();
    let bytecode = compiler
        .compile(&program)
        .map_err(|e| format!("Compile errors: {:?}", e))?;

    let mut vm = VM::new(bytecode);
    vm.run(&SecurityContext::allow_all())
        .map_err(|e| format!("Runtime error: {:?}", e))
}

/// Helper to get diagnostics without running
fn get_diagnostics(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    let mut binder = Binder::new();
    let (mut table, bind_diags) = binder.bind(&program);
    let mut checker = TypeChecker::new(&mut table);
    let type_diags = checker.check(&program);

    let mut all = Vec::new();
    all.extend(lex_diags);
    all.extend(parse_diags);
    all.extend(bind_diags);
    all.extend(type_diags);
    all
}

// ============================================================================
// Drop trait syntax tests
// ============================================================================

#[test]
fn test_drop_trait_impl_parses() {
    // Verify impl Drop for Type syntax parses correctly
    // Note: Drop returns void per builtin trait definition
    let source = r#"
        struct Resource {
            id: number
        }

        impl Drop for Resource {
            fn drop(borrow self): void {
            }
        }
    "#;

    let diags = get_diagnostics(source);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Drop impl should parse and typecheck: {:?}",
        errors
    );
}

#[test]
fn test_drop_is_builtin_trait() {
    // Drop should be recognized as a builtin trait
    let source = r#"
        struct Handle {
            value: number
        }

        impl Drop for Handle {
            fn drop(borrow self): void {
            }
        }
    "#;

    let diags = get_diagnostics(source);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Drop should be recognized as builtin: {:?}",
        errors
    );
}

#[test]
fn test_drop_method_signature() {
    // Drop trait requires fn drop(self): void
    let source = r#"
        struct File {
            path: string
        }

        impl Drop for File {
            fn drop(borrow self): void {
            }
        }

        let f = File { path: "test.txt" };
    "#;

    let diags = get_diagnostics(source);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Valid Drop impl should have no errors: {:?}",
        errors
    );
}

// ============================================================================
// Drop scope tests (compile-time verification)
// ============================================================================

#[test]
fn test_drop_variable_compiles() {
    // Variable of type with Drop impl should compile
    let source = r#"
        struct Counter {
            count: number
        }

        impl Drop for Counter {
            fn drop(borrow self): void {
            }
        }

        fn test(): void {
            let c = Counter { count: 0 };
        }

        test();
    "#;

    let result = run_source(source);
    assert!(
        result.is_ok(),
        "Drop variable should compile and run: {:?}",
        result.err()
    );
}

#[test]
fn test_drop_in_block_scope() {
    // Drop should be called when block scope ends
    let source = r#"
        struct Logger {
            name: string
        }

        impl Drop for Logger {
            fn drop(borrow self): void {
            }
        }

        fn test(): void {
            {
                let log = Logger { name: "inner" };
            }
        }

        test();
    "#;

    let result = run_source(source);
    assert!(
        result.is_ok(),
        "Drop in block scope should work: {:?}",
        result.err()
    );
}

#[test]
fn test_multiple_drops_lifo_order() {
    // Multiple variables should be dropped in LIFO order
    let source = r#"
        struct Item {
            id: number
        }

        impl Drop for Item {
            fn drop(borrow self): void {
            }
        }

        fn test(): void {
            let a = Item { id: 1 };
            let b = Item { id: 2 };
            let c = Item { id: 3 };
        }

        test();
    "#;

    let result = run_source(source);
    assert!(
        result.is_ok(),
        "Multiple drops should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_drop_with_early_return() {
    // Drop should be called on early return
    let source = r#"
        struct Guard {
            active: bool
        }

        impl Drop for Guard {
            fn drop(borrow self): void {
            }
        }

        fn test(early: bool): number {
            let g = Guard { active: true };
            if early {
                return 1;
            }
            return 2;
        }

        test(true);
    "#;

    let result = run_source(source);
    assert!(
        result.is_ok(),
        "Drop with early return should work: {:?}",
        result.err()
    );
}

// ============================================================================
// Non-Drop types should work as before
// ============================================================================

#[test]
fn test_non_drop_type_unchanged() {
    // Types without Drop impl should work normally
    let source = r#"
        struct Point {
            x: number,
            y: number
        }

        fn test(): number {
            let p = Point { x: 1, y: 2 };
            p.x + p.y
        }

        test();
    "#;

    let result = run_source(source);
    assert!(
        result.is_ok(),
        "Non-drop types should work: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), Some(Value::Number(3.0)));
}
