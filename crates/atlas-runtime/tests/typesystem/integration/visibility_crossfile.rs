// Cross-file visibility tests (B38-P04)
//
// Tests for visibility enforcement when importing symbols between modules:
// - pub fn/struct exports can be imported
// - private (default) fn/struct cannot be imported — AT3059
// - nonexistent symbols cannot be imported — AT5006

use atlas_runtime::binder::Binder;
use atlas_runtime::diagnostic::Diagnostic;
use atlas_runtime::module_loader::{ModuleLoader, ModuleRegistry};
use atlas_runtime::typechecker::TypeChecker;
use std::fs;
use tempfile::TempDir;

fn typecheck_modules(entry: &str, modules: &[(&str, &str)]) -> Vec<Diagnostic> {
    let temp_dir = TempDir::new().unwrap();
    for (name, content) in modules {
        let path = temp_dir.path().join(format!("{}.atl", name));
        fs::write(&path, content).unwrap();
    }

    let entry_path = temp_dir
        .path()
        .join(format!("{}.atl", entry))
        .canonicalize()
        .unwrap();
    let mut loader = ModuleLoader::new(temp_dir.path().to_path_buf());
    let loaded_modules = loader.load_module(&entry_path).unwrap();

    let mut registry = ModuleRegistry::new();
    let mut diagnostics = Vec::new();
    let mut entry_ast = None;
    let mut entry_table = None;

    for module in &loaded_modules {
        let mut binder = Binder::new();
        let (table, mut bind_diags) =
            binder.bind_with_modules(&module.ast, &module.path, &registry);
        diagnostics.append(&mut bind_diags);

        if module.path == entry_path {
            entry_ast = Some(module.ast.clone());
            entry_table = Some(table.clone());
        }

        registry.register(module.path.clone(), table);
    }

    if let (Some(ast), Some(mut table)) = (entry_ast, entry_table) {
        let mut checker = TypeChecker::new(&mut table);
        let mut type_diags = checker.check_with_modules(&ast, &entry_path, &registry);
        diagnostics.append(&mut type_diags);
    }

    diagnostics
}

fn has_error_code(diagnostics: &[Diagnostic], code: &str) -> bool {
    diagnostics.iter().any(|d| d.code == code)
}

fn errors_only(diagnostics: &[Diagnostic]) -> Vec<&Diagnostic> {
    diagnostics
        .iter()
        .filter(|d| d.level == atlas_runtime::diagnostic::DiagnosticLevel::Error)
        .collect()
}

// ============================================================================
// Public Export Tests
// ============================================================================

#[test]
fn test_import_pub_fn_allowed() {
    let diags = typecheck_modules(
        "main",
        &[
            (
                "lib",
                r#"
                export fn public_fn(): number {
                    return 42;
                }
                "#,
            ),
            (
                "main",
                r#"
                import { public_fn } from "./lib";
                let x = public_fn();
                "#,
            ),
        ],
    );
    assert!(
        errors_only(&diags).is_empty(),
        "Should allow importing public fn: {:?}",
        diags
    );
}

#[test]
fn test_import_multiple_pub_fns() {
    let diags = typecheck_modules(
        "main",
        &[
            (
                "lib",
                r#"
                export fn foo(): number { return 1; }
                export fn bar(): number { return 2; }
                export fn baz(): number { return 3; }
                "#,
            ),
            (
                "main",
                r#"
                import { foo, bar, baz } from "./lib";
                let x = foo() + bar() + baz();
                "#,
            ),
        ],
    );
    assert!(
        errors_only(&diags).is_empty(),
        "Should allow importing multiple public fns: {:?}",
        diags
    );
}

// ============================================================================
// Private Access Violation Tests (AT3059)
// ============================================================================

#[test]
fn test_import_private_fn_error() {
    let diags = typecheck_modules(
        "main",
        &[
            (
                "lib",
                r#"
                fn private_fn(): number {
                    return 99;
                }
                "#,
            ),
            (
                "main",
                r#"
                import { private_fn } from "./lib";
                "#,
            ),
        ],
    );
    assert!(
        has_error_code(&diags, "AT3059"),
        "Should emit AT3059 for private fn import: {:?}",
        diags
    );
}

