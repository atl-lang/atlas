//! THIN ROUTER — DO NOT ADD TESTS HERE.
//! Add tests to submodule files: tests/frontend_integration/{part_1,part_2,...}.rs

use atlas_formatter::{
    check_formatted, check_formatted_with_config, format_source, format_source_with_config,
    FormatConfig, FormatResult,
};
use atlas_runtime::ast::*;
use atlas_runtime::bytecode::{validate, Bytecode, Opcode, ValidationErrorKind};
use atlas_runtime::diagnostic::error_codes;
use atlas_runtime::diagnostic::formatter::{
    enrich_diagnostic, extract_snippet, offset_to_line_col, DiagnosticFormatter,
};
use atlas_runtime::diagnostic::normalizer::normalize_diagnostic_for_testing;
use atlas_runtime::diagnostic::warnings::{
    WarningConfig, WarningEmitter, WarningKind, WarningLevel,
};
use atlas_runtime::value::Value;
use atlas_runtime::{
    sort_diagnostics, Diagnostic, DiagnosticLevel, Lexer, Parser, Span, DIAG_VERSION,
};
use rstest::rstest;
use std::path::Path;

mod common;

// ============================================================================
// Shared Helper Functions
// ============================================================================

// Helper Functions
// ============================================================

/// Generate an absolute path that works on the current platform
#[cfg(unix)]
fn absolute_test_path(filename: &str) -> String {
    format!("/absolute/path/{}", filename)
}

#[cfg(windows)]
fn absolute_test_path(filename: &str) -> String {
    format!("C:\\absolute\\path\\{}", filename)
}

/// Check if a path looks absolute (cross-platform)
fn is_absolute_path(path: &str) -> bool {
    Path::new(path).is_absolute()
}

/// Parse source and return (AST success, parse error diagnostics only)
fn parse_source(source: &str) -> (bool, Vec<Diagnostic>) {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, diags) = parser.parse();
    let has_items = !program.items.is_empty();
    // Filter to only error-level diagnostics (parser may emit warnings)
    let errors: Vec<Diagnostic> = diags
        .into_iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    (has_items || errors.is_empty(), errors)
}

/// Format source and return formatted string, or panic on error
fn fmt(source: &str) -> String {
    match format_source(source) {
        FormatResult::Ok(s) => s,
        FormatResult::ParseError(e) => panic!("Parse error: {:?}", e),
    }
}

/// Format source and return FormatResult directly
fn try_fmt(source: &str) -> FormatResult {
    format_source(source)
}


// ============================================================================
// Submodules
// ============================================================================

#[path = "frontend_integration/integration_part_1.rs"]
mod integration_part_1;

#[path = "frontend_integration/integration_part_2.rs"]
mod integration_part_2;

#[path = "frontend_integration/integration_part_3.rs"]
mod integration_part_3;

#[path = "frontend_integration/integration_part_4.rs"]
mod integration_part_4;

#[path = "frontend_integration/integration_part_5.rs"]
mod integration_part_5;

#[path = "frontend_integration/ast_part_1.rs"]
mod ast_part_1;

#[path = "frontend_integration/ast_part_2.rs"]
mod ast_part_2;

#[path = "frontend_integration/bytecode_validator.rs"]
mod bytecode_validator;

#[path = "frontend_integration/ownership.rs"]
mod ownership;

#[path = "frontend_integration/traits.rs"]
mod traits;

#[path = "frontend_integration/anonfn_part_1.rs"]
mod anonfn_part_1;

#[path = "frontend_integration/anonfn_part_2.rs"]
mod anonfn_part_2;
