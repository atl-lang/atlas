//! closures.rs — Closure semantic behavior tests
//!
//! Documents the ACTUAL behavior of function scoping in Atlas v0.2.
//!
//! # Closure Implementation
//!
//! Both engines support upvalue capture (as of the closure parity fix):
//! - The **interpreter** uses dynamic scoping (walks the live scope stack at call time)
//! - The **VM** uses upvalue capture at closure creation time (by value)
//!
//! ## What Works (both engines, parity):
//! - Top-level `let` and `var` are accessible from any named function
//! - Top-level `var` can be mutated from any named function
//! - Functions passed as arguments (not closures, just fn references)
//! - Functions stored in variables and called later
//! - Higher-order functions (take fn args, call them)
//! - Inner functions reading outer function locals (upvalue capture)
//! - Inner functions reading outer function parameters (upvalue capture)
//!
//! ## Semantic Note (capture-by-value in VM):
//! The VM captures outer locals BY VALUE at closure creation time.
//! The interpreter uses live dynamic scoping (captures by reference).
//! For `let` variables (immutable), both are identical.
//! For `var` mutations after closure creation, behavior may diverge.

mod common;

use atlas_runtime::binder::Binder;
use atlas_runtime::compiler::Compiler;
use atlas_runtime::interpreter::Interpreter;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::typechecker::TypeChecker;
use atlas_runtime::value::Value;
use atlas_runtime::vm::VM;
use pretty_assertions::assert_eq;

// ============================================================================
// Test helpers (same pattern as pattern_matching.rs)
// ============================================================================

fn interp_eval(source: &str) -> Value {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);

    let mut interpreter = Interpreter::new();
    interpreter
        .eval(&program, &SecurityContext::allow_all())
        .expect("Interpreter failed")
}

fn vm_eval(source: &str) -> Option<Value> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).expect("Compilation failed");
    let mut vm = VM::new(bytecode);
    vm.run(&SecurityContext::allow_all()).expect("VM failed")
}

/// Assert both engines produce the same numeric result.
fn assert_parity_number(source: &str, expected: f64) {
    let interp = interp_eval(source);
    let vm = vm_eval(source).unwrap_or(Value::Null);
    assert_eq!(
        interp,
        Value::Number(expected),
        "Interpreter wrong for: {}\n  got: {:?}",
        source,
        interp
    );
    assert_eq!(
        vm,
        Value::Number(expected),
        "VM wrong for: {}\n  got: {:?}",
        source,
        vm
    );
}

/// Assert both engines produce the same bool result.
fn assert_parity_bool(source: &str, expected: bool) {
    let interp = interp_eval(source);
    let vm = vm_eval(source).unwrap_or(Value::Null);
    assert_eq!(
        interp,
        Value::Bool(expected),
        "Interpreter wrong: {:?}",
        interp
    );
    assert_eq!(vm, Value::Bool(expected), "VM wrong: {:?}", vm);
}

/// Assert both engines produce the same string result.
fn assert_parity_string(source: &str, expected: &str) {
    let interp = interp_eval(source);
    let vm = vm_eval(source).unwrap_or(Value::Null);
    assert_eq!(
        interp,
        Value::string(expected.to_string()),
        "Interpreter wrong: {:?}",
        interp
    );
    assert_eq!(
        vm,
        Value::string(expected.to_string()),
        "VM wrong: {:?}",
        vm
    );
}

// ============================================================================
// Category A: Top-level let/var accessible from named functions
// Both engines work correctly — these are parity tests.
// ============================================================================

#[test]
fn test_top_level_let_accessible_from_fn() {
    // Top-level let is a global — any named fn can read it
    assert_parity_number(
        r#"
let x = 42;
fn get_x() -> number { return x; }
get_x();
"#,
        42.0,
    );
}

#[test]
fn test_top_level_let_used_in_arithmetic() {
    assert_parity_number(
        r#"
let base = 10;
fn double_base() -> number { return base * 2; }
double_base();
"#,
        20.0,
    );
}

#[test]
fn test_top_level_var_readable_from_fn() {
    assert_parity_number(
        r#"
var counter = 5;
fn read_counter() -> number { return counter; }
read_counter();
"#,
        5.0,
    );
}

#[test]
fn test_top_level_var_mutable_from_fn() {
    // Var mutation via named fn — top-level var is global, writable
    assert_parity_number(
        r#"
var counter = 0;
fn increment() { counter = counter + 1; }
increment();
increment();
increment();
counter;
"#,
        3.0,
    );
}

#[test]
fn test_top_level_var_mutation_and_read() {
    assert_parity_number(
        r#"
var total = 100;
fn subtract(n: number) { total = total - n; }
subtract(30);
subtract(20);
total;
"#,
        50.0,
    );
}