#[test]
fn test_import_private_fn_alongside_public() {
    let diags = typecheck_modules(
        "main",
        &[
            (
                "lib",
                r#"
                export fn public_fn(): number { return 1; }
                fn private_fn(): number { return 2; }
                "#,
            ),
            (
                "main",
                r#"
                import { public_fn, private_fn } from "./lib";
                "#,
            ),
        ],
    );
    assert!(
        has_error_code(&diags, "AT3059"),
        "Should emit AT3059 for private fn import: {:?}",
        diags
    );
}

#[test]
fn test_private_access_error_message_has_symbol_name() {
    let diags = typecheck_modules(
        "main",
        &[
            (
                "lib",
                r#"
                fn helper(): number { return 42; }
                "#,
            ),
            (
                "main",
                r#"
                import { helper } from "./lib";
                "#,
            ),
        ],
    );
    let at3059_diags: Vec<_> = diags.iter().filter(|d| d.code == "AT3059").collect();
    assert!(!at3059_diags.is_empty(), "Should have AT3059 error");
    assert!(
        at3059_diags[0].message.contains("helper"),
        "Error message should mention 'helper': {}",
        at3059_diags[0].message
    );
}

// ============================================================================
// Module Not Exported Tests (AT5006)
// ============================================================================

#[test]
fn test_import_nonexistent_symbol_error() {
    let diags = typecheck_modules(
        "main",
        &[
            (
                "lib",
                r#"
                export fn foo(): number { return 1; }
                "#,
            ),
            (
                "main",
                r#"
                import { nonexistent } from "./lib";
                "#,
            ),
        ],
    );
    assert!(
        has_error_code(&diags, "AT5006"),
        "Should emit AT5006 for nonexistent symbol: {:?}",
        diags
    );
}

#[test]
fn test_nonexistent_error_message_accurate() {
    let diags = typecheck_modules(
        "main",
        &[
            (
                "lib",
                r#"
                export fn foo(): number { return 1; }
                "#,
            ),
            (
                "main",
                r#"
                import { missing_symbol } from "./lib";
                "#,
            ),
        ],
    );
    let at5006_diags: Vec<_> = diags.iter().filter(|d| d.code == "AT5006").collect();
    assert!(!at5006_diags.is_empty(), "Should have AT5006 error");
    assert!(
        at5006_diags[0].message.contains("missing_symbol"),
        "Error message should mention 'missing_symbol': {}",
        at5006_diags[0].message
    );
}

// ============================================================================
// Mixed Scenarios
// ============================================================================

#[test]
fn test_import_chain_public_only() {
    let diags = typecheck_modules(
        "main",
        &[
            (
                "base",
                r#"
                export fn base_fn(): number { return 1; }
                "#,
            ),
            (
                "middle",
                r#"
                import { base_fn } from "./base";
                export fn middle_fn(): number { return base_fn() + 1; }
                "#,
            ),
            (
                "main",
                r#"
                import { middle_fn } from "./middle";
                let x = middle_fn();
                "#,
            ),
        ],
    );
    assert!(
        errors_only(&diags).is_empty(),
        "Should allow import chain of public fns: {:?}",
        diags
    );
}

#[test]
fn test_import_from_empty_module() {
    let diags = typecheck_modules(
        "main",
        &[
            ("lib", r#""#),
            (
                "main",
                r#"
                import { anything } from "./lib";
                "#,
            ),
        ],
    );
    assert!(
        has_error_code(&diags, "AT5006"),
        "Should emit AT5006 when importing from empty module: {:?}",
        diags
    );
}

#[test]
fn test_pub_fn_used_correctly() {
    let diags = typecheck_modules(
        "main",
        &[
            (
                "math",
                r#"
                export fn add(x: number, y: number): number {
                    return x + y;
                }

                fn internal_helper(): number {
                    return 0;
                }
                "#,
            ),
            (
                "main",
                r#"
                import { add } from "./math";
                let result = add(1, 2);
                "#,
            ),
        ],
    );
    assert!(
        errors_only(&diags).is_empty(),
        "Should allow using imported public fn: {:?}",
        diags
    );
}

// ============================================================================
// Struct Visibility
// ============================================================================

#[test]
fn test_import_pub_struct_allowed() {
    let diags = typecheck_modules(
        "main",
        &[
            (
                "types",
                r#"
                export struct Point {
                    x: number,
                    y: number,
                }
                "#,
            ),
            (
                "main",
                r#"
                import { Point } from "./types";
                let p = Point { x: 1, y: 2 };
                "#,
            ),
        ],
    );
    assert!(
        errors_only(&diags).is_empty(),
        "Should allow importing public struct: {:?}",
        diags
    );
}
