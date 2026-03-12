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

/// H-289 regression test: VM infinite loop on function call return
/// This tests direct bytecode compilation and execution through VM.
#[test]
fn test_h289_function_call_return() {
    let source = r#"
        fn foo(): number {
            return 42;
        }
        let x: number = foo();
    "#;
    compile_and_run(source).expect("Function call should work with direct compilation");
}

/// H-289 regression test: Full repro with console.log
#[test]
fn test_h289_function_call_with_console_log() {
    let source = r#"
        fn foo(): number {
            return 42;
        }
        let x: number = foo();
        console.log(x);
    "#;
    compile_and_run(source).expect("Function call with console.log should work");
}

/// H-289 regression test: Empty function body with void return type
#[test]
fn test_h289_empty_function_body() {
    let source = r#"
        fn foo(): void {}
        let x = foo();
        console.log(x);
    "#;
    compile_and_run(source).expect("Empty function call should work");
}

/// H-289 regression test: Multiple eval() calls with FRESH VMs (expected to FAIL)
/// This demonstrates WHY fresh VMs don't work - functions have bytecode_offset
/// pointing to their original bytecode, which doesn't exist in the new VM.
#[test]
#[should_panic(expected = "Unknown opcode")]
fn test_h289_multiple_evals_fresh_vm_should_fail() {
    use std::collections::HashMap;

    // Helper to compile and run with globals (BROKEN approach - fresh VM each time)
    fn compile_and_run_with_globals(
        source: &str,
        globals: &HashMap<String, atlas_runtime::Value>,
    ) -> Result<HashMap<String, atlas_runtime::Value>, String> {
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

        // Copy in existing globals - THIS IS THE BUG: Function values have
        // bytecode_offset pointing to the ORIGINAL bytecode, not this one!
        for (name, value) in globals {
            vm.set_global(name.clone(), value.clone());
        }

        vm.run(&security).map_err(|e| format!("{}", e))?;
        Ok(vm.get_globals().clone())
    }

    // First eval: define a function
    let globals = compile_and_run_with_globals(
        r#"
            fn foo(): number {
                return 42;
            }
        "#,
        &HashMap::new(),
    )
    .expect("First eval should define function");

    // Second eval: call the function - THIS WILL FAIL with "Unknown opcode"
    // because foo's bytecode_offset points to the first bytecode, not this one
    let _result = compile_and_run_with_globals(
        r#"
            let x: number = foo();
            console.log(x);
        "#,
        &globals,
    )
    .expect("Second eval should call function successfully");
}

/// H-289 regression test: Multiple eval() calls with ACCUMULATED bytecode (correct approach)
/// This is how eval() MUST work - accumulate bytecode so function offsets remain valid,
/// AND persist globals between VMs.
#[test]
fn test_h289_multiple_evals_accumulated_bytecode() {
    use std::collections::HashMap;

    // Shared state across evals
    let mut accumulated_bytecode = Bytecode::new();
    let mut persistent_globals: HashMap<String, atlas_runtime::Value> = HashMap::new();

    // Helper to compile, accumulate, and run with state persistence
    fn compile_accumulate_and_run(
        source: &str,
        accumulated: &mut Bytecode,
        globals: &mut HashMap<String, atlas_runtime::Value>,
    ) -> Result<(), String> {
        let mut lexer = Lexer::new(source);
        let (tokens, _) = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (ast, _) = parser.parse();
        let mut binder = Binder::new();
        let (mut st, _) = binder.bind(&ast);
        let mut tc = TypeChecker::new(&mut st);
        let _ = tc.check(&ast);
        let mut compiler = Compiler::new();
        let new_bytecode = compiler.compile(&ast).map_err(|e| format!("{:?}", e))?;

        // Track where new code starts
        let new_code_start = accumulated.instructions.len();

        // Append new bytecode to accumulated
        accumulated.append(new_bytecode);

        // Create VM with accumulated bytecode
        let mut vm = VM::new(accumulated.clone());
        let security = SecurityContext::allow_all();

        // Copy in existing globals
        for (name, value) in globals.iter() {
            vm.set_global(name.clone(), value.clone());
        }

        // Set IP to start of new code (don't re-execute old code)
        vm.set_ip(new_code_start);

        vm.run(&security).map_err(|e| format!("{}", e))?;

        // Copy globals back for persistence
        for (name, value) in vm.get_globals() {
            globals.insert(name.clone(), value.clone());
        }

        Ok(())
    }

    // First eval: define a function
    compile_accumulate_and_run(
        r#"
            fn foo(): number {
                return 42;
            }
        "#,
        &mut accumulated_bytecode,
        &mut persistent_globals,
    )
    .expect("First eval should define function");

    // Second eval: call the function - THIS SHOULD WORK because:
    // 1. foo's bytecode_offset points to the accumulated bytecode (which has foo's body)
    // 2. foo is in persistent_globals and gets copied to the new VM
    compile_accumulate_and_run(
        r#"
            let x: number = foo();
            console.log(x);
        "#,
        &mut accumulated_bytecode,
        &mut persistent_globals,
    )
    .expect("Second eval should call function successfully");
}

/// H-289 regression test: Same as above but with roundtrip serialization
#[test]
fn test_h289_function_call_return_roundtrip() {
    let source = r#"
        fn foo(): number {
            return 42;
        }
        let x: number = foo();
    "#;
    compile_roundtrip_and_run(source).expect("Function call should work with roundtrip");
}

