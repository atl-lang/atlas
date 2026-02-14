//! Module Binding and Type Checking Tests (BLOCKER 04-C)
//!
//! Tests cross-module binding, import/export validation, and type checking.

use atlas_runtime::{
    binder::Binder, lexer::Lexer, module_loader::ModuleRegistry, parser::Parser,
    typechecker::TypeChecker,
};
use std::path::PathBuf;

/// Helper to parse and bind a module
fn bind_module(
    source: &str,
) -> (
    atlas_runtime::symbol::SymbolTable,
    Vec<atlas_runtime::diagnostic::Diagnostic>,
) {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    binder.bind(&program)
}

/// Helper to parse, bind with modules, and return symbol table + diagnostics
fn bind_module_with_registry(
    source: &str,
    module_path: &str,
    registry: &ModuleRegistry,
) -> (
    atlas_runtime::symbol::SymbolTable,
    Vec<atlas_runtime::diagnostic::Diagnostic>,
) {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    binder.bind_with_modules(&program, &PathBuf::from(module_path), registry)
}

/// Helper to type check with modules
#[allow(dead_code)] // Preserved for future test expansion
fn typecheck_module_with_registry(
    source: &str,
    module_path: &str,
    registry: &ModuleRegistry,
) -> Vec<atlas_runtime::diagnostic::Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    let (mut symbol_table, bind_diags) =
        binder.bind_with_modules(&program, &PathBuf::from(module_path), registry);

    if !bind_diags.is_empty() {
        return bind_diags;
    }

    let mut typechecker = TypeChecker::new(&mut symbol_table);
    typechecker.check_with_modules(&program, &PathBuf::from(module_path), registry)
}

#[test]
fn test_basic_export_function() {
    let source = r#"
export fn add(a: number, b: number) -> number {
    return a + b;
}
"#;

    let (symbol_table, diags) = bind_module(source);
    assert!(
        diags.is_empty(),
        "Expected no diagnostics, got: {:?}",
        diags
    );

    // Check that function is in symbol table and marked as exported
    let exports = symbol_table.get_exports();
    assert!(exports.contains_key("add"), "Expected 'add' to be exported");
    assert!(
        exports.get("add").unwrap().exported,
        "Expected 'add' to be marked as exported"
    );
}

#[test]
fn test_basic_export_variable() {
    let source = r#"
export let MY_PI = 3.14159;
"#;

    let (symbol_table, diags) = bind_module(source);
    assert!(
        diags.is_empty(),
        "Expected no diagnostics, got: {:?}",
        diags
    );

    // Check that variable is exported
    let exports = symbol_table.get_exports();
    assert!(
        exports.contains_key("MY_PI"),
        "Expected 'MY_PI' to be exported"
    );
}

#[test]
fn test_export_nonexistent_symbol() {
    // Note: In Atlas, you can't export without defining inline
    // This is caught by the parser, not the binder
}

#[test]
fn test_duplicate_exports() {
    let source = r#"
export fn foo() -> number {
    return 1;
}

export fn foo() -> number {
    return 2;
}
"#;

    // Parse and bind
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    let (mut symbol_table, _bind_diags) = binder.bind(&program);

    // Type check should catch duplicate exports
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let diags = typechecker.check_with_modules(
        &program,
        &PathBuf::from("/test.atl"),
        &ModuleRegistry::new(),
    );

    assert!(
        !diags.is_empty(),
        "Expected diagnostic for duplicate export"
    );
    assert!(
        diags.iter().any(|d| d.code == "AT5008"),
        "Expected AT5008 (duplicate export) diagnostic"
    );
}

#[test]
fn test_basic_named_import() {
    // Create registry with exported module
    let mut registry = ModuleRegistry::new();

    // Module A exports 'add'
    let module_a = r#"
export fn add(a: number, b: number) -> number {
    return a + b;
}
"#;
    let (symbol_table_a, _) = bind_module(module_a);
    registry.register(PathBuf::from("/module_a.atl"), symbol_table_a);

    // Module B imports 'add' from A
    let module_b = r#"
import { add } from "/module_a.atl";

let result = add(2, 3);
"#;

    let (symbol_table_b, diags) = bind_module_with_registry(module_b, "/module_b.atl", &registry);
    assert!(
        diags.is_empty(),
        "Expected no diagnostics, got: {:?}",
        diags
    );

    // Check that 'add' is in module B's symbol table
    assert!(
        symbol_table_b.lookup("add").is_some(),
        "Expected 'add' to be in symbol table"
    );
    assert!(
        !symbol_table_b.lookup("add").unwrap().exported,
        "Imported symbols should not be re-exported"
    );
}