#[test]
fn test_two_fns_sharing_top_level_let() {
    // Two functions both reading the same top-level let
    assert_parity_number(
        r#"
let top = 7;
fn get_a() -> number { return top; }
fn get_b() -> number { return top * 2; }
get_a() + get_b();
"#,
        21.0,
    );
}

#[test]
fn test_two_fns_sharing_top_level_var() {
    // Two fns cooperating via a shared mutable top-level var
    assert_parity_number(
        r#"
var acc = 0;
fn add_one() { acc = acc + 1; }
fn add_ten() { acc = acc + 10; }
add_one();
add_ten();
add_one();
acc;
"#,
        12.0,
    );
}

#[test]
fn test_fn_reads_updated_top_level_var() {
    // Function sees the CURRENT value of var, not a captured snapshot
    assert_parity_number(
        r#"
var x = 1;
fn get_x() -> number { return x; }
x = 99;
get_x();
"#,
        99.0,
    );
}

#[test]
fn test_top_level_let_shadow_by_param() {
    // A function parameter shadows the outer let — parameter wins
    assert_parity_number(
        r#"
let x = 100;
fn shadow(x: number) -> number { return x; }
shadow(42);
"#,
        42.0,
    );
}

#[test]
fn test_top_level_let_no_snapshot() {
    // Atlas does NOT capture values — functions see live global state
    // This test verifies the function sees the changed value, not a captured one
    assert_parity_number(
        r#"
var val = 10;
fn get_val() -> number { return val; }
val = 20;
get_val();
"#,
        20.0,
    );
}

// ============================================================================
// Category B: Functions as values (pass/store/call) — both engines
// ============================================================================

#[test]
fn test_fn_stored_in_variable_and_called() {
    // Store a function reference in a variable, then call it
    assert_parity_number(
        r#"
fn add_one(n: number) -> number { return n + 1; }
let f = add_one;
f(41);
"#,
        42.0,
    );
}

#[test]
fn test_fn_passed_as_argument() {
    // Higher-order: pass a function as an argument
    assert_parity_number(
        r#"
fn double(n: number) -> number { return n * 2; }
fn apply(f: (number) -> number, x: number) -> number { return f(x); }
apply(double, 21);
"#,
        42.0,
    );
}

#[test]
fn test_fn_returned_from_fn() {
    // Return a function reference from a function, then call it
    assert_parity_number(
        r#"
fn square(n: number) -> number { return n * n; }
fn get_square_fn() -> (number) -> number { return square; }
let f = get_square_fn();
f(6);
"#,
        36.0,
    );
}

#[test]
fn test_higher_order_apply_twice() {
    assert_parity_number(
        r#"
fn inc(n: number) -> number { return n + 1; }
fn apply_twice(f: (number) -> number, x: number) -> number { return f(f(x)); }
apply_twice(inc, 40);
"#,
        42.0,
    );
}

#[test]
fn test_fn_identity_as_arg() {
    assert_parity_number(
        r#"
fn identity(n: number) -> number { return n; }
fn apply(f: (number) -> number, x: number) -> number { return f(x); }
apply(identity, 99);
"#,
        99.0,
    );
}

#[test]
fn test_fn_composition_manual() {
    // Manually compose two functions via higher-order
    // triple(4) = 12, add_ten(12) = 22
    assert_parity_number(
        r#"
fn add_ten(n: number) -> number { return n + 10; }
fn triple(n: number) -> number { return n * 3; }
fn apply(f: (number) -> number, x: number) -> number { return f(x); }
apply(add_ten, apply(triple, 4));
"#,
        22.0,
    );
}

// ============================================================================
// Category C: Recursive functions with top-level state
// ============================================================================

#[test]
fn test_recursive_fn_with_global_counter() {
    // Recursive calls accumulate into a top-level var
    assert_parity_number(
        r#"
var sum = 0;
fn accumulate(n: number) {
    if (n > 0) {
        sum = sum + n;
        accumulate(n - 1);
    }
}
accumulate(5);
sum;
"#,
        15.0,
    );
}

#[test]
fn test_mutually_referencing_fns_via_global() {
    // Two fns that communicate through a top-level var
    assert_parity_number(
        r#"
var state = 0;
fn step_a() { state = state + 1; }
fn step_b() { state = state * 2; }
step_a();
step_b();
step_a();
state;
"#,
        3.0,
    );
}

// ============================================================================
// Category D: String and multi-type closures — both engines
// ============================================================================

#[test]
fn test_top_level_string_let_in_fn() {
    assert_parity_string(
        r#"
let greeting = "hello";
fn get_greeting() -> string { return greeting; }
get_greeting();
"#,
        "hello",
    );
}

#[test]
fn test_top_level_bool_let_in_fn() {
    assert_parity_bool(
        r#"
let flag = true;
fn check_flag() -> bool { return flag; }
check_flag();
"#,
        true,
    );
}