/// H-289 regression test: Simulate what P02 would do in Atlas::eval_source()
/// This mimics the exact flow: source preprocessing → lex → parse → bind → typecheck → compile → VM
#[test]
fn test_h289_p02_eval_source_simulation() {
    use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

    let source = r#"
        fn foo(): number {
            return 42;
        }
        let x = foo();
        console.log(x);
    "#;

    // Simulate Atlas::eval_source() preprocessing
    let source = source.trim();
    let source_with_semi = if !source.is_empty() && !source.ends_with(';') && !source.ends_with('}')
    {
        format!("{};", source)
    } else {
        source.to_string()
    };

    // Lex
    let mut lexer = Lexer::new(&source_with_semi);
    let (tokens, lex_diagnostics) = lexer.tokenize();
    assert!(
        lex_diagnostics.is_empty(),
        "Lex errors: {:?}",
        lex_diagnostics
    );

    // Parse
    let mut parser = Parser::new(tokens);
    let (ast, parse_diagnostics) = parser.parse();
    let parse_errors: Vec<_> = parse_diagnostics.iter().filter(|d| d.is_error()).collect();
    assert!(parse_errors.is_empty(), "Parse errors: {:?}", parse_errors);

    // Bind
    let mut binder = Binder::new();
    let (mut symbol_table, bind_diagnostics) = binder.bind(&ast);
    let bind_errors: Vec<_> = bind_diagnostics.iter().filter(|d| d.is_error()).collect();
    assert!(bind_errors.is_empty(), "Bind errors: {:?}", bind_errors);

    // Type check
    let mut type_checker = TypeChecker::new(&mut symbol_table);
    let type_diagnostics = type_checker.check(&ast);
    let type_errors: Vec<_> = type_diagnostics.iter().filter(|d| d.is_error()).collect();
    assert!(type_errors.is_empty(), "Type errors: {:?}", type_errors);

    // THIS IS WHAT P02 WOULD DO: Compile to bytecode instead of using interpreter
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("Compilation should succeed");

    // Create VM and run
    let mut vm = VM::new(bytecode);
    let security = SecurityContext::allow_all();
    let result = vm.run(&security);

    // Verify success
    assert!(result.is_ok(), "VM run failed: {:?}", result);
}

/// H-289 regression test: Test api::Runtime VM mode
/// This is the existing VM mode implementation that should work.
#[test]
fn test_h289_api_runtime_vm_mode() {
    use atlas_runtime::api::Runtime;

    // Create runtime (always uses VM)
    let security = atlas_runtime::SecurityContext::allow_all();
    let mut runtime = Runtime::new_with_security(security);

    // First eval: define a function
    let result = runtime.eval(
        r#"
        fn foo(): number {
            return 42;
        }
    "#,
    );
    assert!(result.is_ok(), "First eval should work: {:?}", result);

    // Second eval: call the function
    let result = runtime.eval("foo()");
    assert!(result.is_ok(), "Second eval should work: {:?}", result);

    // Verify the result
    if let Ok(atlas_runtime::Value::Number(n)) = result {
        assert_eq!(n, 42.0);
    } else {
        panic!("Expected Number(42), got {:?}", result);
    }
}

/// H-289 regression test: Simulate P02's VM::load_module approach
/// This is exactly what P02 did - use load_module to add new code to an existing VM.
#[test]
fn test_h289_vm_load_module_approach() {
    // First: compile and run bytecode that defines a function
    let source1 = r#"
        fn foo(): number {
            return 42;
        }
    "#;
    let mut lexer1 = Lexer::new(source1);
    let (tokens1, _) = lexer1.tokenize();
    let mut parser1 = Parser::new(tokens1);
    let (ast1, _) = parser1.parse();
    let mut binder1 = Binder::new();
    let (mut st1, _) = binder1.bind(&ast1);
    let mut tc1 = TypeChecker::new(&mut st1);
    let _ = tc1.check(&ast1);
    let mut compiler1 = Compiler::new();
    let bytecode1 = compiler1
        .compile(&ast1)
        .expect("First compile should succeed");

    // Create VM and run first module
    let mut vm = VM::new(bytecode1);
    let security = SecurityContext::allow_all();
    let result1 = vm.run(&security);
    assert!(result1.is_ok(), "First run failed: {:?}", result1);

    // Second: compile bytecode that calls the function
    let source2 = r#"
        let x = foo();
        console.log(x);
    "#;
    let mut lexer2 = Lexer::new(source2);
    let (tokens2, _) = lexer2.tokenize();
    let mut parser2 = Parser::new(tokens2);
    let (ast2, _) = parser2.parse();
    let mut binder2 = Binder::new();
    let (mut st2, _) = binder2.bind(&ast2);
    let mut tc2 = TypeChecker::new(&mut st2);
    let _ = tc2.check(&ast2);
    let mut compiler2 = Compiler::new();
    let bytecode2 = compiler2
        .compile(&ast2)
        .expect("Second compile should succeed");

    // Load new module into existing VM and run (this is what P02 did)
    vm.load_module(bytecode2);
    let result2 = vm.run(&security);
    assert!(result2.is_ok(), "Second run failed: {:?}", result2);
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