#[test]
fn test_import_nonexistent_module() {
    let registry = ModuleRegistry::new();

    let source = r#"
import { foo } from "/nonexistent.atl";
"#;

    let (_symbol_table, diags) = bind_module_with_registry(source, "/test.atl", &registry);
    assert!(!diags.is_empty(), "Expected diagnostic for missing module");
    assert!(
        diags.iter().any(|d| d.code == "AT5005"),
        "Expected AT5005 (module not found) diagnostic"
    );
}

#[test]
fn test_import_nonexistent_export() {
    let mut registry = ModuleRegistry::new();

    // Module A exports 'add' but not 'subtract'
    let module_a = r#"
export fn add(a: number, b: number) -> number {
    return a + b;
}
"#;
    let (symbol_table_a, _) = bind_module(module_a);
    registry.register(PathBuf::from("/module_a.atl"), symbol_table_a);

    // Module B tries to import 'subtract'
    let module_b = r#"
import { subtract } from "/module_a.atl";
"#;

    let (_symbol_table_b, diags) = bind_module_with_registry(module_b, "/module_b.atl", &registry);
    assert!(!diags.is_empty(), "Expected diagnostic for missing export");
    assert!(
        diags.iter().any(|d| d.code == "AT5006"),
        "Expected AT5006 (export not found) diagnostic"
    );
}

#[test]
fn test_import_multiple_named_exports() {
    let mut registry = ModuleRegistry::new();

    // Module A exports multiple functions
    let module_a = r#"
export fn add(a: number, b: number) -> number {
    return a + b;
}

export fn subtract(a: number, b: number) -> number {
    return a - b;
}

export let MY_PI = 3.14159;
"#;
    let (symbol_table_a, _) = bind_module(module_a);
    registry.register(PathBuf::from("/math.atl"), symbol_table_a);

    // Module B imports multiple symbols
    let module_b = r#"
import { add, subtract, MY_PI } from "/math.atl";
"#;

    let (symbol_table_b, diags) = bind_module_with_registry(module_b, "/test.atl", &registry);
    assert!(
        diags.is_empty(),
        "Expected no diagnostics, got: {:?}",
        diags
    );

    // Check all imported symbols
    assert!(symbol_table_b.lookup("add").is_some());
    assert!(symbol_table_b.lookup("subtract").is_some());
    assert!(symbol_table_b.lookup("MY_PI").is_some());
}

#[test]
fn test_namespace_import_not_supported() {
    let mut registry = ModuleRegistry::new();

    let module_a = r#"
export fn add(a: number, b: number) -> number {
    return a + b;
}
"#;
    let (symbol_table_a, _) = bind_module(module_a);
    registry.register(PathBuf::from("/math.atl"), symbol_table_a);

    // Try namespace import
    let module_b = r#"
import * as math from "/math.atl";
"#;

    let (_symbol_table, diags) = bind_module_with_registry(module_b, "/test.atl", &registry);
    assert!(
        !diags.is_empty(),
        "Expected diagnostic for unsupported namespace import"
    );
    assert!(
        diags.iter().any(|d| d.code == "AT5007"),
        "Expected AT5007 (namespace import not supported) diagnostic"
    );
}

#[test]
fn test_import_preserves_type() {
    let mut registry = ModuleRegistry::new();

    // Module A exports typed function
    let module_a = r#"
export fn add(a: number, b: number) -> number {
    return a + b;
}
"#;
    let (symbol_table_a, _) = bind_module(module_a);
    registry.register(PathBuf::from("/math.atl"), symbol_table_a);

    // Module B imports and checks type
    let module_b = r#"
import { add } from "/math.atl";
"#;

    let (symbol_table_b, diags) = bind_module_with_registry(module_b, "/test.atl", &registry);
    assert!(
        diags.is_empty(),
        "Expected no diagnostics, got: {:?}",
        diags
    );

    // Verify imported symbol has correct type
    let add_symbol = symbol_table_b.lookup("add").unwrap();
    assert!(matches!(
        add_symbol.ty,
        atlas_runtime::types::Type::Function { .. }
    ));
}

#[test]
fn test_exported_function_hoisting() {
    let source = r#"
export fn foo() -> number {
    return bar();
}

export fn bar() -> number {
    return 42;
}
"#;

    let (symbol_table, diags) = bind_module(source);
    assert!(
        diags.is_empty(),
        "Expected no diagnostics, got: {:?}",
        diags
    );

    // Both functions should be hoisted and exported
    let exports = symbol_table.get_exports();
    assert!(exports.contains_key("foo"));
    assert!(exports.contains_key("bar"));
}
