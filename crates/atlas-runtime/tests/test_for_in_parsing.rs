//! Tests for for-in loop parsing (Phase-20a)
//!
//! These tests verify that for-in syntax parses correctly.
//! Execution tests will be added in Phase-20d.

use atlas_runtime::{Lexer, Parser};

#[test]
fn test_parse_for_in_basic() {
    let source = r#"
        for item in array {
            print(item);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty(), "Lexer should not produce errors");

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(parse_diags.is_empty(), "Should parse for-in loop");
}

#[test]
fn test_parse_for_in_with_array_literal() {
    let source = r#"
        for x in [1, 2, 3] {
            print(x);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        parse_diags.is_empty(),
        "Should parse for-in with array literal"
    );
}

#[test]
fn test_parse_for_in_empty_body() {
    let source = r#"
        for x in arr {
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        parse_diags.is_empty(),
        "Should parse for-in with empty body"
    );
}

#[test]
fn test_parse_for_in_nested() {
    let source = r#"
        for outer in outerArray {
            for inner in innerArray {
                print(inner);
            }
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(parse_diags.is_empty(), "Should parse nested for-in loops");
}

#[test]
fn test_parse_for_in_with_function_call() {
    let source = r#"
        for item in getArray() {
            print(item);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        parse_diags.is_empty(),
        "Should parse for-in with function call"
    );
}

#[test]
fn test_parse_for_in_error_missing_in() {
    let source = r#"
        for item array {
            print(item);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(!parse_diags.is_empty(), "Should error without 'in' keyword");
}

#[test]
fn test_parse_for_in_error_missing_variable() {
    let source = r#"
        for in array {
            print(x);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        !parse_diags.is_empty(),
        "Should error without variable name"
    );
}

#[test]
fn test_traditional_for_still_works() {
    let source = r#"
        for (let i = 0; i < 10; i = i + 1) {
            print(i);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        parse_diags.is_empty(),
        "Traditional for loops should still work"
    );
}

#[test]
fn test_parse_for_in_with_method_call() {
    let source = r#"
        for item in obj.getItems() {
            print(item);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        parse_diags.is_empty(),
        "Should parse for-in with method call"
    );
}

#[test]
fn test_parse_for_in_with_complex_body() {
    let source = r#"
        for item in items {
            if (item > 5) {
                print("Large: " + toString(item));
            } else {
                print("Small: " + toString(item));
            }
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        parse_diags.is_empty(),
        "Should parse for-in with complex body"
    );
}
