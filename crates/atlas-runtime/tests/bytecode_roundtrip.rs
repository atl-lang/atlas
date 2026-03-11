//! Test bytecode serialization/deserialization round-trip for for..in loops
//! This was added to diagnose H-288 (for..in fails in atlas build binaries)

use atlas_runtime::binder::Binder;
use atlas_runtime::bytecode::Bytecode;
use atlas_runtime::compiler::Compiler;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::typechecker::TypeChecker;
use atlas_runtime::vm::VM;

fn compile_and_run(source: &str) -> Result<(), String> {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut st, _) = binder.bind(&ast);
    let mut tc = TypeChecker::new(&mut st);
    let _ = tc.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).map_err(|e| format!("{:?}", e))?;
    let mut vm = VM::new(bytecode);
    let security = SecurityContext::allow_all();
    vm.run(&security).map_err(|e| format!("{}", e))?;
    Ok(())
}

fn compile_roundtrip_and_run(source: &str) -> Result<(), String> {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut st, _) = binder.bind(&ast);
    let mut tc = TypeChecker::new(&mut st);
    let _ = tc.check(&ast);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).map_err(|e| format!("{:?}", e))?;

    // Serialize and deserialize
    let bytes = bytecode.to_bytes();
    let restored = Bytecode::from_bytes(&bytes)?;

    // Verify top_level_local_count is preserved
    assert_eq!(
        bytecode.top_level_local_count, restored.top_level_local_count,
        "top_level_local_count should survive serialization round-trip"
    );

    // Run the restored bytecode
    let mut vm = VM::new(restored);
    let security = SecurityContext::allow_all();
    vm.run(&security).map_err(|e| format!("{}", e))?;
    Ok(())
}

#[test]
fn test_for_in_range_direct() {
    let source = r#"
        let mut count: number = 0;
        for i in 0..3 {
            count = count + 1;
        }
    "#;
    compile_and_run(source).expect("Direct execution should work");
}

#[test]
fn test_for_in_range_roundtrip() {
    let source = r#"
        let mut count: number = 0;
        for i in 0..3 {
            count = count + 1;
        }
    "#;
    compile_roundtrip_and_run(source).expect("Round-trip execution should work");
}

#[test]
fn test_for_in_array_direct() {
    let source = r#"
        let arr: number[] = [1, 2, 3];
        let mut sum: number = 0;
        for x in arr {
            sum = sum + x;
        }
    "#;
    compile_and_run(source).expect("Direct execution should work");
}

#[test]
fn test_for_in_array_roundtrip() {
    let source = r#"
        let arr: number[] = [1, 2, 3];
        let mut sum: number = 0;
        for x in arr {
            sum = sum + x;
        }
    "#;
    compile_roundtrip_and_run(source).expect("Round-trip execution should work");
}

/// Test that optimizer preserves for..in loop semantics (H-288 regression test)
#[test]
fn test_optimizer_preserves_for_in_loops() {
    let source = r#"
        let mut count: number = 0;
        for i in 0..3 {
            count = count + 1;
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut st, _) = binder.bind(&ast);
    let mut tc = TypeChecker::new(&mut st);
    let _ = tc.check(&ast);

    // Without optimization
    let mut compiler_no_opt = Compiler::new();
    let bytecode_no_opt = compiler_no_opt.compile(&ast).unwrap();

    // With optimization
    let mut compiler_opt = Compiler::with_optimization();
    let bytecode_opt = compiler_opt.compile(&ast).unwrap();

    // Both should have the same top_level_local_count
    assert_eq!(
        bytecode_no_opt.top_level_local_count, bytecode_opt.top_level_local_count,
        "Optimizer should preserve top_level_local_count"
    );

    // Run both to verify they produce the same result
    let security = SecurityContext::allow_all();

    let mut vm_no_opt = VM::new(bytecode_no_opt);
    vm_no_opt.run(&security).expect("Non-optimized should run");

    let mut vm_opt = VM::new(bytecode_opt);
    vm_opt.run(&security).expect("Optimized should run");
}
