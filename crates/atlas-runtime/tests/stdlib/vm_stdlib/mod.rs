// VM stdlib tests - split by domain for maintainability
//
// Test count: 316 tests total
// - strings: 45 tests
// - json: 69 tests
// - io_part1: ~27 tests (includes execute_with_io helper)
// - io_part2: ~27 tests
// - types_typeof_guards: 91 tests (typeof + Type Guards)
// - types_option_result: 57 tests (Option/Result/Error Propagation)

use super::*;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::{Binder, Compiler, Lexer, Parser, TypeChecker, VM};
use tempfile::TempDir;

// Helper to execute Atlas source via bytecode (used by io_part1 and io_part2)
pub fn execute_with_io(source: &str, temp_dir: &TempDir) -> Result<atlas_runtime::Value, String> {
    // Parse and compile
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&ast);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).unwrap();

    // Execute with security context
    let mut security = SecurityContext::new();
    security.grant_filesystem_read(temp_dir.path(), true);
    security.grant_filesystem_write(temp_dir.path(), true);

    let mut vm = VM::new(bytecode);
    vm.run(&security)
        .map(|opt| opt.unwrap_or(atlas_runtime::Value::Null))
        .map_err(|e| format!("{:?}", e))
}

mod io_part1;
mod io_part2;
mod json;
mod strings;
mod types_option_result;
mod types_typeof_guards;
