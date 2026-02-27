//! Shared test helpers for type inference tests

use atlas_runtime::binder::Binder;
use atlas_runtime::diagnostic::{Diagnostic, DiagnosticLevel};
use atlas_runtime::interpreter::Interpreter;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::typechecker::TypeChecker;
use atlas_runtime::{Compiler, Value, VM};

// ============================================================================
// Parity helpers (interpreter + VM)
// ============================================================================

/// Evaluate source in the interpreter, ignoring warnings (only fail on errors).
pub(super) fn interp_eval(source: &str) -> Value {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    let errors: Vec<_> = lex_diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    if !errors.is_empty() {
        return Value::Null;
    }
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    let errors: Vec<_> = parse_diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    if !errors.is_empty() {
        return Value::Null;
    }
    let mut binder = Binder::new();
    let (mut table, bind_diags) = binder.bind(&program);
    let errors: Vec<_> = bind_diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    if !errors.is_empty() {
        return Value::Null;
    }
    let mut checker = TypeChecker::new(&mut table);
    let type_diags = checker.check(&program);
    let errors: Vec<_> = type_diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    if !errors.is_empty() {
        return Value::Null;
    }
    let security = SecurityContext::allow_all();
    let mut interp = Interpreter::new();
    interp.eval(&program, &security).unwrap_or(Value::Null)
}

pub(super) fn vm_eval(source: &str) -> Option<Value> {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).ok()?;
    let mut vm = VM::new(bytecode);
    // run() returns Result<Option<Value>, RuntimeError> — flatten the Option
    vm.run(&SecurityContext::allow_all()).ok().flatten()
}

pub(super) fn assert_parity_num(source: &str, expected: f64) {
    let interp = interp_eval(source);
    let vm = vm_eval(source);
    assert_eq!(
        interp,
        Value::Number(expected),
        "Interpreter mismatch for:\n{}",
        source
    );
    assert_eq!(
        vm,
        Some(Value::Number(expected)),
        "VM mismatch for:\n{}",
        source
    );
}

pub(super) fn assert_parity_str(source: &str, expected: &str) {
    let interp = interp_eval(source);
    let vm = vm_eval(source);
    assert_eq!(
        interp,
        Value::String(std::sync::Arc::new(expected.to_string())),
        "Interpreter mismatch for:\n{}",
        source
    );
    assert_eq!(
        vm,
        Some(Value::String(std::sync::Arc::new(expected.to_string()))),
        "VM mismatch for:\n{}",
        source
    );
}

pub(super) fn assert_parity_bool(source: &str, expected: bool) {
    let interp = interp_eval(source);
    let vm = vm_eval(source);
    assert_eq!(
        interp,
        Value::Bool(expected),
        "Interpreter mismatch for:\n{}",
        source
    );
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
