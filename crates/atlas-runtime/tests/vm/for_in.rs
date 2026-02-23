use super::*;
use pretty_assertions::assert_eq;

// From test_for_in_execution.rs
// ============================================================================

// For-in loop execution tests (Phase-20c)
//
// Tests that for-in loops execute correctly in the interpreter.

#[test]
fn test_for_in_basic_execution() {
    let source = r#"
        let arr: array = [1, 2, 3];
        var sum: number = 0;
        for item in arr {
            sum = sum + item;
        }
        sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert!(result.is_ok(), "Should execute for-in loop: {:?}", result);
    assert_eq!(result.unwrap(), Value::Number(6.0), "Sum should be 6");
}

#[test]
fn test_for_in_empty_array() {
    let source = r#"
        let arr: array = [];
        var count: number = 0;
        for item in arr {
            count = count + 1;
        }
        count
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert!(result.is_ok(), "Should handle empty array: {:?}", result);
    assert_eq!(result.unwrap(), Value::Number(0.0), "Count should be 0");
}

#[test]
fn test_for_in_with_strings() {
    let source = r#"
        let words: array = ["hello", "world"];
        var result: string = "";
        for word in words {
            result = result + word + " ";
        }
        result
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert!(result.is_ok(), "Should work with strings: {:?}", result);
    match result.unwrap() {
        Value::String(s) => assert_eq!(&*s, "hello world "),
        other => panic!("Expected string, got {:?}", other),
    }
}

#[test]
fn test_for_in_nested() {
    let source = r#"
        let matrix: array = [[1, 2], [3, 4]];
        var sum: number = 0;
        for row in matrix {
            for item in row {
                sum = sum + item;
            }
        }
        sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert!(result.is_ok(), "Should handle nested loops: {:?}", result);
    assert_eq!(result.unwrap(), Value::Number(10.0), "Sum should be 10");
}

#[test]
fn test_for_in_modifies_external_variable() {
    let source = r#"
        let arr: array = [10, 20, 30];
        var total: number = 0;
        for x in arr {
            total = total + x;
        }
        total
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(result.unwrap(), Value::Number(60.0));
}

#[test]
fn test_for_in_with_break() {
    let source = r#"
        let arr: array = [1, 2, 3, 4, 5];
        var sum: number = 0;
        for item in arr {
            if (item > 3) {
                break;
            }
            sum = sum + item;
        }
        sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(
        result.unwrap(),
        Value::Number(6.0),
        "Should break at 4, sum 1+2+3=6"
    );
}

#[test]
fn test_for_in_with_continue() {
    let source = r#"
        let arr: array = [1, 2, 3, 4, 5];
        var sum: number = 0;
        for item in arr {
            if (item == 3) {
                continue;
            }
            sum = sum + item;
        }
        sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(
        result.unwrap(),
        Value::Number(12.0),
        "Should skip 3, sum 1+2+4+5=12"
    );
}

#[test]
fn test_for_in_variable_shadowing() {
    let source = r#"
        let item: number = 100;
        let arr: array = [1, 2, 3];

        for item in arr {
            // 'item' here shadows outer 'item'
        }

        item
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(
        result.unwrap(),
        Value::Number(100.0),
        "Outer variable unchanged"
    );
}

#[test]
fn test_for_in_in_function() {
    let source = r#"
        fn sum_array(arr: array) -> number {
            var total: number = 0;
            for item in arr {
                total = total + item;
            }
            return total;
        }

        sum_array([10, 20, 30])
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(result.unwrap(), Value::Number(60.0));
}

// ============================================================================
// Correctness-04: Callback intrinsic parity tests
// ============================================================================

// --- Invalid callback argument: error message parity ---

