//! Anonymous function parser tests
//!
//! Arrow syntax `(x) => expr` was removed in Decision D-003.
//! This file tests that related constructs still work correctly.

use super::*;

fn parse_expr(source: &str) -> (Vec<Item>, Vec<Diagnostic>) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, diags) = parser.parse();
    (program.items, diags)
}

#[test]
fn test_grouped_expr_parses_correctly() {
    // `(x + 1)` must parse as grouped expression
    let (items, diags) = parse_expr("let v = (x + 1);");
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "{:?}",
        diags
    );
    if let Item::Statement(Stmt::VarDecl(decl)) = &items[0] {
        assert!(
            matches!(&decl.init, Expr::Group(_)),
            "expected Expr::Group, got: {:?}",
            decl.init
        );
    }
}

#[test]
fn test_match_arm_fat_arrow_works() {
    // Match arms with `=>` still parse correctly
    let source = "let v = match x { 1 => true, _ => false };";
    let (items, diags) = parse_expr(source);
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "{:?}",
        diags
    );
    assert_eq!(items.len(), 1);
    if let Item::Statement(Stmt::VarDecl(decl)) = &items[0] {
        assert!(matches!(&decl.init, Expr::Match(_)), "expected Match expr");
    }
}

#[test]
fn test_fn_style_anon_fn_parses() {
    // `fn(x: number) { x + 1 }` anonymous function with typed param
    let (items, diags) = parse_expr("let double = fn(x: number): number { x + 1 };");
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "unexpected errors: {:?}",
        diags
    );
    assert_eq!(items.len(), 1);
    if let Item::Statement(Stmt::VarDecl(decl)) = &items[0] {
        if let Expr::AnonFn {
            params,
            return_type,
            ..
        } = &decl.init
        {
            assert_eq!(params.len(), 1, "expected 1 param");
            assert_eq!(params[0].name.name, "x");
        } else {
            panic!("expected Expr::AnonFn, got: {:?}", decl.init);
        }
    } else {
        panic!("expected VarDecl");
    }
}

#[test]
fn test_fn_style_anon_fn_with_typed_params() {
    // `fn(x: number) -> number { x * 2 }` with explicit types
    let (items, diags) = parse_expr("let double = fn(borrow x: number): number { x * 2 };");
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "unexpected errors: {:?}",
        diags
    );
    if let Item::Statement(Stmt::VarDecl(decl)) = &items[0] {
        if let Expr::AnonFn {
            params,
            return_type,
            ..
        } = &decl.init
        {
            assert_eq!(params.len(), 1);
            assert!(
                !matches!(params[0].type_ref, TypeRef::SelfType(_)),
                "expected typed param"
            );
            assert!(return_type.is_some(), "expected return type");
        } else {
            panic!("expected AnonFn");
        }
    }
}

#[test]
fn test_fn_style_anon_fn_empty_params() {
    // `fn() { 42 }` with no params
    let (items, diags) = parse_expr("let f = fn() { 42 };");
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "unexpected errors: {:?}",
        diags
    );
    if let Item::Statement(Stmt::VarDecl(decl)) = &items[0] {
        if let Expr::AnonFn { params, .. } = &decl.init {
            assert_eq!(params.len(), 0, "expected 0 params");
        } else {
            panic!("expected AnonFn");
        }
    }
}

#[test]
fn test_fn_style_anon_fn_as_call_argument() {
    // fn-style anon fn as direct call argument
    let (items, diags) = parse_expr("apply(fn(x: number): number { x * 2 });");
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "{:?}",
        diags
    );
    if let Item::Statement(Stmt::Expr(expr_stmt)) = &items[0] {
        if let Expr::Call(call) = &expr_stmt.expr {
            assert_eq!(call.args.len(), 1);
            assert!(
                matches!(&call.args[0], Expr::AnonFn { .. }),
                "expected AnonFn arg"
            );
        } else {
            panic!("expected Call");
        }
    }
}
