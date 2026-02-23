//! typesystem — split into domain submodules (typesystem/ directory)

mod common;

use atlas_runtime::binder::Binder;
use atlas_runtime::diagnostic::{Diagnostic, DiagnosticLevel};
use atlas_runtime::lexer::Lexer;
use atlas_runtime::module_loader::{ModuleLoader, ModuleRegistry};
use atlas_runtime::parser::Parser;
use atlas_runtime::repl::ReplCore;
use atlas_runtime::typechecker::TypeChecker;
use atlas_runtime::{Atlas, TypecheckDump, Value, TYPECHECK_VERSION};
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::fs;
use tempfile::TempDir;

fn typecheck_source(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    let mut binder = Binder::new();
    let (mut table, bind_diags) = binder.bind(&program);
    let mut checker = TypeChecker::new(&mut table);
    let type_diags = checker.check(&program);
    [lex_diags, parse_diags, bind_diags, type_diags].concat()
}

fn typecheck(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize_with_comments();
    if !lex_diags.is_empty() {
        return lex_diags;
    }
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    if !parse_diags.is_empty() {
        return parse_diags;
    }
    let mut binder = Binder::new();
    let (mut table, bind_diags) = binder.bind(&program);
    let mut checker = TypeChecker::new(&mut table);
    let type_diags = checker.check(&program);
    [bind_diags, type_diags].concat()
}

fn errors(source: &str) -> Vec<Diagnostic> {
    typecheck(source)
        .into_iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect()
}

fn warnings(source: &str) -> Vec<Diagnostic> {
    typecheck(source)
        .into_iter()
        .filter(|d| d.level == DiagnosticLevel::Warning)
        .collect()
}

fn has_error(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|d| d.level == DiagnosticLevel::Error)
}

fn has_error_code(diagnostics: &[Diagnostic], code: &str) -> bool {
    diagnostics.iter().any(|d| d.code == code)
}

fn assert_no_errors(diagnostics: &[Diagnostic]) {
    let errs: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        errs.is_empty(),
        "Expected no errors, got: {:?}",
        errs.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

fn assert_has_error(diagnostics: &[Diagnostic], code: &str) {
    assert!(
        !diagnostics.is_empty(),
        "Expected at least one diagnostic with code {}",
        code
    );
    assert!(
        diagnostics.iter().any(|d| d.code == code),
        "Expected diagnostic with code {}, got: {:?}",
        code,
        diagnostics
    );
}

// ============================================================================
// From advanced_inference_tests.rs
// ============================================================================

// Advanced Type Inference - Integration Tests (Phase 07)
//
// Tests for:
// - Bidirectional type checking (synthesis & checking modes)
// - Higher-rank polymorphism
// - Let-polymorphism generalization
// - Flow-sensitive typing
// - Unification algorithm
// - Constraint-based inference
// - Cross-module inference
// - Inference heuristics
// - Complex program integration

// ============================================================================
// Helpers
// ============================================================================

fn has_code(diags: &[Diagnostic], code: &str) -> bool {
    diags.iter().any(|d| d.code == code)
}

// ============================================================================
// Bidirectional Type Checking Tests
// ============================================================================

// Domain submodules (files live in tests/typesystem/)
#[path = "typesystem/bindings.rs"]
mod bindings;
#[path = "typesystem/constraints.rs"]
mod constraints;
#[path = "typesystem/flow.rs"]
mod flow;
#[path = "typesystem/generics.rs"]
mod generics;
#[path = "typesystem/inference.rs"]
mod inference;
#[path = "typesystem/integration.rs"]
mod integration;