#[test]
fn test_fn_uses_multiple_top_level_lets() {
    assert_parity_number(
        r#"
let a = 3;
let b = 4;
fn hypotenuse_sq() -> number { return a * a + b * b; }
hypotenuse_sq();
"#,
        25.0,
    );
}

// ============================================================================
// Category E: Nested fn inside fn — upvalue capture (both engines)
//
// The VM uses upvalue capture at closure creation time (by value).
// The interpreter uses live dynamic scoping.
// For let-bound variables, results are identical in both engines.
// ============================================================================

#[test]
fn test_nested_fn_params_only_no_capture() {
    // Inner fn uses only its own parameters — no outer reference — both engines work
    assert_parity_number(
        r#"
fn outer(x: number) -> number {
    fn inner(y: number) -> number {
        return y * 2;
    }
    return inner(x);
}
outer(21);
"#,
        42.0,
    );
}

#[test]
fn test_nested_fn_called_within_outer_uses_outer_var() {
    // Both engines: inner function reads outer function's local via upvalue capture
    assert_parity_number(
        r#"
fn outer() -> number {
    let x = 42;
    fn inner() -> number {
        return x;
    }
    return inner();
}
outer();
"#,
        42.0,
    );
}

#[test]
fn test_nested_fn_with_outer_param() {
    // Both engines: inner function reads outer function's parameter via upvalue capture
    assert_parity_number(
        r#"
fn outer(n: number) -> number {
    fn double_n() -> number {
        return n * 2;
    }
    return double_n();
}
outer(21);
"#,
        42.0,
    );
}

#[test]
fn test_three_levels_no_cross_scope_access() {
    // Three levels of nesting — innermost does NOT reference outer locals
    // Both engines work since no cross-scope access
    assert_parity_number(
        r#"
fn level1(a: number) -> number {
    fn level2(b: number) -> number {
        fn level3(c: number) -> number {
            return c + 1;
        }
        return level3(b);
    }
    return level2(a);
}
level1(41);
"#,
        42.0,
    );
}

#[test]
fn test_inner_fn_sibling_call() {
    // Two inner fns where one calls the other — sibling access via scoped globals
    // The VM compiler registers nested fns globally with scoped names for sibling access
    assert_parity_number(
        r#"
fn outer(x: number) -> number {
    fn helper(n: number) -> number {
        return n + 1;
    }
    fn compute(n: number) -> number {
        return helper(n) * 2;
    }
    return compute(x);
}
outer(20);
"#,
        42.0,
    );
}

// ============================================================================
// Category F: No-closure escape behavior
//
// When a named function is returned as a value and called AFTER the defining
// scope is gone, the outer scope's locals are inaccessible. These tests
// document that Atlas does NOT support closure capture — returned functions
// do not carry their defining scope.
//
// Test: fn that references only globals (top-level let/var) → works after return
// ============================================================================

#[test]
fn test_returned_fn_with_global_access_still_works() {
    // A returned function that references top-level globals still works because
    // top-level let/var are always alive (in globals table)
    assert_parity_number(
        r#"
let multiplier = 3;
fn make_multiplier_fn() -> (number) -> number {
    fn apply_multiplier(x: number) -> number {
        return x * multiplier;
    }
    return apply_multiplier;
}
let f = make_multiplier_fn();
f(14);
"#,
        42.0,
    );
}

#[test]
fn test_fn_value_survives_scope_exit_for_globals() {
    // Store fn reference, change the global var, call fn — sees new value
    assert_parity_number(
        r#"
var factor = 2;
fn times_factor(x: number) -> number { return x * factor; }
let saved = times_factor;
factor = 3;
saved(14);
"#,
        42.0,
    );
}

// ============================================================================
// Category G: Edge cases
// ============================================================================

#[test]
fn test_fn_that_calls_another_global_fn() {
    assert_parity_number(
        r#"
fn add(a: number, b: number) -> number { return a + b; }
fn double_add(a: number, b: number) -> number { return add(a, b) + add(a, b); }
double_add(10, 11);
"#,
        42.0,
    );
}

#[test]
fn test_parameter_does_not_bleed_to_outer_scope() {
    // A function's parameter should not be visible after the function returns
    assert_parity_number(
        r#"
var x = 99;
fn set_inner(y: number) { var x = y; }
set_inner(1);
x;
"#,
        99.0,
    );
}

#[test]
fn test_fn_with_no_captures_works_from_any_context() {
    // Pure function (no external references) works regardless of call site
    assert_parity_number(
        r#"
fn pure_add(a: number, b: number) -> number { return a + b; }
fn caller() -> number { return pure_add(20, 22); }
caller();
"#,
        42.0,
    );
}

