//! THIN ROUTER — DO NOT ADD TESTS HERE.
//! Add tests to the submodule files: tests/vm/{integration,member,complex_programs,regression,performance,functions,nested,for_in}.rs
//! This file only declares submodules and shared helpers.

mod common;

use atlas_runtime::binder::Binder;
use atlas_runtime::bytecode::Bytecode;
use atlas_runtime::compiler::Compiler;
use atlas_runtime::debugger::{DebugRequest, DebugResponse, DebuggerSession, SourceLocation};
use atlas_runtime::interpreter::Interpreter;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::optimizer::Optimizer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::typechecker::generics::Monomorphizer;
use atlas_runtime::typechecker::TypeChecker;
use atlas_runtime::types::{Type, TypeParamDef};
use atlas_runtime::value::Value;
use atlas_runtime::vm::{Profiler, VM};
use atlas_runtime::Atlas;
use common::{assert_error_code, assert_eval_null, assert_eval_number, assert_eval_string};
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::time::Instant;

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

fn vm_run(bc: Bytecode) -> Option<Value> {
    let mut vm = VM::new(bc);
    vm.run(&SecurityContext::allow_all()).expect("VM failed")
}

fn vm_eval(source: &str) -> Option<Value> {
    vm_run(compile(source))
}

fn vm_eval_opt(source: &str) -> Option<Value> {
    vm_run(compile_optimized(source))
}

fn vm_number(source: &str) -> f64 {
    match vm_eval(source) {
        Some(Value::Number(n)) => n,
        other => panic!("Expected Number, got {:?}", other),
    }
}

fn vm_number_opt(source: &str) -> f64 {
    match vm_eval_opt(source) {
        Some(Value::Number(n)) => n,
        other => panic!("Expected Number, got {:?}", other),
    }
}

fn vm_string(source: &str) -> String {
    match vm_eval(source) {
        Some(Value::String(s)) => (*s).clone(),
        other => panic!("Expected String, got {:?}", other),
    }
}

fn vm_bool(source: &str) -> bool {
    match vm_eval(source) {
        Some(Value::Bool(b)) => b,
        other => panic!("Expected Bool, got {:?}", other),
    }
}

fn interp_eval(source: &str) -> Value {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut interpreter = Interpreter::new();
    interpreter
        .eval(&program, &SecurityContext::allow_all())
        .expect("Interpreter failed")
}

fn assert_parity(source: &str) {
    let vm_result = vm_eval(source);
    let interp_result = interp_eval(source);
    let vm_val = vm_result.unwrap_or(Value::Null);
    assert_eq!(
        vm_val, interp_result,
        "Parity mismatch for:\n{}\nVM:    {:?}\nInterp: {:?}",
        source, vm_val, interp_result
    );
}

/// Assert both engines produce the same error message for invalid programs
fn assert_error_parity(source: &str) {
    // VM
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).expect("Compilation failed");
    let mut vm = VM::new(bytecode);
    let vm_err = vm
        .run(&SecurityContext::allow_all())
        .expect_err("VM should have errored");

    // Interpreter
    let mut lexer2 = Lexer::new(source.to_string());
    let (tokens2, _) = lexer2.tokenize();
    let mut parser2 = Parser::new(tokens2);
    let (program2, _) = parser2.parse();
    let mut interpreter = Interpreter::new();
    let interp_err = interpreter
        .eval(&program2, &SecurityContext::allow_all())
        .expect_err("Interpreter should have errored");

    assert_eq!(
        format!("{}", vm_err),
        format!("{}", interp_err),
        "Error parity mismatch for:\n{}\nVM:    {}\nInterp: {}",
        source,
        vm_err,
        interp_err
    );
}

// ============================================================================
// From vm_integration_tests.rs
// ============================================================================

// VM Integration Tests
//
// Tests all bytecode-VM features working together: optimizer, profiler,
// debugger, and performance optimizations. Verifies no interference
// between subsystems.

// ============================================================================
// Helpers
// ============================================================================

// ============================================================================
// 1. Optimizer + Debugger Integration (tests 1-10)
// ============================================================================

fn run_vm(source: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);
    let mut compiler = Compiler::new();
    match compiler.compile(&program) {
        Ok(bytecode) => {
            let mut vm = VM::new(bytecode);
            match vm.run(&SecurityContext::allow_all()) {
                Ok(opt_value) => match opt_value {
                    Some(value) => Ok(format!("{:?}", value)),
                    None => Ok("None".to_string()),
                },
                Err(e) => Err(format!("{:?}", e)),
            }
        }
        Err(e) => Err(format!("Compile error: {:?}", e)),
    }
}

// Domain submodules (files live in tests/vm/)
#[path = "vm/complex_programs.rs"]
mod vm_complex_programs;
#[path = "vm/for_in.rs"]
mod vm_for_in;
#[path = "vm/functions.rs"]
mod vm_functions;
#[path = "vm/integration.rs"]
mod vm_integration;
#[path = "vm/member.rs"]
mod vm_member;
#[path = "vm/nested.rs"]
mod vm_nested;
#[path = "vm/performance.rs"]
mod vm_performance;
#[path = "vm/regression.rs"]
mod vm_regression;
