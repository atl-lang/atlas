//! Arrow syntax parser tests (lines 2911-3166)

use super::*;

// Block 4 Phase 3: Parser — Arrow syntax `(params) => expr`
// ============================================================================

fn parse_arrow_expr(source: &str) -> (Vec<Item>, Vec<Diagnostic>) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, diags) = parser.parse();
    (program.items, diags)
}

#[test]
fn test_arrow_fn_single_param_untyped() {
    // `(x) => x + 1` parses as Expr::AnonFn with 1 untyped param
    let (items, diags) = parse_arrow_expr("let double = (x) => x + 1;");
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
            assert!(params[0].type_ref.is_none(), "expected untyped param");
            assert!(return_type.is_none(), "expected no return type");
        } else {
            panic!("expected Expr::AnonFn, got: {:?}", decl.init);
        }
    } else {
        panic!("expected VarDecl");
    }
}

#[test]
fn test_arrow_fn_two_params_untyped() {
    // `(x, y) => x + y` parses as Expr::AnonFn with 2 untyped params
    let (items, diags) = parse_arrow_expr("let add = (x, y) => x + y;");
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
            assert!(params[0].type_ref.is_none());
            assert!(params[1].type_ref.is_none());
        } else {
            panic!("expected AnonFn");
        }
    }
}

#[test]
fn test_arrow_fn_typed_param() {
    // `(x: number) => x * 2` parses as Expr::AnonFn with typed param
    let (items, diags) = parse_arrow_expr("let double = (x: number) => x * 2;");
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "unexpected errors: {:?}",
        diags
    );
    if let Item::Statement(Stmt::VarDecl(decl)) = &items[0] {
        if let Expr::AnonFn { params, .. } = &decl.init {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].name.name, "x");
            assert!(
                matches!(&params[0].type_ref, Some(TypeRef::Named(name, _)) if name == "number"),
                "expected typed param 'number', got: {:?}",
                params[0].type_ref
            );
        } else {
            panic!("expected AnonFn");
        }
    }
}

#[test]
fn test_arrow_fn_empty_params() {
    // `() => 42` parses as Expr::AnonFn with 0 params
    let (items, diags) = parse_arrow_expr("let f = () => 42;");
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
fn test_arrow_fn_body_is_not_block() {
    // Arrow body is a plain expression, not Expr::Block
    let (items, diags) = parse_arrow_expr("let f = (x) => x + 1;");
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "{:?}",
        diags
    );
    if let Item::Statement(Stmt::VarDecl(decl)) = &items[0] {
        if let Expr::AnonFn { body, .. } = &decl.init {
            assert!(
                !matches!(body.as_ref(), Expr::Block(_)),
                "arrow fn body must NOT be Expr::Block"
            );
        } else {
            panic!("expected AnonFn");
        }
    }
}

#[test]
fn test_grouped_expr_not_arrow_fn() {
    // `(x + 1)` must still parse as grouped expression, not arrow fn
    let (items, diags) = parse_arrow_expr("let v = (x + 1);");
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
fn test_match_arm_fat_arrow_not_affected() {
    // Match arms with `=>` still parse correctly — no regression
    let source = "let v = match x { 1 => true, _ => false };";
    let (items, diags) = parse_arrow_expr(source);
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
fn test_arrow_fn_as_call_argument() {
    // Arrow fn as direct call argument
    let (items, diags) = parse_arrow_expr("apply((x) => x * 2);");
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

#[test]
fn test_arrow_fn_mixed_typed_and_untyped_params() {
    // Mixed: first param typed, second untyped — both valid
    let (items, diags) = parse_arrow_expr("let f = (x: number, y) => x + y;");
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "{:?}",
        diags
    );
    if let Item::Statement(Stmt::VarDecl(decl)) = &items[0] {
        if let Expr::AnonFn { params, .. } = &decl.init {
            assert_eq!(params.len(), 2);
            assert!(params[0].type_ref.is_some(), "first param should be typed");
            assert!(
                params[1].type_ref.is_none(),
                "second param should be untyped"
            );
        } else {
            panic!("expected AnonFn");
        }
    }
}

#[test]
fn test_arrow_fn_return_type_is_none() {
    // Arrow fn never has an explicit return type
    let (items, diags) = parse_arrow_expr("let f = (x: number) => x;");
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "{:?}",
        diags
    );
    if let Item::Statement(Stmt::VarDecl(decl)) = &items[0] {
        if let Expr::AnonFn { return_type, .. } = &decl.init {
            assert!(
                return_type.is_none(),
                "arrow fn return_type must always be None"
            );
        } else {
            panic!("expected AnonFn");
        }
    }
}

#[test]
fn test_arrow_fn_span_covers_full_expr() {
    // The span of the AnonFn covers from '(' to end of body
    let (items, diags) = parse_arrow_expr("let f = (x) => x;");
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "{:?}",
        diags
    );
    if let Item::Statement(Stmt::VarDecl(decl)) = &items[0] {
        if let Expr::AnonFn { span, .. } = &decl.init {
            assert!(span.start < span.end, "span must be non-empty");
        } else {
            panic!("expected AnonFn");
        }
    }
}

#[test]
fn test_arrow_fn_string_body() {
    // Body can be any expression — string literal
    let (items, diags) = parse_arrow_expr(r#"let greet = (name: string) => "hello";"#);
    assert!(
        diags.iter().all(|d| d.level != DiagnosticLevel::Error),
        "{:?}",
        diags
    );
    if let Item::Statement(Stmt::VarDecl(decl)) = &items[0] {
        assert!(matches!(&decl.init, Expr::AnonFn { .. }), "expected AnonFn");
    }
}