#[test]
fn test_multiple_calls_do_not_pollute_scope() {
    // Each call to a function with var declarations gets a fresh scope
    assert_parity_number(
        r#"
fn make_local() -> number {
    var n = 10;
    n = n + 5;
    return n;
}
let a = make_local();
let b = make_local();
a + b;
"#,
        30.0,
    );
}

// ============================================================================
// Category H: Upvalue capture parity — both engines produce identical results
//
// These tests specifically exercise the VM's upvalue capture mechanism
// and verify it matches the interpreter's dynamic scoping behavior.
// ============================================================================

#[test]
fn test_upvalue_multiple_outer_lets() {
    // Inner function captures multiple outer let variables
    assert_parity_number(
        r#"
fn outer() -> number {
    let a = 10;
    let b = 32;
    fn sum() -> number {
        return a + b;
    }
    return sum();
}
outer();
"#,
        42.0,
    );
}

#[test]
fn test_upvalue_arithmetic_with_outer_vars() {
    // Inner function uses outer vars in arithmetic
    assert_parity_number(
        r#"
fn make_adder(base: number) -> number {
    fn add(x: number) -> number {
        return base + x;
    }
    return add(10);
}
make_adder(32);
"#,
        42.0,
    );
}

#[test]
fn test_upvalue_multiple_inner_fns_same_outer() {
    // Two inner functions both capturing the same outer variable
    assert_parity_number(
        r#"
fn outer() -> number {
    let factor = 3;
    fn triple(n: number) -> number {
        return n * factor;
    }
    fn check() -> number {
        return factor * 2;
    }
    return triple(10) + check();
}
outer();
"#,
        36.0,
    );
}

#[test]
fn test_upvalue_conditional_in_outer() {
    // Outer function has branching; inner captures the result
    assert_parity_number(
        r#"
fn outer(x: number) -> number {
    let result = x * 2;
    fn get_result() -> number {
        return result;
    }
    return get_result();
}
outer(21);
"#,
        42.0,
    );
}

#[test]
fn test_upvalue_inner_fn_called_multiple_times() {
    // Inner function can be called multiple times and sees correct captured value
    assert_parity_number(
        r#"
fn outer(n: number) -> number {
    fn double() -> number {
        return n * 2;
    }
    return double() + double();
}
outer(10);
"#,
        40.0,
    );
}

#[test]
fn test_upvalue_three_level_capture() {
    // Innermost function captures from outermost via upvalue chain
    assert_parity_number(
        r#"
fn level1(x: number) -> number {
    fn level2() -> number {
        fn level3() -> number {
            return x + 1;
        }
        return level3();
    }
    return level2();
}
level1(41);
"#,
        42.0,
    );
}

#[test]
fn test_upvalue_string_capture() {
    // Upvalue capture works for string type too
    assert_parity_string(
        r#"
fn outer(name: string) -> string {
    fn greet() -> string {
        return name;
    }
    return greet();
}
outer("atlas");
"#,
        "atlas",
    );
}

#[test]
fn test_upvalue_bool_capture() {
    // Upvalue capture works for bool type
    assert_parity_bool(
        r#"
fn outer(flag: bool) -> bool {
    fn check() -> bool {
        return flag;
    }
    return check();
}
outer(true);
"#,
        true,
    );
}

#[test]
fn test_upvalue_sibling_and_capture() {
    // Sibling calls (via scoped globals) AND upvalue capture work together
    assert_parity_number(
        r#"
fn outer(n: number) -> number {
    fn add_one(x: number) -> number {
        return x + 1;
    }
    fn add_n(x: number) -> number {
        return x + n;
    }
    return add_one(add_n(10));
}
outer(5);
"#,
        16.0,
    );
}

#[test]
fn test_upvalue_outer_computation() {
    // Outer function computes a value, inner captures and uses it
    assert_parity_number(
        r#"
fn outer(a: number, b: number) -> number {
    let product = a * b;
    fn get_product() -> number {
        return product;
    }
    return get_product();
}
outer(6, 7);
"#,
        42.0,
    );
}

// ============================================================================
// Category I: Capture-by-value behavioral pins (v0.2 defined semantics)
//
// The VM captures outer locals BY VALUE at inner function definition time.
// These tests pin that behavior explicitly so v0.3 reference-semantics work
// can be measured against a clear baseline.
//
// IMPORTANT: These tests intentionally exercise only the VM, because the
// interpreter uses live dynamic scoping and cannot replicate capture-by-value
// semantics for returned closures (the outer frame no longer exists).
// Where both engines can be exercised, parity helpers are used.
// ============================================================================

/// Helper: run only the VM and return the last value.
fn vm_eval_last(source: &str) -> Value {
    vm_eval(source).unwrap_or(Value::Null)
}

#[test]
fn test_vm_upvalue_captures_let_at_definition_time() {
    // let is immutable — captured value equals definition-time value.
    // Both engines agree: no divergence possible.
    assert_parity_number(
        r#"
fn outer() -> number {
    let x = 10;
    fn get_x() -> number {
        return x;
    }
    return get_x();
}
outer();
"#,
        10.0,
    );
}

