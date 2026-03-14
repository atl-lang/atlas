//! Module re-export syntax tests (H-402)

use super::*;

// ============================================================================
// H-402: export { X } from './module' re-export syntax
// ============================================================================

fn parse_ok(src: &str) -> atlas_runtime::ast::Program {
    let (tokens, lex_diags) = lex(src);
    let lex_errors: Vec<_> = lex_diags.iter().filter(|d| d.is_error()).collect();
    assert!(lex_errors.is_empty(), "lex errors: {:?}", lex_errors);
    let mut parser = Parser::new(tokens);
    let (ast, parse_diags) = parser.parse();
    let parse_errors: Vec<_> = parse_diags.iter().filter(|d| d.is_error()).collect();
    assert!(parse_errors.is_empty(), "parse errors: {:?}", parse_errors);
    ast
}

#[test]
fn test_h402_reexport_single() {
    let ast = parse_ok(r#"export { new_router } from "./src/router";"#);
    assert_eq!(ast.items.len(), 1);
    match &ast.items[0] {
        Item::Export(decl) => match &decl.item {
            ExportItem::ReExport { names, source, .. } => {
                assert_eq!(names.len(), 1);
                assert_eq!(names[0].name.name, "new_router");
                assert_eq!(source, "./src/router");
            }
            _ => panic!("expected ReExport, got {:?}", decl.item),
        },
        _ => panic!("expected Export item"),
    }
}

#[test]
fn test_h402_reexport_multiple() {
    let ast = parse_ok(r#"export { new_router, htmx, render } from "./src/all";"#);
    match &ast.items[0] {
        Item::Export(decl) => match &decl.item {
            ExportItem::ReExport { names, source, .. } => {
                assert_eq!(names.len(), 3);
                assert_eq!(names[0].name.name, "new_router");
                assert_eq!(names[1].name.name, "htmx");
                assert_eq!(names[2].name.name, "render");
                assert_eq!(source, "./src/all");
            }
            _ => panic!("expected ReExport"),
        },
        _ => panic!("expected Export item"),
    }
}

#[test]
fn test_h402_reexport_trailing_comma() {
    // Trailing comma in re-export list should be allowed
    let ast = parse_ok(r#"export { new_router, htmx, } from "./src/all";"#);
    match &ast.items[0] {
        Item::Export(decl) => match &decl.item {
            ExportItem::ReExport { names, .. } => {
                assert_eq!(names.len(), 2);
            }
            _ => panic!("expected ReExport"),
        },
        _ => panic!("expected Export item"),
    }
}

#[test]
fn test_h402_reexport_with_alias() {
    let ast = parse_ok(r#"export { Router as new_router } from "./src/router";"#);
    match &ast.items[0] {
        Item::Export(decl) => match &decl.item {
            ExportItem::ReExport { names, source, .. } => {
                assert_eq!(names.len(), 1);
                assert_eq!(names[0].name.name, "Router");
                assert_eq!(names[0].alias.as_ref().unwrap().name, "new_router");
                assert_eq!(source, "./src/router");
            }
            _ => panic!("expected ReExport"),
        },
        _ => panic!("expected Export item"),
    }
}

#[test]
fn test_h402_reexport_bare_pkg() {
    // Re-export from a bare package specifier (e.g. installed via atlas install)
    let ast = parse_ok(r#"export { Component } from "some-package";"#);
    match &ast.items[0] {
        Item::Export(decl) => match &decl.item {
            ExportItem::ReExport { names, source, .. } => {
                assert_eq!(names.len(), 1);
                assert_eq!(source, "some-package");
            }
            _ => panic!("expected ReExport"),
        },
        _ => panic!("expected Export item"),
    }
}

#[test]
fn test_h402_mixed_exports_and_reexports() {
    // A file can have both regular exports and re-exports
    let ast = parse_ok(
        r#"export fn helper(): string { return "hi"; }
export { Router } from "./router";
export let VERSION: string = "1.0.0";"#,
    );
    assert_eq!(ast.items.len(), 3);
}

// ============================================================================
// H-403: untyped anonymous function params fn(req, res) { ... }
// ============================================================================

#[test]
fn test_h403_untyped_anon_fn_single_param() {
    let ast = parse_ok(r#"let f = fn(x) { return x; };"#);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_h403_untyped_anon_fn_two_params() {
    let ast = parse_ok(r#"let handler = fn(req, res) { return req; };"#);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_h403_mixed_typed_and_untyped_params() {
    // Mixing typed and untyped params should work
    let ast = parse_ok(r#"let f = fn(req, n: number) { return n; };"#);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_h403_typed_anon_fn_still_works() {
    // Fully typed still works
    let ast = parse_ok(r#"let f = fn(x: number, y: number): number { return x; };"#);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_h403_no_params_still_works() {
    let ast = parse_ok(r#"let f = fn() { return 1; };"#);
    assert_eq!(ast.items.len(), 1);
}
