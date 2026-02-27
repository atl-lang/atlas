//! Ownership annotation tests (lines 2419-2705)

use super::*;

// Ownership Keyword Token Tests (Phase 01 — Block 2)
// ============================================================================

#[test]
fn test_ownership_keywords_lex_as_keywords() {
    use atlas_runtime::token::TokenKind;

    for (src, expected) in [
        ("own", TokenKind::Own),
        ("borrow", TokenKind::Borrow),
        ("shared", TokenKind::Shared),
    ] {
        let mut lexer = Lexer::new(src);
        let (tokens, errors) = lexer.tokenize();
        assert!(
            errors.is_empty(),
            "{src}: unexpected lex errors: {errors:?}"
        );
        // tokens: [keyword, EOF]
        assert_eq!(tokens.len(), 2, "{src}: expected 2 tokens (keyword + EOF)");
        assert_eq!(tokens[0].kind, expected, "{src}: wrong token kind");
        assert_eq!(tokens[0].lexeme, src, "{src}: wrong lexeme");
    }
}

#[test]
fn test_ownership_keywords_in_function_signature() {
    use atlas_runtime::token::TokenKind;

    let src = "fn process(own data: number) -> number { return 0; }";
    let mut lexer = Lexer::new(src);
    let (tokens, errors) = lexer.tokenize();
    assert!(errors.is_empty(), "unexpected lex errors: {errors:?}");

    let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind).collect();
    assert!(
        kinds.contains(&TokenKind::Own),
        "expected Own token in: {kinds:?}"
    );
}

#[test]
fn test_ownership_keywords_not_identifiers() {
    use atlas_runtime::token::TokenKind;

    for src in ["own", "borrow", "shared"] {
        let mut lexer = Lexer::new(src);
        let (tokens, _) = lexer.tokenize();
        assert_ne!(
            tokens[0].kind,
            TokenKind::Identifier,
            "{src} should not lex as Identifier"
        );
    }
}

// ============================================================================
// Parser Ownership Annotation Tests (Phase 03 — Block 2)
// ============================================================================

fn parse_fn_params(src: &str) -> Vec<Param> {
    let mut lexer = Lexer::new(src);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, diags) = parser.parse();
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(errors.is_empty(), "parse errors: {errors:?}");
    match program
        .items
        .into_iter()
        .next()
        .expect("expected a function item")
    {
        atlas_runtime::ast::Item::Function(f) => f.params,
        other => panic!("expected function, got {other:?}"),
    }
}

#[test]
fn test_parse_own_param() {
    let params = parse_fn_params("fn process(own data: number) -> number { return data; }");
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].ownership, Some(OwnershipAnnotation::Own));
    assert_eq!(params[0].name.name, "data");
}

#[test]
fn test_parse_borrow_param() {
    let params = parse_fn_params("fn read(borrow data: number) -> number { return data; }");
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].ownership, Some(OwnershipAnnotation::Borrow));
}

#[test]
fn test_parse_shared_param() {
    let params = parse_fn_params("fn share(shared data: number) -> number { return data; }");
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].ownership, Some(OwnershipAnnotation::Shared));
}

#[test]
fn test_parse_unannotated_param_unchanged() {
    let params = parse_fn_params("fn f(x: number) -> number { return x; }");
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].ownership, None);
}

#[test]
fn test_parse_mixed_ownership_params() {
    let params =
        parse_fn_params("fn mixed(own a: number, borrow b: string, c: bool) -> bool { return c; }");
    assert_eq!(params.len(), 3);
    assert_eq!(params[0].ownership, Some(OwnershipAnnotation::Own));
    assert_eq!(params[1].ownership, Some(OwnershipAnnotation::Borrow));
    assert_eq!(params[2].ownership, None);
}

#[test]
fn test_parse_ownership_annotation_error_no_identifier() {
    let src = "fn f(own: number) -> number { return 0; }";
    let mut lexer = Lexer::new(src);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (_, diags) = parser.parse();
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        !errors.is_empty(),
        "expected parse error for 'own' without identifier"
    );
}

// ============================================================================
// Parser Return Type Ownership Annotation Tests (Phase 04 — Block 2)
// ============================================================================

