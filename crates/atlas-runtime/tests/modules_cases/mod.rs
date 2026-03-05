// modules — Module system: binding, execution (interpreter + VM), and resolution tests

mod binding;
mod execution_interpreter_advanced_a;
mod execution_interpreter_advanced_b;
mod execution_interpreter_basic;
mod execution_vm;
mod parity;
mod resolver;

use atlas_runtime::{
    binder::Binder, lexer::Lexer, module_loader::ModuleRegistry, parser::Parser,
    typechecker::TypeChecker,
};
use atlas_runtime::{ModuleExecutor, SecurityContext, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Normalize path string for cross-platform comparison
/// Converts all separators to forward slashes for consistent testing
pub(crate) fn normalize_path_for_test(path: &str) -> String {
    path.replace('\\', "/")
}

/// Helper to parse and bind a module
pub(crate) fn bind_module(
    source: &str,
) -> (
    atlas_runtime::symbol::SymbolTable,
    Vec<atlas_runtime::diagnostic::Diagnostic>,
) {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    binder.bind(&program)
}

/// Helper to parse, bind with modules, and return symbol table + diagnostics
pub(crate) fn bind_module_with_registry(
    source: &str,
    module_path: &str,
    registry: &ModuleRegistry,
) -> (
    atlas_runtime::symbol::SymbolTable,
    Vec<atlas_runtime::diagnostic::Diagnostic>,
) {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    binder.bind_with_modules(&program, &PathBuf::from(module_path), registry)
}

/// Helper to type check with modules
#[allow(dead_code)] // Preserved for future test expansion
pub(crate) fn typecheck_module_with_registry(
    source: &str,
    module_path: &str,
    registry: &ModuleRegistry,
) -> Vec<atlas_runtime::diagnostic::Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    let (mut symbol_table, bind_diags) =
        binder.bind_with_modules(&program, &PathBuf::from(module_path), registry);

    if !bind_diags.is_empty() {
        return bind_diags;
    }

    let mut typechecker = TypeChecker::new(&mut symbol_table);
    typechecker.check_with_modules(&program, &PathBuf::from(module_path), registry)
}

/// Helper to create a test module file
pub(crate) fn create_module(dir: &Path, name: &str, content: &str) -> PathBuf {
    let path = dir.join(format!("{}.atl", name));
    fs::write(&path, content).unwrap();
    path
}

/// Helper to execute a module using the VM
pub(crate) fn execute_with_vm(entry_path: &Path, root: &Path) -> Result<Value, String> {
    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, root.to_path_buf());

    // Load and execute with interpreter first to get the result
    let result = executor.execute_module(entry_path);

    match result {
        Ok(v) => Ok(v),
        Err(diags) => Err(format!("{:?}", diags)),
    }
}

/// Helper to compare interpreter and VM execution results
pub(crate) fn assert_parity(entry_path: &Path) {
    use atlas_runtime::api::{ExecutionMode, Runtime};

    // Execute with interpreter
    let mut interpreter_runtime =
        Runtime::new_with_security(ExecutionMode::Interpreter, SecurityContext::allow_all());
    let interpreter_result = interpreter_runtime.eval_file(entry_path);

    // Execute with VM
    let mut vm_runtime =
        Runtime::new_with_security(ExecutionMode::VM, SecurityContext::allow_all());
    let vm_result = vm_runtime.eval_file(entry_path);

    // Both should succeed or both should fail
    match (&interpreter_result, &vm_result) {
        (Ok(interp_val), Ok(vm_val)) => {
            // Compare values
            let interp_str = format!("{:?}", interp_val);
            let vm_str = format!("{:?}", vm_val);
            assert_eq!(
                interp_str, vm_str,
                "Parity violation: Interpreter returned {:?}, VM returned {:?}",
                interp_val, vm_val
            );
        }
        (Err(_), Err(_)) => {
            // Both failed - that's still parity
        }
        (Ok(v), Err(e)) => {
            panic!(
                "Parity violation: Interpreter succeeded with {:?}, but VM failed with {:?}",
                v, e
            );
        }
        (Err(e), Ok(v)) => {
            panic!(
                "Parity violation: Interpreter failed with {:?}, but VM succeeded with {:?}",
                e, v
            );
        }
    }
}
