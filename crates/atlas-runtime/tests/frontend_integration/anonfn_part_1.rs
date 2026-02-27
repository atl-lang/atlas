//! AnonFn parser tests (lines 2706-2910)

use super::*;

// Block 4 Phase 2: Parser — fn expression syntax (AnonFn)
// ============================================================================

fn parse_anon_fn_expr(source: &str) -> (Vec<Item>, Vec<Diagnostic>) {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, diags) = parser.parse();
    (program.items, diags)
}

#[test]
fn test_anon_fn_basic_parses_as_anon_fn_node() {
    let (items, diags) = parse_anon_fn_expr("let f = fn(x: number) -> number { x; };");
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "unexpected errors: {:?}",
        diags
    );
    assert_eq!(items.len(), 1);
    if let Item::Statement(Stmt::VarDecl(decl)) = &items[0] {
        assert!(
            matches!(&decl.init, Expr::AnonFn { params, .. } if params.len() == 1),
            "expected Expr::AnonFn with 1 param, got: {:?}",
            decl.init
        );
    } else {
        panic!("expected var decl, got: {:?}", items[0]);
    }
}

#[test]
fn test_anon_fn_return_type_present() {
    let (items, diags) = parse_anon_fn_expr("let f = fn(x: number) -> number { x; };");
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "unexpected errors: {:?}",
        diags
    );
    if let Item::Statement(Stmt::VarDecl(decl)) = &items[0] {
        if let Expr::AnonFn { return_type, .. } = &decl.init {
            assert!(return_type.is_some(), "expected return_type = Some");
        } else {
            panic!("expected AnonFn");
        }
    }
}

#[test]
fn test_anon_fn_no_return_type_is_none() {
    let (items, diags) = parse_anon_fn_expr("let f = fn(x: number) { x; };");
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "unexpected errors: {:?}",
        diags
    );
    if let Item::Statement(Stmt::VarDecl(decl)) = &items[0] {
        if let Expr::AnonFn { return_type, .. } = &decl.init {
            assert!(return_type.is_none(), "expected return_type = None");
        } else {
            panic!("expected AnonFn");
        }
    }
}

#[test]
fn test_anon_fn_no_params() {
    let (items, diags) = parse_anon_fn_expr("let f = fn() -> number { 42; };");
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
fn test_anon_fn_multiple_params() {
    let (items, diags) = parse_anon_fn_expr("let f = fn(x: number, y: string) -> bool { true; };");
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "unexpected errors: {:?}",
        diags
    );
    if let Item::Statement(Stmt::VarDecl(decl)) = &items[0] {
        if let Expr::AnonFn { params, .. } = &decl.init {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].name.name, "x");
            assert_eq!(params[1].name.name, "y");
        } else {
            panic!("expected AnonFn");
        }
    }
}

#[test]
fn test_anon_fn_ownership_annotation_on_param() {
    let (items, diags) = parse_anon_fn_expr("let f = fn(own x: number) -> number { x; };");
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "unexpected errors: {:?}",
        diags
    );
    if let Item::Statement(Stmt::VarDecl(decl)) = &items[0] {
        if let Expr::AnonFn { params, .. } = &decl.init {
            assert_eq!(params.len(), 1);
            assert!(
                matches!(params[0].ownership, Some(OwnershipAnnotation::Own)),
                "expected Own annotation"
            );
        } else {
            panic!("expected AnonFn");
        }
    }
}

#[test]
fn test_anon_fn_borrow_annotation_on_param() {
    let (items, diags) = parse_anon_fn_expr("let f = fn(borrow x: number) -> number { x; };");
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "unexpected errors: {:?}",
        diags
    );
    if let Item::Statement(Stmt::VarDecl(decl)) = &items[0] {
        if let Expr::AnonFn { params, .. } = &decl.init {
            assert!(matches!(
                params[0].ownership,
                Some(OwnershipAnnotation::Borrow)
            ));
        } else {
            panic!("expected AnonFn");
        }
    }
}

#[test]
fn test_anon_fn_body_is_block_expr() {
    let (items, diags) = parse_anon_fn_expr("let f = fn(x: number) -> number { x; };");
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "unexpected errors: {:?}",
        diags
    );
    if let Item::Statement(Stmt::VarDecl(decl)) = &items[0] {
        if let Expr::AnonFn { body, .. } = &decl.init {
            assert!(
                matches!(body.as_ref(), Expr::Block(_)),
                "expected body to be Expr::Block"
            );
        } else {
            panic!("expected AnonFn");
        }
    }
}

#[test]
fn test_anon_fn_as_call_argument() {
    let (items, diags) = parse_anon_fn_expr("apply(fn(x: number) -> number { x; });");
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "unexpected errors: {:?}",
        diags
    );
    assert_eq!(items.len(), 1);
    if let Item::Statement(Stmt::Expr(expr_stmt)) = &items[0] {
        if let Expr::Call(call) = &expr_stmt.expr {
            assert_eq!(call.args.len(), 1);
            assert!(
                matches!(&call.args[0], Expr::AnonFn { .. }),
                "expected AnonFn as call arg"
            );
        } else {
            panic!("expected Call");
        }
    }
}

#[test]
fn test_anon_fn_missing_paren_produces_diagnostic() {
    let (_, diags) = parse_anon_fn_expr("let f = fn x: number) -> number { x; };");
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(!errors.is_empty(), "expected parse error for missing '('");
}

#[test]
fn test_anon_fn_missing_body_brace_produces_diagnostic() {
    let (_, diags) = parse_anon_fn_expr("let f = fn(x: number) -> number x; };");
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(!errors.is_empty(), "expected parse error for missing '{{'");
}

// ============================================================================
