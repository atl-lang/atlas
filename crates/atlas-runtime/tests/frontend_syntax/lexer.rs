//! Lexer tests (lines 1-277 from original frontend_syntax.rs)

use super::*;

// ============================================================================
// String Literal Tests - Parameterized with rstest
// ============================================================================

#[rstest]
#[case(r#""hello world""#, "hello world")]
#[case(r#""line1\nline2\ttab\r\n""#, "line1\nline2\ttab\r\n")]
#[case(r#""He said \"hello\"""#, r#"He said "hello""#)]
#[case(r#""path\\to\\file""#, r"path\to\file")]
#[case("\"line1\nline2\nline3\"", "line1\nline2\nline3")]
fn test_string_literals_valid(#[case] input: &str, #[case] expected: &str) {
    let (tokens, diagnostics) = lex(input);

    assert_eq!(diagnostics.len(), 0, "Should have no errors");
    assert_eq!(tokens[0].kind, TokenKind::String);
    assert_eq!(tokens[0].lexeme, expected);
}

#[rstest]
#[case(r#""unterminated string"#, "Unterminated")]
#[case(r#""invalid\xescape""#, "Invalid escape")]
fn test_string_literals_errors(#[case] input: &str, #[case] error_message: &str) {
    let (tokens, diagnostics) = lex(input);

    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert!(!diagnostics.is_empty(), "Should have errors");
    assert!(
        diagnostics[0].message.contains(error_message),
        "Expected error containing '{}', got '{}'",
        error_message,
        diagnostics[0].message
    );
}

// ============================================================================
// Number Literal Tests - Table-driven
// ============================================================================

#[rstest]
#[case("0", "0")]
#[case("1", "1")]
#[case("42", "42")]
#[case("999", "999")]
#[case("1234567890", "1234567890")]
#[case("0.0", "0.0")]
#[case("3.14", "3.14")]
#[case("99.999", "99.999")]
#[case("0.5", "0.5")]
fn test_number_literals(#[case] input: &str, #[case] expected: &str) {
    let (tokens, diagnostics) = lex(input);

    assert_eq!(diagnostics.len(), 0, "Should have no errors");
    assert_eq!(tokens[0].kind, TokenKind::Number);
    assert_eq!(tokens[0].lexeme, expected);
}

// ============================================================================
// Keyword Tests - Single parameterized test instead of 20+ individual tests
// ============================================================================

#[rstest]
#[case("let", TokenKind::Let)]
#[case("var", TokenKind::Var)]
#[case("fn", TokenKind::Fn)]
#[case("if", TokenKind::If)]
#[case("else", TokenKind::Else)]
#[case("while", TokenKind::While)]
#[case("for", TokenKind::For)]
#[case("return", TokenKind::Return)]
#[case("break", TokenKind::Break)]
#[case("continue", TokenKind::Continue)]
#[case("true", TokenKind::True)]
#[case("false", TokenKind::False)]
#[case("null", TokenKind::Null)]
fn test_keywords(#[case] keyword: &str, #[case] expected_kind: TokenKind) {
    let (tokens, diagnostics) = lex(keyword);

    assert_eq!(diagnostics.len(), 0);
    assert_eq!(tokens[0].kind, expected_kind);
    assert_eq!(tokens[0].lexeme, keyword);
}

// ============================================================================
// Operator Tests
// ============================================================================

#[rstest]
#[case("+", TokenKind::Plus)]
#[case("-", TokenKind::Minus)]
#[case("*", TokenKind::Star)]
#[case("/", TokenKind::Slash)]
#[case("%", TokenKind::Percent)]
#[case("==", TokenKind::EqualEqual)]
#[case("!=", TokenKind::BangEqual)]
#[case("<", TokenKind::Less)]
#[case("<=", TokenKind::LessEqual)]
#[case(">", TokenKind::Greater)]
#[case(">=", TokenKind::GreaterEqual)]
#[case("&&", TokenKind::AmpAmp)]
#[case("||", TokenKind::PipePipe)]
#[case("!", TokenKind::Bang)]
#[case("=", TokenKind::Equal)]
#[case("+=", TokenKind::PlusEqual)]
#[case("-=", TokenKind::MinusEqual)]
#[case("*=", TokenKind::StarEqual)]
#[case("/=", TokenKind::SlashEqual)]
#[case("%=", TokenKind::PercentEqual)]
#[case("++", TokenKind::PlusPlus)]
#[case("--", TokenKind::MinusMinus)]
fn test_operators(#[case] operator: &str, #[case] expected_kind: TokenKind) {
    let (tokens, diagnostics) = lex(operator);

    assert_eq!(diagnostics.len(), 0);
    assert_eq!(tokens[0].kind, expected_kind);
}