#[test]
fn test_vm_upvalue_captures_var_at_definition_time() {
    // VM: inner fn captures var BY VALUE at definition time.
    // The outer var is NOT mutated before the inner fn is called here,
    // so both engines agree.
    assert_parity_number(
        r#"
fn outer() -> number {
    var x = 5;
    fn get_x() -> number {
        return x;
    }
    return get_x();
}
outer();
"#,
        5.0,
    );
}

#[test]
fn test_vm_upvalue_var_mutation_before_inner_def_is_captured() {
    // Outer var is mutated BEFORE inner function is defined.
    // VM captures the value AT DEFINITION TIME of the inner fn (which is 20).
    // Both engines agree because the inner fn is defined after the mutation.
    assert_parity_number(
        r#"
fn outer() -> number {
    var x = 5;
    x = 20;
    fn get_x() -> number {
        return x;
    }
    return get_x();
}
outer();
"#,
        20.0,
    );
}

#[test]
fn test_vm_upvalue_is_snapshot_not_live_reference() {
    // VM: outer var is mutated AFTER inner fn is defined.
    // VM sees the captured snapshot (5), not the updated value (99).
    // This is the defined v0.2 capture-by-value behavior.
    // NOTE: Only VM is tested here — interpreter uses live scoping and would see 99.
    let result = vm_eval_last(
        r#"
fn outer() -> number {
    var x = 5;
    fn get_x() -> number {
        return x;
    }
    x = 99;
    return get_x();
}
outer();
"#,
    );
    assert_eq!(
        result,
        Value::Number(5.0),
        "VM must return captured snapshot (5), not updated value (99)"
    );
}

#[test]
fn test_vm_returned_closure_accesses_top_level_globals() {
    // A returned inner function that references top-level globals works after scope exit.
    // Top-level globals are always alive — both engines agree.
    assert_parity_number(
        r#"
let base = 100;
fn make_fn() -> (number) -> number {
    fn add_base(n: number) -> number {
        return n + base;
    }
    return add_base;
}
let f = make_fn();
f(42);
"#,
        142.0,
    );
}

#[test]
fn test_vm_upvalue_param_captured_correctly() {
    // Outer function parameter is captured by inner fn.
    // Both engines: parameter value at call time is the captured value.
    assert_parity_number(
        r#"
fn make_adder(n: number) -> number {
    fn add(x: number) -> number {
        return x + n;
    }
    return add(10);
}
make_adder(32);
"#,
        42.0,
    );
}

#[test]
fn test_vm_two_inner_fns_capture_same_var_independently() {
    // Two inner functions capture the same outer var independently at their
    // respective definition times. Since var is not mutated between definitions,
    // both see the same value. Both engines agree.
    assert_parity_number(
        r#"
fn outer() -> number {
    var cap = 7;
    fn get_a() -> number {
        return cap;
    }
    fn get_b() -> number {
        return cap;
    }
    return get_a() + get_b();
}
outer();
"#,
        14.0,
    );
}

// ============================================================================
// Phase 4: Typechecker — AnonFn type resolution
// ============================================================================

fn typecheck_errors(source: &str) -> Vec<String> {
    let mut lexer = atlas_runtime::lexer::Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::parser::Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = atlas_runtime::binder::Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let diagnostics = typechecker.check(&program);
    diagnostics
        .iter()
        .filter(|d| d.level == atlas_runtime::diagnostic::DiagnosticLevel::Error)
        .map(|d| d.message.clone())
        .collect()
}

fn typecheck_ok(source: &str) {
    let errors = typecheck_errors(source);
    assert!(
        errors.is_empty(),
        "Expected no type errors, got: {:?}",
        errors
    );
}

#[test]
fn test_tc_anon_fn_typed_params_produces_function_type() {
    // Typed anon fn — typechecker resolves it without error
    typecheck_ok(
        r#"
let f = fn(x: number) -> number { x + 1; };
"#,
    );
}

#[test]
fn test_tc_anon_fn_untyped_params_no_error() {
    // Arrow fn with untyped params — Unknown type, no crash, no error
    typecheck_ok(
        r#"
let f = (x) => x;
"#,
    );
}

#[test]
fn test_tc_anon_fn_return_type_mismatch_errors() {
    // Declared return type doesn't match body type
    let errors = typecheck_errors(
        r#"
let f = fn(x: number) -> string { x + 1; };
"#,
    );
    assert!(
        !errors.is_empty(),
        "Expected type error for return type mismatch"
    );
}

