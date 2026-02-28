use super::*;

#[rstest]
#[case(r#"var x: number = 5; x++; x"#, 6.0)]
#[case(r#"var x: number = 10; x--; x"#, 9.0)]
#[case(r#"var x: number = 0; x++; x++; x++; x"#, 3.0)]
#[case(r#"var x: number = 10; x--; x--; x"#, 8.0)]
fn test_increment_decrement_basics(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

#[test]
fn test_increment_array_element() {
    let code = r#"
        let arr: number[] = [5, 10, 15];
        arr[0]++;
        arr[0]
    "#;
    assert_eval_number(code, 6.0);
}

#[test]
fn test_decrement_array_element() {
    let code = r#"
        let arr: number[] = [5, 10, 15];
        arr[2]--;
        arr[2]
    "#;
    assert_eval_number(code, 14.0);
}

#[test]
fn test_increment_in_loop() {
    let code = r#"
        var sum: number = 0;
        var i: number = 0;
        while (i < 5) {
            sum += i;
            i++;
        }
        sum
    "#;
    assert_eval_number(code, 10.0);
}

#[rstest]
#[case("let x: number = 5; x++; x", "AT3003")]
#[case("let x: number = 10; x += 5; x", "AT3003")]
#[case("let x: number = 1; x = 2; x", "AT3003")] // Basic assignment to let
#[case("let x: number = 5; x--; x", "AT3003")] // Decrement
fn test_immutable_mutation_errors(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[rstest]
#[case("var x: number = 10; x += 5; x", 15.0)]
#[case("var x: number = 20; x -= 8; x", 12.0)]
#[case("var x: number = 7; x *= 3; x", 21.0)]
#[case("var x: number = 50; x /= 5; x", 10.0)]
#[case("var x: number = 17; x %= 5; x", 2.0)]
#[case("var x: number = 1; x = 2; x", 2.0)] // Basic assignment to var
#[case("var x: number = 5; x++; x", 6.0)] // Increment
#[case("var x: number = 5; x--; x", 4.0)] // Decrement
fn test_mutable_var_assignments(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

#[test]
fn test_compound_chained() {
    let code = r#"
        var x: number = 10;
        x += 5;
        x *= 2;
        x -= 10;
        x
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn test_compound_array_element() {
    let code = r#"
        let arr: number[] = [10, 20, 30];
        arr[1] += 5;
        arr[1]
    "#;
    assert_eval_number(code, 25.0);
}

#[test]
fn test_compound_divide_by_zero() {
    let code = r#"
        var x: number = 10;
        x /= 0;
        x
    "#;
    assert_error_code(code, "AT0005");
}

// ============================================================================
// Phase interpreter-02: Interpreter-VM Parity Tests
// ============================================================================

use atlas_runtime::compiler::Compiler;
use atlas_runtime::vm::VM;

/// Run code through both interpreter and VM, assert identical results
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
                interp_str,
                vm_str,
                "Parity mismatch for:\n{}\nInterpreter: {}\nVM: {}",
                source,
                interp_str,
                vm_str
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