#[test]
fn test_parity_map_invalid_callback() {
    // Debug: check what each engine returns
    assert_error_parity(r#"map([1,2,3], "not a function");"#);
}

#[test]
fn test_parity_filter_invalid_callback() {
    assert_error_parity(r#"filter([1,2,3], "not a function");"#);
}

#[test]
fn test_parity_reduce_invalid_callback() {
    assert_error_parity(r#"reduce([1,2,3], "not a function", 0);"#);
}

#[test]
fn test_parity_foreach_invalid_callback() {
    assert_error_parity(r#"forEach([1,2,3], "not a function");"#);
}

#[test]
fn test_parity_find_invalid_callback() {
    assert_error_parity(r#"find([1,2,3], "not a function");"#);
}

#[test]
fn test_parity_find_index_invalid_callback() {
    assert_error_parity(r#"findIndex([1,2,3], "not a function");"#);
}

#[test]
fn test_parity_flat_map_invalid_callback() {
    assert_error_parity(r#"flatMap([1,2,3], "not a function");"#);
}

#[test]
fn test_parity_some_invalid_callback() {
    assert_error_parity(r#"some([1,2,3], "not a function");"#);
}

#[test]
fn test_parity_every_invalid_callback() {
    assert_error_parity(r#"every([1,2,3], "not a function");"#);
}

#[test]
fn test_parity_sort_invalid_callback() {
    assert_error_parity(r#"sort([1,2,3], "not a function");"#);
}

#[test]
fn test_parity_sort_by_invalid_callback() {
    assert_error_parity(r#"sortBy([1,2,3], "not a function");"#);
}

#[test]
fn test_parity_result_map_invalid_callback() {
    assert_error_parity(r#"result_map(Ok(1), "not a function");"#);
}

#[test]
fn test_parity_result_map_err_invalid_callback() {
    assert_error_parity(r#"result_map_err(Err("e"), "not a function");"#);
}

#[test]
fn test_parity_result_and_then_invalid_callback() {
    assert_error_parity(r#"result_and_then(Ok(1), "not a function");"#);
}

#[test]
fn test_parity_result_or_else_invalid_callback() {
    assert_error_parity(r#"result_or_else(Err("e"), "not a function");"#);
}

// ============================================================================
// for-in VM parity tests (fix/pre-v03-blockers)
// ============================================================================

#[test]
fn test_forin_vm_sum_array() {
    assert_parity(
        r#"
var sum = 0;
let arr = [1, 2, 3, 4, 5];
for x in arr {
    sum = sum + x;
}
sum;
"#,
    );
}

#[test]
fn test_forin_vm_empty_array() {
    assert_parity(
        r#"
var count = 0;
let arr: number[] = [];
for x in arr {
    count = count + 1;
}
count;
"#,
    );
}

#[test]
fn test_forin_vm_single_element() {
    assert_parity(
        r#"
var result = 0;
let arr = [42];
for x in arr {
    result = x;
}
result;
"#,
    );
}

#[test]
fn test_forin_vm_string_array() {
    assert_parity(
        r#"
var count = 0;
let words = ["hello", "world", "atlas"];
for w in words {
    count = count + 1;
}
count;
"#,
    );
}

#[test]
fn test_forin_vm_nested_loop() {
    assert_parity(
        r#"
var total = 0;
let outer = [1, 2, 3];
let inner = [10, 20];
for a in outer {
    for b in inner {
        total = total + a + b;
    }
}
total;
"#,
    );
}

#[test]
fn test_forin_vm_break() {
    assert_parity(
        r#"
var found = 0;
let arr = [1, 2, 3, 4, 5];
for x in arr {
    if (x == 3) {
        found = x;
        break;
    }
}
found;
"#,
    );
}

#[test]
fn test_forin_vm_last_value() {
    assert_parity(
        r#"
var last = 0;
let arr = [10, 20, 30];
for x in arr {
    last = x;
}
last;
"#,
    );
}

// ============================================================================
// Phase 16: Array method CoW write-back — VM parity tests
// Tests use run_vm() which runs the full pipeline (incl. typechecker).
// ============================================================================

#[test]
fn test_vm_array_push_cow_writeback() {
    let result = run_vm(r#"var arr: array = [1, 2, 3]; arr.push(4); arr[3];"#);
    assert_eq!(result, Ok("Number(4)".to_string()));
}

#[test]
fn test_vm_array_push_len_increases() {
    let result = run_vm(r#"var arr: array = [1, 2]; arr.push(9); len(arr);"#);
    assert_eq!(result, Ok("Number(3)".to_string()));
}

#[test]
fn test_vm_array_pop_returns_element() {
    let result = run_vm(r#"var arr: array = [1, 2, 3]; let x = arr.pop(); x;"#);
    assert_eq!(result, Ok("Number(3)".to_string()));
}

#[test]
fn test_vm_array_pop_shrinks_receiver() {
    let result = run_vm(r#"var arr: array = [1, 2, 3]; arr.pop(); len(arr);"#);
    assert_eq!(result, Ok("Number(2)".to_string()));
}

#[test]
fn test_vm_array_sort_non_mutating() {
    // sort() should not update the receiver
    let result = run_vm(r#"var arr: array = [3, 1, 2]; let s = arr.sort(); arr[0];"#);
    assert_eq!(result, Ok("Number(3)".to_string()));
}

#[test]
fn test_vm_array_sort_result_sorted() {
    let result = run_vm(r#"var arr: array = [3, 1, 2]; let s = arr.sort(); s[0];"#);
    assert_eq!(result, Ok("Number(1)".to_string()));
}

#[test]
fn test_vm_array_push_parity() {
    let code = r#"var arr: array = [10, 20]; arr.push(30); arr[2];"#;
    let vm_result = run_vm(code);
    assert_eq!(vm_result, Ok("Number(30)".to_string()));
}

#[test]
fn test_vm_free_fn_pop_cow_writeback() {
    let result = run_vm(r#"var arr: array = [1, 2, 3]; let x = pop(arr); x;"#);
    assert_eq!(result, Ok("Number(3)".to_string()));
}

#[test]
fn test_vm_free_fn_pop_receiver_updated() {
    let result = run_vm(r#"var arr: array = [1, 2, 3]; pop(arr); len(arr);"#);
    assert_eq!(result, Ok("Number(2)".to_string()));
}

#[test]
fn test_vm_free_fn_shift_cow_writeback() {
    let result = run_vm(r#"var arr: array = [10, 20, 30]; let x = shift(arr); x;"#);
    assert_eq!(result, Ok("Number(10)".to_string()));
}

#[test]
fn test_vm_free_fn_reverse_cow_writeback() {
    let result = run_vm(r#"var arr: array = [1, 2, 3]; reverse(arr); arr[0];"#);
    assert_eq!(result, Ok("Number(3)".to_string()));
}

// ============================================================================
// Value semantics regression tests — CoW behavior must never regress (VM)
// ============================================================================

/// Regression: assignment creates independent copy; mutation of source does not
/// affect the copy (CoW value semantics).
#[test]
fn test_vm_value_semantics_regression_assign_copy() {
    let result = run_vm(r#"let a: number[] = [1, 2, 3]; let b: number[] = a; a[0] = 99; b[0];"#);
    assert_eq!(result, Ok("Number(1)".to_string()));
}

/// Regression: mutation of assigned copy does not affect source.
#[test]
fn test_vm_value_semantics_regression_copy_mutation_isolated() {
    let result = run_vm(r#"let a: number[] = [1, 2, 3]; let b: number[] = a; b[0] = 42; a[0];"#);
    assert_eq!(result, Ok("Number(1)".to_string()));
}

/// Regression: push on assigned copy does not grow the source.
#[test]
fn test_vm_value_semantics_regression_push_copy_isolated() {
    let result = run_vm(r#"var a: array = [1, 2, 3]; var b: array = a; b.push(4); len(a);"#);
    assert_eq!(result, Ok("Number(3)".to_string()));
}

/// Regression: function parameter is an independent copy — mutations stay local.
#[test]
fn test_vm_value_semantics_regression_fn_param_copy() {
    let result = run_vm(
        r#"fn fill(arr: number[]) -> void { arr[0] = 999; } let nums: number[] = [1, 2, 3]; fill(nums); nums[0];"#,
    );
    assert_eq!(result, Ok("Number(1)".to_string()));
}

/// Regression: three-way copy — each variable is independent.
#[test]
fn test_vm_value_semantics_regression_three_way_copy() {
    let result = run_vm(
        r#"let a: number[] = [1, 2, 3]; let b: number[] = a; let c: number[] = b; b[0] = 10; c[1] = 20; a[0] + a[1];"#,
    );
    assert_eq!(result, Ok("Number(3)".to_string()));
}

// ============================================================================
// Phase 10: Compiler ownership metadata in FunctionRef
// ============================================================================

/// Helper: compile source and extract the FunctionRef for the named function from constants.
fn find_function_ref(
    bytecode: &atlas_runtime::bytecode::Bytecode,
    name: &str,
) -> atlas_runtime::value::FunctionRef {
    for constant in &bytecode.constants {
        if let atlas_runtime::value::Value::Function(f) = constant {
            if f.name == name {
                return f.clone();
            }
        }
    }
    panic!("Function '{}' not found in bytecode constants", name);
}

/// Compiling a function with an `own` param produces FunctionRef with param_ownership[0] = Some(Own).
#[test]
fn test_compiler_emits_own_annotation() {
    use atlas_runtime::ast::OwnershipAnnotation;
    let bc = compile("fn process(own data: number[]) -> void { }");
    let func = find_function_ref(&bc, "process");
    assert_eq!(func.param_ownership.len(), 1);
    assert_eq!(func.param_ownership[0], Some(OwnershipAnnotation::Own));
    assert_eq!(func.return_ownership, None);
}

/// Compiling a function with mixed annotations produces correct per-param ownership.
#[test]
fn test_compiler_emits_mixed_annotations() {
    use atlas_runtime::ast::OwnershipAnnotation;
    let bc = compile("fn f(own a: number, borrow b: string, c: bool) -> void { }");
    let func = find_function_ref(&bc, "f");
    assert_eq!(func.param_ownership.len(), 3);
    assert_eq!(func.param_ownership[0], Some(OwnershipAnnotation::Own));
    assert_eq!(func.param_ownership[1], Some(OwnershipAnnotation::Borrow));
    assert_eq!(func.param_ownership[2], None);
    assert_eq!(func.return_ownership, None);
}

/// Compiling an unannotated function produces param_ownership: [None].
#[test]
fn test_compiler_unannotated_function() {
    let bc = compile("fn f(x: number) -> number { return x; }");
    let func = find_function_ref(&bc, "f");
    assert_eq!(func.param_ownership.len(), 1);
    assert_eq!(func.param_ownership[0], None);
    assert_eq!(func.return_ownership, None);
}

/// Bytecode serialize → deserialize round-trips ownership annotations correctly.
#[test]
fn test_bytecode_round_trips_ownership() {
    use atlas_runtime::ast::OwnershipAnnotation;
    let bc = compile("fn consume(own data: number[], borrow key: string) -> void { }");
    let func_before = find_function_ref(&bc, "consume");

    // Serialize and deserialize
    let bytes = bc.to_bytes();
    let bc2 =
        atlas_runtime::bytecode::Bytecode::from_bytes(&bytes).expect("Deserialization failed");
    let func_after = find_function_ref(&bc2, "consume");

    assert_eq!(func_after.param_ownership.len(), 2);
    assert_eq!(
        func_after.param_ownership[0],
        Some(OwnershipAnnotation::Own)
    );
    assert_eq!(
        func_after.param_ownership[1],
        Some(OwnershipAnnotation::Borrow)
    );
    assert_eq!(func_after.return_ownership, None);
    assert_eq!(func_after.name, func_before.name);
    assert_eq!(func_after.arity, func_before.arity);
    assert_eq!(func_after.bytecode_offset, func_before.bytecode_offset);
}

// ============================================================================
// Phase 11: Runtime `own` enforcement in VM (debug mode)
// ============================================================================

/// Run source through the VM; return Ok(display) or Err(error message).
fn vm_run_source(source: &str) -> Result<String, String> {
    use atlas_runtime::binder::Binder;
    use atlas_runtime::typechecker::TypeChecker;
    let mut lexer = atlas_runtime::lexer::Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::parser::Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);
    let bc = compile(source);
    let mut vm = VM::new(bc);
    match vm.run(&SecurityContext::allow_all()) {
        Ok(value) => Ok(format!("{:?}", value)),
        Err(e) => Err(format!("{:?}", e)),
    }
}

/// Passing a local variable to an `own` param consumes it — subsequent read is a runtime error.
#[test]
#[cfg(debug_assertions)]
fn test_vm_own_consumes_local() {
    let src = r#"
        fn consume(own data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        consume(arr);
        arr;
    "#;
    let result = vm_run_source(src);
    assert!(
        result.is_err(),
        "Expected error after consuming arr via VM, got: {:?}",
        result
    );
    let msg = result.unwrap_err();
    assert!(
        msg.contains("use of moved value"),
        "Error should mention 'use of moved value', got: {}",
        msg
    );
}

/// A `borrow` parameter must NOT consume the caller's local in the VM.
#[test]
#[cfg(debug_assertions)]
fn test_vm_borrow_does_not_consume_local() {
    let src = r#"
        fn read(borrow data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        read(arr);
        len(arr);
    "#;
    let result = vm_run_source(src);
    assert!(
        result.is_ok(),
        "borrow should not consume binding in VM, got: {:?}",
        result
    );
    assert_eq!(result.unwrap(), "Some(Number(3))");
}

/// Passing a literal to an `own` param must not error (no binding to consume).
#[test]
#[cfg(debug_assertions)]
fn test_vm_own_literal_arg_no_consume() {
    let src = r#"
        fn consume(own data: array<number>) -> void { }
        consume([1, 2, 3]);
        42;
    "#;
    let result = vm_run_source(src);
    assert!(
        result.is_ok(),
        "literal arg to own param should not error in VM, got: {:?}",
        result
    );
    assert_eq!(result.unwrap(), "Some(Number(42))");
}

/// VM and interpreter produce the same error for the same own-violation source.
#[test]
#[cfg(debug_assertions)]
fn test_vm_own_borrow_identical_to_interpreter() {
    use atlas_runtime::binder::Binder;
    use atlas_runtime::interpreter::Interpreter;
    use atlas_runtime::typechecker::TypeChecker;

    let src = r#"
        fn consume(own data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        consume(arr);
        arr;
    "#;

    // Interpreter result
    let mut lexer = atlas_runtime::lexer::Lexer::new(src.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::parser::Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);
    let mut interp = Interpreter::new();
    let interp_result = interp.eval(&program, &SecurityContext::allow_all());

    // VM result
    let vm_result = vm_run_source(src);

    // Both must fail with "use of moved value"
    assert!(interp_result.is_err(), "Interpreter should error");
    assert!(vm_result.is_err(), "VM should error");
    assert!(
        format!("{:?}", interp_result.unwrap_err()).contains("use of moved value"),
        "Interpreter error should mention 'use of moved value'"
    );
    assert!(
        vm_result.unwrap_err().contains("use of moved value"),
        "VM error should mention 'use of moved value'"
    );
}

// ─── Phase 12: shared enforcement in VM ──────────────────────────────────────

/// Passing a plain (non-shared) value to a `shared` param must error in the VM (debug mode).
#[test]
#[cfg(debug_assertions)]
fn test_vm_shared_param_rejects_plain_value() {
    let src = r#"
        fn register(shared handler: number[]) -> void { }
        let arr: number[] = [1, 2, 3];
        register(arr);
    "#;
    let result = vm_run_source(src);
    assert!(
        result.is_err(),
        "Expected ownership violation error in VM, got: {:?}",
        result
    );
    assert!(
        result.unwrap_err().contains("ownership violation"),
        "VM error should mention 'ownership violation'"
    );
}

/// Passing an actual SharedValue to a `shared` param must succeed in the VM.
#[test]
#[cfg(debug_assertions)]
fn test_vm_shared_param_accepts_shared_value() {
    use atlas_runtime::value::{Shared, Value};

    let src = r#"
        fn register(shared handler: number[]) -> void { }
        register(sv);
    "#;
    let mut lexer = atlas_runtime::lexer::Lexer::new(src.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::parser::Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = atlas_runtime::binder::Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = atlas_runtime::typechecker::TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);

    let bc = compile(src);
    let mut vm = VM::new(bc);
    let shared_val = Value::SharedValue(Shared::new(Box::new(Value::array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
    ]))));
    vm.set_global("sv".to_string(), shared_val);

    let result = vm.run(&SecurityContext::allow_all());
    assert!(
        result.is_ok(),
        "SharedValue passed to shared param should succeed in VM, got: {:?}",
        result
    );
}

/// VM and interpreter produce the same shared-ownership error for identical source.
#[test]
#[cfg(debug_assertions)]
fn test_vm_shared_identical_to_interpreter() {
    use atlas_runtime::interpreter::Interpreter;

    let src = r#"
        fn register(shared handler: number[]) -> void { }
        let arr: number[] = [1, 2, 3];
        register(arr);
    "#;

    // Interpreter result
    let mut lexer = atlas_runtime::lexer::Lexer::new(src.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::parser::Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = atlas_runtime::binder::Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = atlas_runtime::typechecker::TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);
    let mut interp = Interpreter::new();
    let interp_result = interp.eval(&program, &SecurityContext::allow_all());

    // VM result
    let vm_result = vm_run_source(src);

    // Both must fail with "ownership violation"
    assert!(
        interp_result.is_err(),
        "Interpreter should error on shared violation"
    );
    assert!(vm_result.is_err(), "VM should error on shared violation");
    assert!(
        format!("{:?}", interp_result.unwrap_err()).contains("ownership violation"),
        "Interpreter error should mention 'ownership violation'"
    );
    assert!(
        vm_result.unwrap_err().contains("ownership violation"),
        "VM error should mention 'ownership violation'"
    );
}

// ============================================================
// Phase 12 — Compiler: Static Trait Method Dispatch
// ============================================================

#[test]
fn test_vm_trait_method_static_dispatch_number() {
    let result = run_vm(
        "
        trait Describe { fn describe(self: Describe) -> string; }
        impl Describe for number {
            fn describe(self: number) -> string { return str(self); }
        }
        let x: number = 42;
        let s: string = x.describe();
        s
    ",
    );
    assert_eq!(result.unwrap(), r#"String("42")"#);
}

#[test]
fn test_vm_trait_method_static_dispatch_string() {
    let result = run_vm(
        r#"
        trait Wrap { fn wrap(self: Wrap) -> string; }
        impl Wrap for string {
            fn wrap(self: string) -> string { return "[" + self + "]"; }
        }
        let s: string = "hello";
        let r: string = s.wrap();
        r
    "#,
    );
    assert_eq!(result.unwrap(), r#"String("[hello]")"#);
}

#[test]
fn test_vm_multiple_impl_methods_callable() {
    let result = run_vm(
        "
        trait Math {
            fn double(self: Math) -> number;
            fn triple(self: Math) -> number;
        }
        impl Math for number {
            fn double(self: number) -> number { return self * 2; }
            fn triple(self: number) -> number { return self * 3; }
        }
        let n: number = 5;
        let d: number = n.double();
        let t: number = n.triple();
        let sum: number = d + t;
        sum
    ",
    );
    assert_eq!(result.unwrap(), "Number(25)");
}

#[test]
fn test_vm_impl_for_different_types_no_collision() {
    // Both number and string implement Label — each should dispatch to its own impl
    let d_result = run_vm(
        "
        trait Label { fn label(self: Label) -> string; }
        impl Label for number {
            fn label(self: number) -> string { return \"num:\" + str(self); }
        }
        impl Label for string {
            fn label(self: string) -> string { return \"str:\" + self; }
        }
        let n: number = 7;
        let nr: string = n.label();
        nr
    ",
    );
    assert_eq!(d_result.unwrap(), r#"String("num:7")"#);

    let s_result = run_vm(
        r#"
        trait Label { fn label(self: Label) -> string; }
        impl Label for number {
            fn label(self: number) -> string { return "num:" + str(self); }
        }
        impl Label for string {
            fn label(self: string) -> string { return "str:" + self; }
        }
        let s: string = "world";
        let sr: string = s.label();
        sr
    "#,
    );
    assert_eq!(s_result.unwrap(), r#"String("str:world")"#);
}

#[test]
fn test_vm_impl_method_return_bool() {
    let result = run_vm(
        "
        trait Check { fn is_positive(self: Check) -> bool; }
        impl Check for number {
            fn is_positive(self: number) -> bool { return self > 0; }
        }
        let n: number = 5;
        let r: bool = n.is_positive();
        r
    ",
    );
    assert_eq!(result.unwrap(), "Bool(true)");
}

#[test]
fn test_vm_trait_compiles_without_bytecode() {
    // Trait declarations alone should compile cleanly with no runtime effect
    // The program just declares a trait and evaluates a number — no error expected
    let result = run_vm(
        "
        trait Marker { fn mark(self: Marker) -> void; }
        let x: number = 42;
        x
    ",
    );
    // x is the last local on stack — Number(42) in Atlas display format
    assert_eq!(result.unwrap(), "Number(42)");
}

// ============================================================
// Phase 13 — Interpreter: Trait Method Dispatch (Parity Tests)
// ============================================================

// Interpreter trait dispatch tests use Atlas::eval() which:
// 1. Runs the full pipeline (typecheck + interpret) so trait_dispatch annotations are set
// 2. Auto-adds semicolons for REPL-style last expressions

#[test]
fn test_interp_trait_method_dispatch_number() {
    let atlas = Atlas::new();
    let result = atlas.eval(
        "
        trait Describe { fn describe(self: Describe) -> string; }
        impl Describe for number {
            fn describe(self: number) -> string { return str(self); }
        }
        let x: number = 42;
        let s: string = x.describe();
        s
    ",
    );
    assert_eq!(result.unwrap(), Value::string("42"));
}

#[test]
fn test_interp_trait_method_dispatch_bool_return() {
    let atlas = Atlas::new();
    let result = atlas.eval(
        "
        trait Check { fn is_positive(self: Check) -> bool; }
        impl Check for number {
            fn is_positive(self: number) -> bool { return self > 0; }
        }
        let n: number = 5;
        let r: bool = n.is_positive();
        r
    ",
    );
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
fn test_interp_trait_method_multi_methods() {
    let atlas = Atlas::new();
    let result = atlas.eval(
        "
        trait Math {
            fn double(self: Math) -> number;
            fn triple(self: Math) -> number;
        }
        impl Math for number {
            fn double(self: number) -> number { return self * 2; }
            fn triple(self: number) -> number { return self * 3; }
        }
        let n: number = 5;
        let d: number = n.double();
        let t: number = n.triple();
        let sum: number = d + t;
        sum
    ",
    );
    assert_eq!(result.unwrap(), Value::Number(25.0));
}

#[test]
fn test_interp_vm_trait_dispatch_parity() {
    // VM path (via run_vm which uses the compiler pipeline)
    let source = "
        trait Label { fn label(self: Label) -> string; }
        impl Label for number {
            fn label(self: number) -> string { return \"n:\" + str(self); }
        }
        let x: number = 7;
        let s: string = x.label();
        s
    ";

    let vm_result = run_vm(source).expect("VM should succeed");
    assert_eq!(vm_result, r#"String("n:7")"#);

    // Interpreter path (via Atlas::eval which uses the interpreter pipeline)
    let atlas = Atlas::new();
    let interp_result = atlas.eval(source).expect("Interpreter should succeed");
    assert_eq!(interp_result, Value::string("n:7"));
}

// ─── Block-03 Phase 17: Parity Hardening — 10 Extended VM Scenarios ──────────

#[test]
fn test_parity_block03_scenario_a_vm() {
    // Multiple traits on same type
    let result = run_vm(
        "
        trait Addable { fn add(self: Addable, n: number) -> number; }
        trait Subtractable { fn sub(self: Subtractable, n: number) -> number; }
        impl Addable for number { fn add(self: number, n: number) -> number { return self + n; } }
        impl Subtractable for number { fn sub(self: number, n: number) -> number { return self - n; } }
        let x: number = 10;
        let a: number = x.add(5);
        let b: number = a.sub(3);
        b
        ",
    );
    assert_eq!(result.unwrap(), "Number(12)");
}

#[test]
fn test_parity_block03_scenario_b_vm() {
    // Trait method returning bool, used in condition
    let result = run_vm(
        r#"
        trait Comparable { fn greater_than(self: Comparable, other: number) -> bool; }
        impl Comparable for number {
            fn greater_than(self: number, other: number) -> bool { return self > other; }
        }
        let x: number = 10;
        var r: string = "no";
        if (x.greater_than(5)) { r = "yes"; }
        r
        "#,
    );
    assert_eq!(result.unwrap(), r#"String("yes")"#);
}

#[test]
fn test_parity_block03_scenario_c_vm() {
    // Trait method calling stdlib function
    let result = run_vm(
        r#"
        trait Formatted { fn fmt(self: Formatted) -> string; }
        impl Formatted for number {
            fn fmt(self: number) -> string { return "Value: " + str(self); }
        }
        let x: number = 42;
        let r: string = x.fmt();
        r
        "#,
    );
    assert_eq!(result.unwrap(), r#"String("Value: 42")"#);
}

#[test]
fn test_parity_block03_scenario_d_vm() {
    // Chained trait method calls via intermediate variables
    let result = run_vm(
        "
        trait Inc { fn inc(self: Inc) -> number; }
        impl Inc for number { fn inc(self: number) -> number { return self + 1; } }
        let x: number = 40;
        let y: number = x.inc();
        let z: number = y.inc();
        z
        ",
    );
    assert_eq!(result.unwrap(), "Number(42)");
}

#[test]
fn test_parity_block03_scenario_e_vm() {
    // Trait method with multiple parameters
    let result = run_vm(
        "
        trait Interpolator { fn interpolate(self: Interpolator, t: number, other: number) -> number; }
        impl Interpolator for number {
            fn interpolate(self: number, t: number, other: number) -> number {
                return self + (other - self) * t;
            }
        }
        let a: number = 0;
        let r: number = a.interpolate(0.5, 100);
        r
        ",
    );
    assert_eq!(result.unwrap(), "Number(50)");
}

#[test]
fn test_parity_block03_scenario_f_vm() {
    // Trait method with conditional return paths (clamp)
    let result = run_vm(
        "
        trait Clamp { fn clamp(self: Clamp, min: number, max: number) -> number; }
        impl Clamp for number {
            fn clamp(self: number, min: number, max: number) -> number {
                if (self < min) { return min; }
                if (self > max) { return max; }
                return self;
            }
        }
        let x: number = 150;
        let r: number = x.clamp(0, 100);
        r
        ",
    );
    assert_eq!(result.unwrap(), "Number(100)");
}

#[test]
fn test_parity_block03_scenario_g_vm() {
    // Impl method with local state (no leakage to caller)
    let result = run_vm(
        "
        trait Counter { fn count_to(self: Counter, n: number) -> number; }
        impl Counter for number {
            fn count_to(self: number, n: number) -> number {
                var total: number = 0;
                var i: number = self;
                while (i <= n) { total = total + i; i = i + 1; }
                return total;
            }
        }
        let x: number = 1;
        let r: number = x.count_to(10);
        r
        ",
    );
    assert_eq!(result.unwrap(), "Number(55)");
}

#[test]
fn test_parity_block03_scenario_h_vm() {
    // String type impl
    let result = run_vm(
        r#"
        trait Shouter { fn shout(self: Shouter) -> string; }
        impl Shouter for string {
            fn shout(self: string) -> string { return self + "!!!"; }
        }
        let s: string = "hello";
        let r: string = s.shout();
        r
        "#,
    );
    assert_eq!(result.unwrap(), r#"String("hello!!!")"#);
}

#[test]
fn test_parity_block03_scenario_i_vm() {
    // Bool type impl
    let result = run_vm(
        "
        trait Toggle { fn toggle(self: Toggle) -> bool; }
        impl Toggle for bool { fn toggle(self: bool) -> bool { return !self; } }
        let b: bool = true;
        let r: bool = b.toggle();
        r
        ",
    );
    assert_eq!(result.unwrap(), "Bool(false)");
}

#[test]
fn test_parity_block03_scenario_j_vm() {
    // Trait method returning array, index into result
    let result = run_vm(
        "
        trait Pair { fn pair(self: Pair) -> number[]; }
        impl Pair for number { fn pair(self: number) -> number[] { return [self, self * 2]; } }
        let x: number = 7;
        let p: number[] = x.pair();
        let r: number = p[1];
        r
        ",
    );
    assert_eq!(result.unwrap(), "Number(14)");
}

