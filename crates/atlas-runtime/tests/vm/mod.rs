//! Common utilities for VM tests

pub mod array_intrinsics;
pub mod array_pure;
pub mod math_basic;
pub mod math_trig;
pub mod math_utils_constants;

use atlas_runtime::compiler::Compiler;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::value::Value;
use atlas_runtime::vm::VM;

/// Helper to execute Atlas source code using VM
pub fn execute_vm(source: &str) -> Result<Option<Value>, atlas_runtime::value::RuntimeError> {
    let mut lexer = Lexer::new(source);
    let (tokens, _lex_diagnostics) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _diagnostics) = parser.parse();

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    let mut vm = VM::new(bytecode);
    vm.run()
}

/// Helper to execute and unwrap result
pub fn execute_vm_ok(source: &str) -> Value {
    execute_vm(source).unwrap().unwrap()
}

/// Helper to execute and expect error
pub fn execute_vm_err(source: &str) -> atlas_runtime::value::RuntimeError {
    execute_vm(source).unwrap_err()
}
