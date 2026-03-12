//! Visibility keyword tests (B37-P04)
//!
//! Tests for `pub`, `private`, and `internal` visibility modifiers.

use super::*;

// ============================================================================
// Visibility Keywords - Parsing
// ============================================================================

#[test]
fn test_pub_function_parses() {
    let (program, diagnostics) = parse_source("pub fn foo(): number { return 1; }");
    assert_eq!(diagnostics.len(), 0, "pub fn should parse without errors");
    assert_eq!(program.items.len(), 1);
    if let Item::Function(func) = &program.items[0] {
        assert_eq!(func.visibility, Visibility::Public);
        assert_eq!(func.name.name, "foo");
    } else {
        panic!("Expected function item");
    }
}

#[test]
fn test_private_function_parses() {
    let (program, diagnostics) = parse_source("private fn bar(): string { return \"hi\"; }");
    assert_eq!(
        diagnostics.len(),
        0,
        "private fn should parse without errors"
    );
    assert_eq!(program.items.len(), 1);
    if let Item::Function(func) = &program.items[0] {
        assert_eq!(func.visibility, Visibility::Private);
        assert_eq!(func.name.name, "bar");
    } else {
        panic!("Expected function item");
    }
}

#[test]
fn test_internal_function_parses() {
    let (program, diagnostics) = parse_source("internal fn baz(): bool { return true; }");
    assert_eq!(
        diagnostics.len(),
        0,
        "internal fn should parse without errors"
    );
    assert_eq!(program.items.len(), 1);
    if let Item::Function(func) = &program.items[0] {
        assert_eq!(func.visibility, Visibility::Internal);
        assert_eq!(func.name.name, "baz");
    } else {
        panic!("Expected function item");
    }
}

#[test]
fn test_default_visibility_is_private() {
    let (program, diagnostics) = parse_source("fn implicit(): number { return 42; }");
    assert_eq!(diagnostics.len(), 0);
    if let Item::Function(func) = &program.items[0] {
        assert_eq!(
            func.visibility,
            Visibility::Private,
            "default visibility should be Private"
        );
    } else {
        panic!("Expected function item");
    }
}

// ============================================================================
// Visibility on Structs
// ============================================================================

#[test]
fn test_pub_struct_parses() {
    let (program, diagnostics) = parse_source("pub struct Point { x: number, y: number }");
    assert_eq!(diagnostics.len(), 0);
    if let Item::Struct(s) = &program.items[0] {
        assert_eq!(s.visibility, Visibility::Public);
        assert_eq!(s.name.name, "Point");
    } else {
        panic!("Expected struct item");
    }
}

#[test]
fn test_private_struct_parses() {
    let (program, diagnostics) = parse_source("private struct Internal { value: number }");
    assert_eq!(diagnostics.len(), 0);
    if let Item::Struct(s) = &program.items[0] {
        assert_eq!(s.visibility, Visibility::Private);
    } else {
        panic!("Expected struct item");
    }
}

#[test]
fn test_internal_struct_parses() {
    let (program, diagnostics) = parse_source("internal struct ModuleOnly { data: string }");
    assert_eq!(diagnostics.len(), 0);
    if let Item::Struct(s) = &program.items[0] {
        assert_eq!(s.visibility, Visibility::Internal);
    } else {
        panic!("Expected struct item");
    }
}

// ============================================================================
// Visibility on Enums
// ============================================================================

#[test]
fn test_pub_enum_parses() {
    let (program, diagnostics) = parse_source("pub enum Color { Red, Green, Blue }");
    assert_eq!(diagnostics.len(), 0);
    if let Item::Enum(e) = &program.items[0] {
        assert_eq!(e.visibility, Visibility::Public);
        assert_eq!(e.name.name, "Color");
    } else {
        panic!("Expected enum item");
    }
}

#[test]
fn test_private_enum_parses() {
    let (program, diagnostics) = parse_source("private enum Status { Active, Inactive }");
    assert_eq!(diagnostics.len(), 0);
    if let Item::Enum(e) = &program.items[0] {
        assert_eq!(e.visibility, Visibility::Private);
    } else {
        panic!("Expected enum item");
    }
}

// ============================================================================
// Visibility on Traits
// ============================================================================

#[test]
fn test_pub_trait_parses() {
    let (program, diagnostics) =
        parse_source("pub trait Display { fn show(borrow self): string; }");
    assert_eq!(diagnostics.len(), 0);
    if let Item::Trait(t) = &program.items[0] {
        assert_eq!(t.visibility, Visibility::Public);
        assert_eq!(t.name.name, "Display");
    } else {
        panic!("Expected trait item");
    }
}

#[test]
fn test_private_trait_parses() {
    let (program, diagnostics) =
        parse_source("private trait Helper { fn help(borrow self): number; }");
    assert_eq!(diagnostics.len(), 0);
    if let Item::Trait(t) = &program.items[0] {
        assert_eq!(t.visibility, Visibility::Private);
    } else {
        panic!("Expected trait item");
    }
}

// ============================================================================
// Export with Visibility (export sets visibility to Public)
// ============================================================================

#[test]
fn test_export_function_is_public() {
    let (program, diagnostics) = parse_source("export fn exported(): number { return 1; }");
    assert_eq!(diagnostics.len(), 0);
    if let Item::Export(export) = &program.items[0] {
        if let ExportItem::Function(func) = &export.item {
            assert_eq!(
                func.visibility,
                Visibility::Public,
                "exported functions should be Public"
            );
        } else {
            panic!("Expected exported function");
        }
    } else {
        panic!("Expected export item");
    }
}