#[test]
fn test_tc_anon_fn_passed_as_arg_no_error() {
    // Anon fn passed as argument to a higher-order function
    // Atlas function type syntax: (params) -> return, not fn(params) -> return
    typecheck_ok(
        r#"
fn apply(f: (number) -> number, x: number) -> number {
    return f(x);
}
apply(fn(x: number) -> number { x * 2; }, 5);
"#,
    );
}

#[test]
fn test_tc_anon_fn_arrow_syntax_no_error() {
    // Arrow syntax — no declared types, typechecks without error
    typecheck_ok(
        r#"
let double = (x) => x;
double(4);
"#,
    );
}

#[test]
fn test_tc_anon_fn_captures_borrow_param_errors() {
    // Capturing a `borrow` param in a closure is an error
    // Atlas function type syntax: () -> number, not fn() -> number
    let errors = typecheck_errors(
        r#"
fn outer(borrow x: number) -> () -> number {
    return fn() -> number { x; };
}
"#,
    );
    assert!(
        !errors.is_empty(),
        "Expected error for capturing borrow param in closure"
    );
    assert!(
        errors
            .iter()
            .any(|e| e.contains("borrow") || e.contains("AT3040")),
        "Expected borrow capture error, got: {:?}",
        errors
    );
}

// ============================================================================
// Phase 05: Compiler — Emit MakeClosure for Anonymous Functions
// VM-path and parity tests. Parity tests re-enabled in Phase 06 (interpreter AnonFn support).
// (interpreter AnonFn support).
// ============================================================================

fn assert_vm_number(source: &str, expected: f64) {
    let result = vm_eval(source).unwrap_or(Value::Null);
    assert_eq!(
        result,
        Value::Number(expected),
        "VM wrong for:\n{}\n  got: {:?}",
        source,
        result
    );
}

fn assert_vm_string(source: &str, expected: &str) {
    let result = vm_eval(source).unwrap_or(Value::Null);
    assert_eq!(
        result,
        Value::string(expected.to_string()),
        "VM wrong for:\n{}\n  got: {:?}",
        source,
        result
    );
}

// --- Block-form anonymous function (fn expression) ---

#[test]
fn test_anon_fn_block_form_basic() {
    // Phase 05 AC: fn expression with explicit return compiles and executes
    assert_vm_number(
        r#"
let f = fn(x: number) -> number { return x + 1; };
f(5);
"#,
        6.0,
    );
}

#[test]
fn test_anon_fn_block_form_two_params() {
    assert_vm_number(
        r#"
let add = fn(a: number, b: number) -> number { return a + b; };
add(3, 4);
"#,
        7.0,
    );
}

#[test]
fn test_anon_fn_block_form_returns_null_without_explicit_return() {
    let result = vm_eval(
        r#"
let f = fn() { };
f();
"#,
    );
    assert_eq!(result, Some(Value::Null));
}

#[test]
fn test_anon_fn_block_form_string_result() {
    assert_vm_string(
        r#"
let greet = fn(name: string) -> string { return "hello " + name; };
greet("world");
"#,
        "hello world",
    );
}

// --- Arrow-form anonymous function ---

#[test]
fn test_anon_fn_arrow_form_basic() {
    // Phase 05 AC: `let f = (x) => x * 2; f(3);` → 6
    assert_vm_number(
        r#"
let f = (x) => x * 2;
f(3);
"#,
        6.0,
    );
}

#[test]
fn test_anon_fn_arrow_form_arithmetic() {
    assert_vm_number(
        r#"
let square = (x) => x * x;
square(7);
"#,
        49.0,
    );
}

#[test]
fn test_anon_fn_arrow_form_no_params() {
    assert_vm_number(
        r#"
let forty_two = () => 42;
forty_two();
"#,
        42.0,
    );
}

#[test]
fn test_anon_fn_arrow_form_two_params() {
    assert_vm_number(
        r#"
let mul = (a, b) => a * b;
mul(6, 7);
"#,
        42.0,
    );
}

#[test]
fn test_anon_fn_arrow_form_string_concat() {
    assert_vm_string(
        r#"
let join = (a, b) => a + b;
join("foo", "bar");
"#,
        "foobar",
    );
}

// --- Upvalue capture ---

#[test]
fn test_anon_fn_captures_outer_param() {
    // Phase 05 AC: closure capturing an outer variable compiles with correct upvalue count
    assert_vm_number(
        r#"
fn make_adder(n: number) -> number {
    let f = (x) => x + n;
    return f(10);
}
make_adder(5);
"#,
        15.0,
    );
}

#[test]
fn test_anon_fn_captures_outer_param_block_form() {
    assert_vm_number(
        r#"
fn make_adder(n: number) -> number {
    let f = fn(x: number) -> number { return x + n; };
    return f(10);
}
make_adder(3);
"#,
        13.0,
    );
}

#[test]
fn test_anon_fn_captures_multiple_outer_vars() {
    assert_vm_number(
        r#"
fn compute(a: number, b: number) -> number {
    let f = (x) => x + a + b;
    return f(1);
}
compute(2, 3);
"#,
        6.0,
    );
}

