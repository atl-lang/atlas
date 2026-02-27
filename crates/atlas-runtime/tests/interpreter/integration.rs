use super::*;
use atlas_runtime::compiler::Compiler;
use atlas_runtime::vm::VM;

// Shared helper functions for integration tests

fn assert_parity(source: &str) {
    // Run interpreter (with binder + typechecker for type-tag resolution)
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);

    let mut interp = Interpreter::new();
    let interp_result = interp.eval(&program, &SecurityContext::allow_all());

    // Run VM (with binder + typechecker so compiler has type tags)
    let mut lexer2 = Lexer::new(source);
    let (tokens2, _) = lexer2.tokenize();
    let mut parser2 = Parser::new(tokens2);
    let (program2, _) = parser2.parse();
    let mut binder2 = Binder::new();
    let (mut symbol_table2, _) = binder2.bind(&program2);
    let mut typechecker2 = TypeChecker::new(&mut symbol_table2);
    let _ = typechecker2.check(&program2);

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program2).expect("compilation failed");
    let mut vm = VM::new(bytecode);
    let vm_result = vm.run(&SecurityContext::allow_all());

    // Compare results
    match (interp_result, vm_result) {
        (Ok(interp_val), Ok(vm_val)) => {
            let interp_str = format!("{:?}", interp_val);
            let vm_str = format!("{:?}", vm_val.unwrap_or(Value::Null));
            std::assert_eq!(
                interp_str, vm_str,
                "Parity mismatch for:\n{}\nInterpreter: {}\nVM: {}",
                source, interp_str, vm_str
            );
        }
        (Err(interp_err), Err(vm_err)) => {
            // Both errored - acceptable parity
            let _ = (interp_err, vm_err);
        }
        (Ok(val), Err(err)) => {
            panic!(
                "Parity mismatch: interpreter succeeded with {:?}, VM failed with {:?}",
                val, err
            );
        }
        (Err(err), Ok(val)) => {
            panic!(
                "Parity mismatch: interpreter failed with {:?}, VM succeeded with {:?}",
                err, val
            );
        }
    }
}

fn assert_ownership_parity(source: &str) {
    // Interpreter
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);
    let mut interp = Interpreter::new();
    let interp_result = interp.eval(&program, &SecurityContext::allow_all());

    // VM
    let mut lexer2 = Lexer::new(source);
    let (tokens2, _) = lexer2.tokenize();
    let mut parser2 = Parser::new(tokens2);
    let (program2, _) = parser2.parse();
    let mut binder2 = Binder::new();
    let (mut symbol_table2, _) = binder2.bind(&program2);
    let mut typechecker2 = TypeChecker::new(&mut symbol_table2);
    let _ = typechecker2.check(&program2);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program2).expect("VM compilation failed");
    let mut vm = VM::new(bytecode);
    let vm_result = vm.run(&SecurityContext::allow_all());

    match (interp_result, vm_result) {
        (Ok(_), Ok(_)) => {}   // Both succeeded — parity holds
        (Err(_), Err(_)) => {} // Both errored — parity holds (message checked per-test)
        (Ok(v), Err(e)) => panic!(
            "Parity mismatch: interpreter OK ({:?}), VM err ({:?})\n{}",
            v, e, source
        ),
        (Err(e), Ok(v)) => panic!(
            "Parity mismatch: interpreter err ({:?}), VM ok ({:?})\n{}",
            e, v, source
        ),
    }
}

/// Assert both engines fail with an error containing `expected_fragment`.
fn assert_ownership_parity_err(source: &str, expected_fragment: &str) {
    let interp_result = run_interpreter(source);
    let vm_result = run_vm(source);

    assert!(
        interp_result.is_err(),
        "Interpreter should error for:\n{}\ngot: {:?}",
        source,
        interp_result
    );
    assert!(
        vm_result.is_err(),
        "VM should error for:\n{}\ngot: {:?}",
        source,
        vm_result
    );
    let ie = interp_result.unwrap_err();
    let ve = vm_result.unwrap_err();
    assert!(
        ie.contains(expected_fragment),
        "Interpreter error missing '{}': {}",
        expected_fragment,
        ie
    );
    assert!(
        ve.contains(expected_fragment),
        "VM error missing '{}': {}",
        expected_fragment,
        ve
    );
}

/// Run source through interpreter, return Ok(debug_string) or Err(debug_string).
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

/// Run source through VM, return Ok(debug_string) or Err(debug_string).
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
    let bytecode = compiler.compile(&program).expect("VM compilation failed");
    let mut vm = VM::new(bytecode);
    match vm.run(&SecurityContext::allow_all()) {
        Ok(v) => Ok(format!("{:?}", v)),
        Err(e) => Err(format!("{:?}", e)),
    }
}

// Integration test submodules
#[path = "integration/arithmetic.rs"]
mod arithmetic;

#[path = "integration/arrays.rs"]
mod arrays;

#[path = "integration/control_flow.rs"]
mod control_flow;

#[path = "integration/functions.rs"]
mod functions;

#[path = "integration/logical.rs"]
mod logical;

#[path = "integration/strings.rs"]
mod strings;

#[path = "integration/mutations.rs"]
mod mutations;

#[path = "integration/parity_basic.rs"]
mod parity_basic;

#[path = "integration/integration_advanced.rs"]
mod integration_advanced;

#[path = "integration/edge_cases.rs"]
mod edge_cases;

#[path = "integration/arrays_advanced.rs"]
mod arrays_advanced;

#[path = "integration/value_semantics.rs"]
mod value_semantics;

#[path = "integration/ownership_fundamentals.rs"]
mod ownership_fundamentals;

#[path = "integration/ownership_parity.rs"]
mod ownership_parity;

#[path = "integration/ownership_traits.rs"]
mod ownership_traits;
