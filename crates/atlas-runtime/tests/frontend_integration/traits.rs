//! Trait system token tests (lines 2620, 2706-2619)

use super::*;

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