fn parse_fn_decl(src: &str) -> FunctionDecl {
    let mut lexer = Lexer::new(src);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, diags) = parser.parse();
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(errors.is_empty(), "parse errors: {errors:?}");
    match program
        .items
        .into_iter()
        .next()
        .expect("expected a function item")
    {
        atlas_runtime::ast::Item::Function(f) => f,
        other => panic!("expected function, got {other:?}"),
    }
}

#[test]
fn test_parse_own_return_type() {
    let decl = parse_fn_decl("fn allocate(size: number) -> own number { return 0; }");
    assert_eq!(decl.return_ownership, Some(OwnershipAnnotation::Own));
    assert!(matches!(decl.return_type, Some(TypeRef::Named(ref n, _)) if n == "number"));
}

#[test]
fn test_parse_borrow_return_type() {
    let decl = parse_fn_decl("fn peek(borrow arr: number) -> borrow number { return arr; }");
    assert_eq!(decl.return_ownership, Some(OwnershipAnnotation::Borrow));
    assert!(matches!(decl.return_type, Some(TypeRef::Named(ref n, _)) if n == "number"));
}

#[test]
fn test_parse_unannotated_return_type_unchanged() {
    let decl = parse_fn_decl("fn f() -> number { return 1; }");
    assert_eq!(decl.return_ownership, None);
    assert!(matches!(decl.return_type, Some(TypeRef::Named(ref n, _)) if n == "number"));
}

#[test]
fn test_parse_shared_return_type_is_error() {
    let src = "fn bad() -> shared number { return 0; }";
    let mut lexer = Lexer::new(src);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (_, diags) = parser.parse();
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        !errors.is_empty(),
        "expected parse error for `shared` in return annotation position"
    );
}

// ============================================================================
// Block 3: Trait system — token tests
// ============================================================================

#[test]
fn test_trait_impl_keywords_lex_correctly() {
    use atlas_runtime::token::TokenKind;

    for (src, expected) in [("trait", TokenKind::Trait), ("impl", TokenKind::Impl)] {
        let mut lexer = Lexer::new(src);
        let (tokens, errors) = lexer.tokenize();
        assert!(
            errors.is_empty(),
            "{src}: unexpected lex errors: {errors:?}"
        );
        // tokens: [keyword, EOF]
        assert_eq!(tokens.len(), 2, "{src}: expected 2 tokens (keyword + EOF)");
        assert_eq!(tokens[0].kind, expected, "{src}: wrong token kind");
        assert_eq!(tokens[0].lexeme, src, "{src}: wrong lexeme");
    }
}

#[test]
fn test_trait_keyword_is_not_identifier() {
    use atlas_runtime::token::TokenKind;

    let mut lexer = Lexer::new("trait");
    let (tokens, _) = lexer.tokenize();
    assert_ne!(
        tokens[0].kind,
        TokenKind::Identifier,
        "'trait' must not lex as identifier"
    );
    assert_eq!(tokens[0].kind, TokenKind::Trait);
}

#[test]
fn test_impl_keyword_is_not_identifier() {
    use atlas_runtime::token::TokenKind;

    let mut lexer = Lexer::new("impl");
    let (tokens, _) = lexer.tokenize();
    assert_ne!(
        tokens[0].kind,
        TokenKind::Identifier,
        "'impl' must not lex as identifier"
    );
    assert_eq!(tokens[0].kind, TokenKind::Impl);
}

#[test]
fn test_trait_as_variable_name_is_parse_error() {
    // 'trait' is a keyword; using it as a variable name must fail
    let src = "let trait = 1;";
    let mut lexer = Lexer::new(src);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (_, diags) = parser.parse();
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        !errors.is_empty(),
        "expected parse error: `trait` is a keyword, not an identifier"
    );
}

#[test]
fn test_impl_as_variable_name_is_parse_error() {
    // 'impl' is a keyword; using it as a variable name must fail
    let src = "let impl = 1;";
    let mut lexer = Lexer::new(src);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (_, diags) = parser.parse();
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        !errors.is_empty(),
        "expected parse error: `impl` is a keyword, not an identifier"
    );
}

// ============================================================================
