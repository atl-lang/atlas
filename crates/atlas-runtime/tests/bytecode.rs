//! bytecode — router for bytecode compiler, optimizer, profiler, and parity tests

mod common;

use atlas_runtime::binder::Binder;
use atlas_runtime::bytecode::{Bytecode, Opcode};
use atlas_runtime::compiler::Compiler;
use atlas_runtime::interpreter::Interpreter;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::optimizer::{
    ConstantFoldingPass, DeadCodeEliminationPass, OptimizationPass, OptimizationStats, Optimizer,
    PeepholePass,
};
use atlas_runtime::parser::Parser;
use atlas_runtime::profiler::{HotspotDetector, ProfileCollector, ProfileReport, Profiler};
use atlas_runtime::security::SecurityContext;
use atlas_runtime::span::Span;
use atlas_runtime::typechecker::TypeChecker;
use atlas_runtime::value::{FunctionRef, Value};
use atlas_runtime::vm::VM;
use rstest::rstest;

// ============================================================================
// Shared helpers (available to all submodules via `use super::*`)
// ============================================================================

fn compile(source: &str) -> Bytecode {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = Compiler::new();
    compiler.compile(&program).expect("Compilation failed")
}

fn compile_optimized(source: &str) -> Bytecode {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = Compiler::with_optimization();
    compiler.compile(&program).expect("Compilation failed")
}

fn run(bc: Bytecode) -> Option<Value> {
    let security = SecurityContext::allow_all();
    let mut vm = VM::new(bc);
    vm.run(&security).unwrap_or(None)
}

fn run_interpreter(source: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = atlas_runtime::binder::Binder::new();
    let (mut table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut table);
    let _ = typechecker.check(&program);
    let mut interpreter = Interpreter::new();
    interpreter
        .eval(&program, &SecurityContext::allow_all())
        .map_err(|e| format!("{:?}", e))
}

fn run_vm(source: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = atlas_runtime::binder::Binder::new();
    let (mut table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut table);
    let _ = typechecker.check(&program);
    let bc = Compiler::new()
        .compile(&program)
        .map_err(|e| format!("Compile: {:?}", e))?;
    let mut vm = VM::new(bc);
    vm.run(&SecurityContext::allow_all())
        .map_err(|e| format!("VM: {:?}", e))
        .map(|v| v.unwrap_or(Value::Null))
}

// ============================================================================
// Submodules
// ============================================================================

#[path = "bytecode/compiler.rs"]
mod bytecode_compiler;

#[path = "bytecode/optimizer.rs"]
mod bytecode_optimizer;

#[path = "bytecode/profiler.rs"]
mod bytecode_profiler;

#[path = "bytecode/parity.rs"]
mod bytecode_parity;

#[path = "bytecode/patterns.rs"]
mod bytecode_patterns;

#[path = "bytecode/mod_tests.rs"]
mod bytecode_mod_tests;

#[path = "bytecode/validator.rs"]
mod bytecode_validator;