// ============================================================================
// Comment Tests
// ============================================================================

#[rstest]
#[case("// single line comment\n", 1)] // Just EOF
#[case("/* block comment */", 1)] // Just EOF
#[case("/* multi\nline\ncomment */", 1)]
#[case("let x = 1; // comment", 6)] // let x = 1 ; EOF (6 tokens)
fn test_comments_ignored(#[case] input: &str, #[case] expected_token_count: usize) {
    let (tokens, diagnostics) = lex(input);

    assert_eq!(diagnostics.len(), 0);
    assert_eq!(tokens.len(), expected_token_count);
}

// ============================================================================
// Integration Test - Complex Expression
// ============================================================================

#[test]
fn test_complex_expression() {
    let source = r#"fn add(a: number, b: number) -> number { return a + b; }"#;
    let (tokens, diagnostics) = lex(source);

    assert_eq!(diagnostics.len(), 0, "Should lex without errors");

    // Verify token sequence
    let expected_kinds = vec![
        TokenKind::Fn,
        TokenKind::Identifier,
        TokenKind::LeftParen,
        TokenKind::Identifier,
        TokenKind::Colon,
        TokenKind::Identifier,
        TokenKind::Comma,
        TokenKind::Identifier,
        TokenKind::Colon,
        TokenKind::Identifier,
        TokenKind::RightParen,
        TokenKind::Arrow,
        TokenKind::Identifier,
        TokenKind::LeftBrace,
        TokenKind::Return,
        TokenKind::Identifier,
        TokenKind::Plus,
        TokenKind::Identifier,
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::Eof,
    ];

    for (i, expected_kind) in expected_kinds.iter().enumerate() {
        assert_eq!(
            tokens[i].kind, *expected_kind,
            "Token {} should be {:?}, got {:?}",
            i, expected_kind, tokens[i].kind
        );
    }
}

// ============================================================================
// Lexer Golden Tests (from lexer_golden_tests.rs)
// ============================================================================

fn lex_file(filename: &str) -> Vec<Diagnostic> {
    let path = Path::new("tests/errors").join(filename);
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read test file: {}", path.display()));

    let mut lexer = Lexer::new(&source);
    let (_, diagnostics) = lexer.tokenize();
    diagnostics
}

// ============================================================================
// Individual Error File Tests with Snapshots
// ============================================================================

#[rstest]
#[case("unterminated_string.atl", "AT1002")]
#[case("invalid_escape.atl", "AT1003")]
#[case("unexpected_char.atl", "AT1001")]
#[case("unterminated_comment.atl", "AT1004")]
fn test_lexer_error_files(#[case] filename: &str, #[case] expected_code: &str) {
    let diagnostics = lex_file(filename);

    // Verify we got the expected error
    assert!(
        !diagnostics.is_empty(),
        "Expected diagnostics for {}",
        filename
    );
    assert!(
        diagnostics.iter().any(|d| d.code == expected_code),
        "Expected error code {} in {}, got: {:?}",
        expected_code,
        filename,
        diagnostics.iter().map(|d| &d.code).collect::<Vec<_>>()
    );

    // Snapshot the diagnostics for stability tracking
    insta::assert_yaml_snapshot!(
        format!("lexer_error_{}", filename.replace(".atl", "")),
        diagnostics
    );
}

// ============================================================================
// Stability Test
// ============================================================================

#[test]
fn test_diagnostic_stability() {
    // Verify that running the same file twice produces identical diagnostics
    let diag1 = lex_file("unterminated_string.atl");
    let diag2 = lex_file("unterminated_string.atl");

    assert_eq!(
        diag1.len(),
        diag2.len(),
        "Diagnostic count should be stable"
    );
    for (d1, d2) in diag1.iter().zip(diag2.iter()) {
        assert_eq!(d1.code, d2.code, "Diagnostic codes should be stable");
        assert_eq!(
            d1.message, d2.message,
            "Diagnostic messages should be stable"
        );
        assert_eq!(d1.line, d2.line, "Diagnostic lines should be stable");
        assert_eq!(d1.column, d2.column, "Diagnostic columns should be stable");
    }
}

// ============================================================================
