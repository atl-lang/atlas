//! THIN ROUTER — DO NOT ADD TESTS HERE.
//! Add tests to submodule files: tests/frontend_syntax/{lexer,parser_basics,...}.rs

#[allow(unused_imports)]
use atlas_runtime::ast::*;
use atlas_runtime::diagnostic::warnings::{
    config_from_toml, WarningConfig, WarningEmitter, WarningKind, WarningLevel,
};
use atlas_runtime::token::TokenKind;
use atlas_runtime::{Binder, Diagnostic, DiagnosticLevel, Lexer, Parser, Span, TypeChecker};
use rstest::rstest;
use std::fs;
use std::path::Path;

mod common;

// ============================================================================
// Shared Helper Functions
// ============================================================================

fn lex(source: &str) -> (Vec<atlas_runtime::token::Token>, Vec<Diagnostic>) {
    let mut lexer = Lexer::new(source.to_string());
    lexer.tokenize()
}

#[allow(dead_code)]
fn lex_file(filename: &str) -> Vec<Diagnostic> {
    let path = Path::new("tests/errors").join(filename);
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read test file: {}", path.display()));

    let mut lexer = Lexer::new(&source);
    let (_, diagnostics) = lexer.tokenize();
    diagnostics
}

fn parse_source(source: &str) -> (Program, Vec<atlas_runtime::diagnostic::Diagnostic>) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[allow(dead_code)]
fn parse_errors(source: &str) -> Vec<atlas_runtime::diagnostic::Diagnostic> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (_program, diagnostics) = parser.parse();
    diagnostics
}

fn is_parser_error_code(code: &str) -> bool {
    matches!(
        code,
        "AT1000" | "AT1001" | "AT1002" | "AT1003" | "AT1004" | "AT1005"
    )
}

#[allow(dead_code)]
fn assert_has_parser_error(
    diagnostics: &[atlas_runtime::diagnostic::Diagnostic],
    expected_substring: &str,
) {
    assert!(!diagnostics.is_empty(), "Expected at least one diagnostic");
    let expected_lower = expected_substring.to_lowercase();
    let found = diagnostics.iter().any(|d| {
        d.message.to_lowercase().contains(&expected_lower) && is_parser_error_code(&d.code)
    });
    assert!(
        found,
        "Expected parser error with '{}', got: {:?}",
        expected_substring,
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

fn parse_valid(source: &str) -> atlas_runtime::ast::Program {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, diagnostics) = parser.parse();
    assert_eq!(diagnostics.len(), 0, "Should parse without errors");
    program
}

#[allow(dead_code)]
fn assert_parse_error_present(diagnostics: &[atlas_runtime::diagnostic::Diagnostic]) {
    assert!(!diagnostics.is_empty(), "Expected at least one diagnostic");
    let found = diagnostics.iter().any(|d| is_parser_error_code(&d.code));
    assert!(
        found,
        "Expected parser diagnostic, got: {:?}",
        diagnostics
            .iter()
            .map(|d| (&d.code, &d.message))
            .collect::<Vec<_>>()
    );
}

#[allow(dead_code)]
fn assert_error_mentions(diagnostics: &[atlas_runtime::diagnostic::Diagnostic], keywords: &[&str]) {
    assert!(
        diagnostics.iter().any(|d| {
            let msg_lower = d.message.to_lowercase();
            keywords.iter().any(|kw| msg_lower.contains(kw))
        }),
        "Expected error message to mention one of {:?}, got: {:?}",
        keywords,
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

fn try_parse(source: &str) -> Result<Program, Vec<atlas_runtime::Diagnostic>> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, lex_diags) = lexer.tokenize();

    if !lex_diags.is_empty() {
        return Err(lex_diags);
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    if !parse_diags.is_empty() {
        return Err(parse_diags);
    }

    Ok(program)
}

fn parse(source: &str) -> (bool, Vec<String>) {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        return (false, lex_diags.iter().map(|d| d.message.clone()).collect());
    }

    let mut parser = Parser::new(tokens);
    let (_, parse_diags) = parser.parse();

    let success = parse_diags.is_empty();
    let messages = parse_diags.iter().map(|d| d.message.clone()).collect();
    (success, messages)
}

fn get_all_diagnostics(source: &str) -> Vec<atlas_runtime::Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    let mut binder = Binder::new();
    let (mut table, bind_diags) = binder.bind(&program);

    let mut checker = TypeChecker::new(&mut table);
    let type_diags = checker.check(&program);

    let mut all_diags = Vec::new();
    all_diags.extend(lex_diags);
    all_diags.extend(parse_diags);
    all_diags.extend(bind_diags);
    all_diags.extend(type_diags);
    all_diags
}

// ============================================================================
// Submodules
// ============================================================================

#[path = "frontend_syntax/lexer.rs"]
mod lexer;

#[path = "frontend_syntax/parser_basics.rs"]
mod parser_basics;

#[path = "frontend_syntax/parser_errors.rs"]
mod parser_errors;

#[path = "frontend_syntax/operator_precedence_keywords.rs"]
mod operator_precedence_keywords;

#[path = "frontend_syntax/generics.rs"]
mod generics;

#[path = "frontend_syntax/modules_warnings_part1.rs"]
mod modules_warnings_part1;

#[path = "frontend_syntax/warnings_part2.rs"]
mod warnings_part2;

#[path = "frontend_syntax/for_in_traits_part1.rs"]
mod for_in_traits_part1;

#[path = "frontend_syntax/traits_part2.rs"]
mod traits_part2;