#[test]
fn test_export_struct_is_public() {
    let (program, diagnostics) = parse_source("export struct Exported { val: number }");
    assert_eq!(diagnostics.len(), 0);
    if let Item::Export(export) = &program.items[0] {
        if let ExportItem::Struct(s) = &export.item {
            assert_eq!(s.visibility, Visibility::Public);
        } else {
            panic!("Expected exported struct");
        }
    } else {
        panic!("Expected export item");
    }
}

// ============================================================================
// Visibility with Async Functions
// ============================================================================

#[test]
fn test_pub_async_function_parses() {
    let (program, diagnostics) = parse_source("pub async fn fetch(): string { return \"data\"; }");
    assert_eq!(diagnostics.len(), 0);
    if let Item::Function(func) = &program.items[0] {
        assert_eq!(func.visibility, Visibility::Public);
        assert!(func.is_async);
    } else {
        panic!("Expected function item");
    }
}

#[test]
fn test_internal_async_function_parses() {
    let (program, diagnostics) = parse_source("internal async fn process(): number { return 42; }");
    assert_eq!(diagnostics.len(), 0);
    if let Item::Function(func) = &program.items[0] {
        assert_eq!(func.visibility, Visibility::Internal);
        assert!(func.is_async);
    } else {
        panic!("Expected function item");
    }
}

// ============================================================================
// Visibility Symbol Tracking (Binder)
// ============================================================================

#[test]
fn test_pub_function_symbol_has_public_visibility() {
    let source = "pub fn visible(): number { return 1; }";
    let (program, _) = parse_source(source);
    let mut binder = Binder::new();
    let (table, _) = binder.bind(&program);

    if let Some(symbol) = table.lookup("visible") {
        assert_eq!(symbol.visibility, Visibility::Public);
    } else {
        panic!("Symbol 'visible' should be in symbol table");
    }
}

#[test]
fn test_private_function_symbol_has_private_visibility() {
    let source = "private fn hidden(): number { return 1; }";
    let (program, _) = parse_source(source);
    let mut binder = Binder::new();
    let (table, _) = binder.bind(&program);

    if let Some(symbol) = table.lookup("hidden") {
        assert_eq!(symbol.visibility, Visibility::Private);
    } else {
        panic!("Symbol 'hidden' should be in symbol table");
    }
}

#[test]
fn test_default_function_symbol_has_private_visibility() {
    let source = "fn defaultvis(): number { return 1; }";
    let (program, _) = parse_source(source);
    let mut binder = Binder::new();
    let (table, _) = binder.bind(&program);

    if let Some(symbol) = table.lookup("defaultvis") {
        assert_eq!(
            symbol.visibility,
            Visibility::Private,
            "default should be Private"
        );
    } else {
        panic!("Symbol 'defaultvis' should be in symbol table");
    }
}

// ============================================================================
// Same-file visibility (all access allowed)
// ============================================================================

#[test]
fn test_private_function_callable_in_same_file() {
    let source = r#"
        private fn helper(): number { return 42; }
        fn main(): number { return helper(); }
    "#;
    let diagnostics = get_all_diagnostics(source);
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Private functions should be callable in same file: {:?}",
        errors
    );
}

#[test]
fn test_internal_function_callable_in_same_file() {
    let source = r#"
        internal fn module_helper(): number { return 100; }
        fn main(): number { return module_helper(); }
    "#;
    let diagnostics = get_all_diagnostics(source);
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Internal functions should be callable in same file: {:?}",
        errors
    );
}

#[test]
fn test_pub_function_callable_in_same_file() {
    let source = r#"
        pub fn public_api(): number { return 1; }
        fn main(): number { return public_api(); }
    "#;
    let diagnostics = get_all_diagnostics(source);
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Public functions should be callable in same file: {:?}",
        errors
    );
}

// ============================================================================
// Mixed visibility declarations
// ============================================================================

#[test]
fn test_mixed_visibility_declarations() {
    let source = r#"
        pub fn api(): number { return internal_impl(); }
        internal fn internal_impl(): number { return private_helper(); }
        private fn private_helper(): number { return 42; }
    "#;
    let diagnostics = get_all_diagnostics(source);
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Mixed visibility should work in same file: {:?}",
        errors
    );
}

#[test]
fn test_visibility_on_generic_function() {
    let source = "pub fn identity<T>(x: T): T { return x; }";
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    if let Item::Function(func) = &program.items[0] {
        assert_eq!(func.visibility, Visibility::Public);
        assert!(!func.type_params.is_empty());
    } else {
        panic!("Expected function item");
    }
}

// ============================================================================
// AT3059 Error Code Structure Test
// ============================================================================

#[test]
fn test_at3059_error_code_exists() {
    // Verify the error code is properly defined
    use atlas_runtime::diagnostic::error_codes::PRIVATE_ACCESS_VIOLATION;
    assert_eq!(PRIVATE_ACCESS_VIOLATION.code, "AT3059");
    assert_eq!(PRIVATE_ACCESS_VIOLATION.level, DiagnosticLevel::Error);
    assert!(PRIVATE_ACCESS_VIOLATION.title.contains("private"));
}
