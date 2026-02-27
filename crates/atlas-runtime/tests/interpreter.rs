//! THIN ROUTER — DO NOT ADD TESTS HERE.
//! Add tests to the submodule files: tests/interpreter/{member,nested_functions,scope,pattern_matching,assignment,for_in,integration}.rs
//! This file only declares submodules and shared helpers.

mod common;

use atlas_runtime::binder::Binder;
use atlas_runtime::diagnostic::{Diagnostic, DiagnosticLevel};
use atlas_runtime::interpreter::Interpreter;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::typechecker::TypeChecker;
use atlas_runtime::value::Value;
use atlas_runtime::Atlas;
use common::*;
use rstest::rstest;

fn run_interpreter(source: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);
    let mut interpreter = Interpreter::new();
    match interpreter.eval(&program, &SecurityContext::allow_all()) {
        Ok(value) => Ok(format!("{:?}", value)),
        Err(e) => Err(format!("{:?}", e)),
    }
}

// Domain submodules (files live in tests/interpreter/)
#[path = "interpreter/assignment.rs"]
mod interp_assignment;
#[path = "interpreter/for_in.rs"]
mod interp_for_in;
#[path = "interpreter/integration.rs"]
mod interp_integration;
#[path = "interpreter/member.rs"]
mod interp_member;
#[path = "interpreter/nested_functions.rs"]
mod interp_nested_functions;
#[path = "interpreter/pattern_matching.rs"]
mod interp_pattern_matching;
#[path = "interpreter/scope.rs"]
mod interp_scope;
