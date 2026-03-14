use super::super::*;
use atlas_runtime::binder::Binder;
use atlas_runtime::diagnostic::Diagnostic;
use atlas_runtime::module_loader::{ModuleLoader, ModuleRegistry};
use atlas_runtime::typechecker::TypeChecker;
use std::fs;
use tempfile::TempDir;

fn has_error_crossfile(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(|d| d.is_error())
}

fn typecheck_multifile(entry: &str, modules: &[(&str, &str)]) -> Vec<Diagnostic> {
    let temp_dir = TempDir::new().unwrap();
    for (name, content) in modules {
        fs::write(temp_dir.path().join(format!("{}.atl", name)), content).unwrap();
    }
    let entry_path = temp_dir
        .path()
        .join(format!("{}.atl", entry))
        .canonicalize()
        .unwrap();
    let mut loader = ModuleLoader::new(temp_dir.path().to_path_buf());
    let loaded = loader.load_module(&entry_path).unwrap();
    let mut registry = ModuleRegistry::new();
    let mut diags = Vec::new();
    let mut entry_ast = None;
    let mut entry_table = None;
    for module in &loaded {
        let mut binder = Binder::new();
        let (table, mut bd) = binder.bind_with_modules(&module.ast, &module.path, &registry);
        diags.append(&mut bd);
        if module.path == entry_path {
            entry_ast = Some(module.ast.clone());
            entry_table = Some(table.clone());
        }
        registry.register(module.path.clone(), table);
    }
    if let (Some(ast), Some(mut table)) = (entry_ast, entry_table) {
        let mut checker = TypeChecker::new(&mut table);
        diags.append(&mut checker.check_with_modules(&ast, &entry_path, &registry));
    }
    diags
}

#[test]
fn test_struct_field_access_type_inference() {
    let diagnostics = typecheck_source(
        r#"
        struct Item { id: number }
        let item = Item { id: 5 };
        let item_id: number = item.id;
        "#,
    );
    assert!(!has_error(&diagnostics), "Errors: {:?}", diagnostics);
}

#[test]
fn test_struct_field_access_comparison() {
    let diagnostics = typecheck_source(
        r#"
        struct Item { id: number }
        let item = Item { id: 5 };
        if (item.id == 5) { }
        "#,
    );
    assert!(!has_error(&diagnostics), "Errors: {:?}", diagnostics);
}

#[test]
fn test_struct_field_access_assignment() {
    let diagnostics = typecheck_source(
        r#"
        struct Item { id: number }
        let mut item = Item { id: 5 };
        item.id = 6;
        "#,
    );
    assert!(!has_error(&diagnostics), "Errors: {:?}", diagnostics);
}

#[test]
fn test_struct_field_access_nested() {
    let diagnostics = typecheck_source(
        r#"
        struct Address { city: string }
        struct Person { address: Address }
        let person = Person { address: Address { city: "NY" } };
        let city: string = person.address.city;
        "#,
    );
    assert!(!has_error(&diagnostics), "Errors: {:?}", diagnostics);
}

#[test]
fn test_struct_field_access_global() {
    let diagnostics = typecheck_source(
        r#"
        struct Point { x: number }
        let mut point = Point { x: 1 };
        let x: number = point.x;
        "#,
    );
    assert!(!has_error(&diagnostics), "Errors: {:?}", diagnostics);
}

#[test]
fn test_struct_field_access_in_closure() {
    let diagnostics = typecheck_source(
        r#"
        struct Item { id: number }
        let item = Item { id: 5 };
        let get_id = fn (): number {
            return item.id;
        };
        let value: number = get_id();
        "#,
    );
    assert!(!has_error(&diagnostics), "Errors: {:?}", diagnostics);
}

// H-117: struct T[] as fn parameter — binder stores ?[], typechecker must update to struct type
#[test]
fn struct_array_fn_param_resolves_correctly() {
    let src = r#"
struct Point { x: number, y: number }

fn sum_x(borrow pts: Point[]): number {
    let mut total: number = 0;
    for p in pts {
        total = total + p.x;
    }
    return total;
}

let arr: Point[] = [Point { x: 1, y: 2 }];
let result: number = sum_x(arr);
"#;
    let diagnostics = typecheck_source(src);
    assert_no_errors(&diagnostics);
}

/// H-406: Cross-module impl method visibility.
/// Typechecker must recognize impl methods on imported struct types.
#[test]
fn test_h406_cross_module_impl_methods_visible() {
    let diags = typecheck_multifile(
        "main",
        &[
            (
                "types",
                r#"
                export struct Counter { value: number }

                impl Counter {
                    fn increment(borrow self): Counter {
                        return Counter { value: self.value + 1 };
                    }
                    fn get(borrow self): number {
                        return self.value;
                    }
                }
                "#,
            ),
            (
                "main",
                r#"
                import { Counter } from "./types";
                let c = Counter { value: 0 };
                let c2 = c.increment();
                let n: number = c2.get();
                "#,
            ),
        ],
    );
    assert!(
        !has_error_crossfile(&diags),
        "Cross-module impl methods should be visible: {:?}",
        diags
    );
}