// --- Anonymous function as argument (higher-order) ---

#[test]
fn test_anon_fn_passed_as_arg() {
    assert_vm_number(
        r#"
fn apply(f: any, x: number) -> number {
    return f(x);
}
apply((n) => n * 3, 4);
"#,
        12.0,
    );
}

#[test]
fn test_anon_fn_arrow_passed_as_arg() {
    assert_vm_number(
        r#"
fn apply(f: any, x: number) -> number {
    return f(x);
}
apply((n) => n + 100, 5);
"#,
        105.0,
    );
}

// --- Returned from a function ---

#[test]
fn test_anon_fn_returned_from_function() {
    assert_vm_number(
        r#"
fn make_multiplier(factor: number) -> any {
    return (x) => x * factor;
}
let double = make_multiplier(2);
double(21);
"#,
        42.0,
    );
}

#[test]
fn test_anon_fn_parity_fn_expr() {
    // Both engines: explicit return ensures identical result (interpreter also returns last
    // expr value, VM pops it — use return to guarantee parity).
    assert_parity_number(
        r#"
let f = fn(x: number) -> number { return x + 1; };
f(5);
"#,
        6.0,
    );
}

#[test]
fn test_anon_fn_parity_arrow() {
    assert_parity_number(
        r#"
let f = (x) => x * 2;
f(3);
"#,
        6.0,
    );
}

#[test]
fn test_anon_fn_parity_capture() {
    assert_parity_number(
        r#"
fn outer(n: number) -> number {
    let f = (x) => x + n;
    return f(10);
}
outer(5);
"#,
        15.0,
    );
}

// ============================================================================
// Phase 07: Parity Hardening — var-mutation capture semantics
// Canonical rule: closure captures `var` BY VALUE at creation time.
// Outer `var` mutations after closure creation are NOT visible inside.
// Both engines must agree (VM snapshot semantics; interpreter now aligned).
// ============================================================================

/// Outer `var` mutation after closure creation is not visible inside (arrow form).
#[test]
fn test_parity_var_mutation_after_closure_creation_not_visible() {
    assert_parity_number(
        r#"
fn outer() -> number {
    var x = 10;
    let f = () => x;
    x = 99;
    return f();
}
outer();
"#,
        10.0,
    );
}

/// Outer `var` mutation after closure creation is not visible inside (block form).
#[test]
fn test_parity_var_mutation_after_closure_creation_block_form() {
    assert_parity_number(
        r#"
fn outer() -> number {
    var x = 10;
    let f = fn() -> number { return x; };
    x = 99;
    return f();
}
outer();
"#,
        10.0,
    );
}

/// Mutation of a `var` INSIDE a closure works correctly within the call.
#[test]
fn test_parity_var_mutation_inside_closure_works() {
    assert_parity_number(
        r#"
fn outer() -> number {
    var counter = 0;
    let inc = () => counter + 1;
    return inc();
}
outer();
"#,
        1.0,
    );
}

/// Multiple captures, only one mutated — only mutated one frozen at creation value.
#[test]
fn test_parity_partial_var_mutation_after_capture() {
    assert_parity_number(
        r#"
fn outer() -> number {
    var a = 1;
    var b = 2;
    let f = () => a + b;
    a = 100;
    return f();
}
outer();
"#,
        3.0,
    );
}

/// `let` (immutable) binding: no mutation possible; both engines agree on snapshot.
#[test]
fn test_parity_let_binding_captured_stable() {
    assert_parity_number(
        r#"
fn outer() -> number {
    let x = 42;
    let f = () => x;
    return f();
}
outer();
"#,
        42.0,
    );
}

/// Two closures created at different points see their respective snapshots.
#[test]
fn test_parity_two_closures_different_snapshots() {
    assert_parity_number(
        r#"
fn outer() -> number {
    var x = 1;
    let f1 = () => x;
    x = 2;
    let f2 = () => x;
    return f1() + f2();
}
outer();
"#,
        3.0,
    );
}

// ============================================================================
// Phase 09: Stdlib Higher-Order Function Audit
// All 9 HOF free functions tested with fn-expr and arrow form, both engines.
// Also verifies Value::Closure (closures with upvalues) works in all HOFs.
// Note: HOFs in Atlas are free functions: map(arr, fn), not arr.map(fn).
// ============================================================================

// --- map ---

#[test]
fn test_hof_map_fn_expr() {
    assert_parity_number(
        r#"
let arr = [1, 2, 3];
let result = map(arr, fn(x: number) -> number { return x * 2; });
result[2];
"#,
        6.0,
    );
}

#[test]
fn test_hof_map_arrow() {
    assert_parity_number(
        r#"
let arr = [1, 2, 3];
let result = map(arr, (x) => x * 2);
result[0];
"#,
        2.0,
    );
}

