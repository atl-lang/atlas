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
// H-076: Trait inheritance — trait B: A syntax
// ============================================================================

/// Helper: run typechecker on source, return error diagnostic strings.
fn typecheck_errors(source: &str) -> Vec<String> {
    use atlas_runtime::binder::Binder;
    use atlas_runtime::typechecker::TypeChecker;
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_errors) = parser.parse();
    let parse_errs: Vec<_> = parse_errors
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(parse_errs.is_empty(), "parse errors: {:?}", parse_errs);
    let mut binder = Binder::new();
    let (mut table, _) = binder.bind(&program);
    let mut checker = TypeChecker::new(&mut table);
    let diags = checker.check(&program);
    diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect()
}

/// Helper: parse source and assert no parse errors.
fn assert_parses(source: &str) {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (_, diags) = parser.parse();
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(errors.is_empty(), "Unexpected parse errors: {:?}", errors);
}

#[test]
fn test_h076_supertrait_parses() {
    assert_parses(
        r#"
        trait A {
            fn foo(borrow self: A) -> number;
        }
        trait B: A {
            fn bar(borrow self: B) -> string;
        }
        "#,
    );
}

#[test]
fn test_h076_multiple_supertraits_parse() {
    assert_parses(
        r#"
        trait A { fn a(borrow self: A) -> number; }
        trait B { fn b(borrow self: B) -> string; }
        trait C: A + B { fn c(borrow self: C) -> bool; }
        "#,
    );
}

#[test]
fn test_h076_impl_of_subtrait_requires_supertrait_methods() {
    let errors = typecheck_errors(
        r#"
        struct MyType { x: number }
        trait A { fn foo(borrow self: A) -> number; }
        trait B: A { fn bar(borrow self: B) -> string; }
        impl B for MyType {
            fn bar(borrow self: MyType) -> string { "hello"; }
        }
        "#,
    );
    assert!(
        !errors.is_empty(),
        "Expected error: impl B for MyType missing inherited method 'foo' from A"
    );
}

#[test]
fn test_h076_impl_with_all_methods_is_ok() {
    let errors = typecheck_errors(
        r#"
        struct MyType { x: number }
        trait A { fn foo(borrow self: A) -> number; }
        trait B: A {
            fn bar(borrow self: B) -> string;
        }
        impl B for MyType {
            fn foo(borrow self: MyType) -> number { 42; }
            fn bar(borrow self: MyType) -> string { "hello"; }
        }
        "#,
    );
    assert!(
        errors.is_empty(),
        "Expected no errors when all methods satisfied, borrow got: {:?}",
        errors
    );
}

#[test]
fn test_h076_undefined_supertrait_is_error() {
    let errors = typecheck_errors(
        r#"
        trait B: NonExistent {
            fn bar(borrow self: B) -> string;
        }
        "#,
    );
    assert!(
        !errors.is_empty(),
        "Expected error: supertrait 'NonExistent' is not defined"
    );
}

// ============================================================================
// H-077: Generic traits — trait Container<T> syntax
// ============================================================================

#[test]
fn test_h077_generic_trait_parses() {
    assert_parses(
        r#"
        trait Container<T> {
            fn get(borrow self: Container<T>) -> T;
            fn set(borrow self: Container<T>, borrow value: T) -> Container<T>;
        }
        "#,
    );
}

#[test]
fn test_h077_impl_generic_trait_with_concrete_type_is_ok() {
    let errors = typecheck_errors(
        r#"
        struct Box { value: number }
        trait Container<T> {
            fn get(borrow self: Container<T>) -> T;
        }
        impl Container<number> for Box {
            fn get(borrow self: Box) -> number { self.value; }
        }
        "#,
    );
    assert!(
        errors.is_empty(),
        "Expected no errors for valid generic trait impl, borrow got: {:?}",
        errors
    );
}

#[test]
fn test_h077_impl_generic_trait_wrong_return_type_is_error() {
    let errors = typecheck_errors(
        r#"
        struct Box { value: number }
        trait Container<T> {
            fn get(borrow self: Container<T>) -> T;
        }
        impl Container<number> for Box {
            fn get(borrow self: Box) -> string { "wrong"; }
        }
        "#,
    );
    assert!(
        !errors.is_empty(),
        "Expected error: return type 'string' doesn't match Container<number>.get -> number"
    );
}

#[test]
fn test_h077_generic_trait_with_string_type_arg_is_ok() {
    let errors = typecheck_errors(
        r#"
        struct StrBox { value: string }
        trait Container<T> {
            fn get(borrow self: Container<T>) -> T;
        }
        impl Container<string> for StrBox {
            fn get(borrow self: StrBox) -> string { self.value; }
        }
        "#,
    );
    assert!(
        errors.is_empty(),
        "Expected no errors for Container<string> impl, borrow got: {:?}",
        errors
    );
}
