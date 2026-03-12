//! Shared test helpers for type inference tests
//! D-052: Uses VM-only execution

use atlas_runtime::binder::Binder;
use atlas_runtime::diagnostic::{Diagnostic, DiagnosticLevel};
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::typechecker::TypeChecker;
use atlas_runtime::{Compiler, Value, VM};

// ============================================================================
// VM execution helper
// ============================================================================

pub(super) fn vm_eval(source: &str) -> Option<Value> {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut table, _) = binder.bind(&program);
    let mut checker = TypeChecker::new(&mut table);
    let _ = checker.check(&program);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).ok()?;
    let mut vm = VM::new(bytecode);
    // run() returns Result<Option<Value>, RuntimeError> — flatten the Option
    vm.run(&SecurityContext::allow_all()).ok().flatten()
}

pub(super) fn assert_parity_num(source: &str, expected: f64) {
    let vm = vm_eval(source);
    assert_eq!(
        vm,
        Some(Value::Number(expected)),
        "VM mismatch for:\n{}",
        source
    );
}

pub(super) fn assert_parity_str(source: &str, expected: &str) {
    let vm = vm_eval(source);
    assert_eq!(
        vm,
        Some(Value::String(std::sync::Arc::new(expected.to_string()))),
        "VM mismatch for:\n{}",
        source
    );
}

pub(super) fn assert_parity_bool(source: &str, expected: bool) {
    let vm = vm_eval(source);
    assert_eq!(
        vm,
        Some(Value::Bool(expected)),
        "VM mismatch for:\n{}",
        source
    );
}

pub(super) fn has_code(diags: &[Diagnostic], code: &str) -> bool {
    diags.iter().any(|d| d.code == code)
}