#[test]
fn test_hof_map_closure_with_upvalue() {
    // Arrow fn captures outer var (produces Value::Closure in VM)
    assert_parity_number(
        r#"
fn run() -> number {
    let factor = 3;
    let arr = [1, 2, 3];
    let result = map(arr, (x) => x * factor);
    return result[1];
}
run();
"#,
        6.0,
    );
}

// --- filter ---

#[test]
fn test_hof_filter_fn_expr() {
    assert_parity_number(
        r#"
let arr = [1, 2, 3, 4, 5];
let result = filter(arr, fn(x: number) -> bool { return x > 2; });
result[0];
"#,
        3.0,
    );
}

#[test]
fn test_hof_filter_arrow() {
    assert_parity_number(
        r#"
let arr = [1, 2, 3, 4, 5];
let result = filter(arr, (x) => x > 1);
len(result);
"#,
        4.0,
    );
}

#[test]
fn test_hof_filter_closure_with_upvalue() {
    assert_parity_number(
        r#"
fn run() -> number {
    let threshold = 2;
    let arr = [1, 2, 3, 4];
    let result = filter(arr, (x) => x > threshold);
    return len(result);
}
run();
"#,
        2.0,
    );
}

// --- reduce ---

#[test]
fn test_hof_reduce_fn_expr() {
    assert_parity_number(
        r#"
let arr = [1, 2, 3];
reduce(arr, fn(acc: number, x: number) -> number { return acc + x; }, 0);
"#,
        6.0,
    );
}

#[test]
fn test_hof_reduce_arrow() {
    assert_parity_number(
        r#"
let arr = [1, 2, 3, 4];
reduce(arr, (acc, x) => acc + x, 0);
"#,
        10.0,
    );
}

// --- forEach ---

#[test]
fn test_hof_for_each_fn_expr_executes() {
    // forEach returns null; verify it runs without error via side-effect-free path
    let result = vm_eval(
        r#"
let arr = [1, 2, 3];
forEach(arr, fn(x: number) { });
42;
"#,
    );
    assert_eq!(result, Some(Value::Number(42.0)));
}

#[test]
fn test_hof_for_each_arrow_executes() {
    let result = vm_eval(
        r#"
let arr = [1, 2, 3];
forEach(arr, (x) => x);
99;
"#,
    );
    assert_eq!(result, Some(Value::Number(99.0)));
}

// --- find ---

#[test]
fn test_hof_find_fn_expr() {
    assert_parity_number(
        r#"
let arr = [1, 2, 3, 4];
find(arr, fn(x: number) -> bool { return x == 3; });
"#,
        3.0,
    );
}

#[test]
fn test_hof_find_arrow() {
    assert_parity_number(
        r#"
let arr = [10, 20, 30];
find(arr, (x) => x == 20);
"#,
        20.0,
    );
}

// --- some ---

#[test]
fn test_hof_any_fn_expr() {
    assert_parity_bool(
        r#"
let arr = [1, 2, 3];
some(arr, fn(x: number) -> bool { return x > 2; });
"#,
        true,
    );
}

#[test]
fn test_hof_any_arrow() {
    assert_parity_bool(
        r#"
let arr = [1, 2, 3];
some(arr, (x) => x > 10);
"#,
        false,
    );
}

// --- every ---

#[test]
fn test_hof_all_fn_expr() {
    assert_parity_bool(
        r#"
let arr = [1, 2, 3];
every(arr, fn(x: number) -> bool { return x > 0; });
"#,
        true,
    );
}

#[test]
fn test_hof_all_arrow() {
    assert_parity_bool(
        r#"
let arr = [1, 2, 3];
every(arr, (x) => x > 1);
"#,
        false,
    );
}

// --- sort ---

#[test]
fn test_hof_sort_fn_expr() {
    assert_parity_number(
        r#"
let arr = [3, 1, 2];
let result = sort(arr, fn(a: number, b: number) -> number { return a - b; });
result[0];
"#,
        1.0,
    );
}

#[test]
fn test_hof_sort_arrow() {
    assert_parity_number(
        r#"
let arr = [3, 1, 2];
let result = sort(arr, (a, b) => a - b);
result[2];
"#,
        3.0,
    );
}

// --- flatMap ---

#[test]
fn test_hof_flat_map_fn_expr() {
    assert_parity_number(
        r#"
let arr = [1, 2];
let result = flatMap(arr, fn(x: number) -> any { return [x, x * 10]; });
len(result);
"#,
        4.0,
    );
}

#[test]
fn test_hof_flat_map_arrow() {
    assert_parity_number(
        r#"
let arr = [1, 2, 3];
let result = flatMap(arr, (x) => [x, x]);
result[0];
"#,
        1.0,
    );
}
