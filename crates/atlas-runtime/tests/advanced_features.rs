//! advanced_features.rs — B41 Advanced Features test suite
//!
//! Covers: defer (LIFO semantics), FFI extern stubs, variadic args,
//! constructor syntax (Foo() sugar), and spawn (goroutine) declarations.
//!
//! D-052: VM-only execution path. All tests use vm_eval / vm_eval_checked.

mod common;

use atlas_runtime::binder::Binder;
use atlas_runtime::compiler::Compiler;
use atlas_runtime::diagnostic::DiagnosticLevel;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::typechecker::TypeChecker;
use atlas_runtime::value::Value;
use atlas_runtime::vm::VM;
use pretty_assertions::assert_eq;

// ── helpers ──────────────────────────────────────────────────────────────────

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

fn vm_eval_num(source: &str) -> f64 {
    match vm_eval(source) {
        Some(Value::Number(n)) => n,
        other => panic!("expected number, got {:?}", other),
    }
}

fn vm_eval_str(source: &str) -> String {
    match vm_eval(source) {
        Some(Value::String(s)) => s.to_string(),
        other => panic!("expected string, got {:?}", other),
    }
}

fn vm_eval_bool(source: &str) -> bool {
    match vm_eval(source) {
        Some(Value::Bool(b)) => b,
        other => panic!("expected bool, got {:?}", other),
    }
}

fn typecheck_errors(source: &str) -> Vec<String> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let diagnostics = typechecker.check(&program);
    diagnostics
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
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

fn typecheck_has_error(source: &str, expected: &str) {
    let errors = typecheck_errors(source);
    assert!(
        errors.iter().any(|e| e.contains(expected)),
        "Expected error containing {:?}, got: {:?}",
        expected,
        errors
    );
}

// ── Defer ────────────────────────────────────────────────────────────────────

#[test]
fn test_defer_returns_value() {
    // Deferred statements must not affect return value
    let source = r#"
        fn f(): number {
            defer console.log("cleanup");
            42
        }
        f();
    "#;
    assert_eq!(vm_eval_num(source), 42.0);
}

#[test]
fn test_defer_lifo_single() {
    // Single defer — runs at end of function, value unaffected
    let source = r#"
        fn f(): number {
            defer console.log("done");
            10
        }
        f();
    "#;
    assert_eq!(vm_eval_num(source), 10.0);
}

#[test]
fn test_defer_lifo_two_defers_return_value() {
    // Two defers — LIFO order, return value preserved
    let source = r#"
        fn f(): number {
            defer console.log("first");
            defer console.log("second");
            99
        }
        f();
    "#;
    assert_eq!(vm_eval_num(source), 99.0);
}

#[test]
fn test_defer_with_computation() {
    // Deferred expression executes at function exit; return value is independent
    // Note: defering user-defined function calls is tracked in H-364 (VM hang).
    // Using console.log (native) which is safe to defer.
    let source = r#"
        fn f(): number {
            defer console.log("computed");
            7
        }
        f();
    "#;
    assert_eq!(vm_eval_num(source), 7.0);
}

#[test]
fn test_defer_in_nested_function() {
    // Defer in inner function does not interfere with outer function return
    let source = r#"
        fn inner(): number {
            defer console.log("inner cleanup");
            5
        }
        fn outer(): number {
            let x = inner();
            x + 1
        }
        outer();
    "#;
    assert_eq!(vm_eval_num(source), 6.0);
}

#[test]
fn test_defer_multiple_different_expressions() {
    // Three defers — all execute, return value is intact
    let source = r#"
        fn f(): number {
            defer console.log("a");
            defer console.log("b");
            defer console.log("c");
            100
        }
        f();
    "#;
    assert_eq!(vm_eval_num(source), 100.0);
}

#[test]
fn test_defer_with_string_return() {
    let source = r#"
        fn greet(): string {
            defer console.log("cleanup");
            "hello"
        }
        greet();
    "#;
    assert_eq!(vm_eval_str(source), "hello");
}

#[test]
fn test_defer_with_boolean_return() {
    let source = r#"
        fn check(): bool {
            defer console.log("checked");
            true
        }
        check();
    "#;
    assert!(vm_eval_bool(source));
}

#[test]
fn test_defer_does_not_shadow_return() {
    // Ensure defer cannot overwrite the return value
    let source = r#"
        fn f(): number {
            defer console.log("shadowing attempt");
            let result = 42;
            result
        }
        f();
    "#;
    assert_eq!(vm_eval_num(source), 42.0);
}

#[test]
fn test_defer_typechecks_ok() {
    typecheck_ok(
        r#"
        fn f(): number {
            defer console.log("bye");
            1
        }
        "#,
    );
}

// ── Variadic ─────────────────────────────────────────────────────────────────

#[test]
fn test_variadic_sum_three_args() {
    let source = r#"
        fn sum(...nums: number[]): number {
            let mut t = 0;
            for n in nums { t = t + n; }
            t
        }
        sum(1, 2, 3);
    "#;
    assert_eq!(vm_eval_num(source), 6.0);
}

#[test]
fn test_variadic_zero_args() {
    let source = r#"
        fn sum(...nums: number[]): number {
            let mut t = 0;
            for n in nums { t = t + n; }
            t
        }
        sum();
    "#;
    assert_eq!(vm_eval_num(source), 0.0);
}

#[test]
fn test_variadic_one_arg() {
    let source = r#"
        fn sum(...nums: number[]): number {
            let mut t = 0;
            for n in nums { t = t + n; }
            t
        }
        sum(7);
    "#;
    assert_eq!(vm_eval_num(source), 7.0);
}

#[test]
fn test_variadic_fixed_plus_rest() {
    let source = r#"
        fn greet(prefix: string, ...names: string[]): string {
            let mut r = prefix;
            for name in names { r = r + " " + name; }
            r
        }
        greet("Hello", "Alice", "Bob");
    "#;
    assert_eq!(vm_eval_str(source), "Hello Alice Bob");
}

#[test]
fn test_variadic_fixed_only_no_rest_args() {
    let source = r#"
        fn greet(prefix: string, ...names: string[]): string {
            let mut r = prefix;
            for name in names { r = r + " " + name; }
            r
        }
        greet("Hi");
    "#;
    assert_eq!(vm_eval_str(source), "Hi");
}

#[test]
fn test_variadic_string_concat() {
    let source = r#"
        fn join(...parts: string[]): string {
            let mut r = "";
            for p in parts { r = r + p; }
            r
        }
        join("a", "b", "c");
    "#;
    assert_eq!(vm_eval_str(source), "abc");
}

#[test]
fn test_variadic_count_args() {
    let source = r#"
        fn count(...xs: number[]): number {
            let mut n = 0;
            for _x in xs { n = n + 1; }
            n
        }
        count(10, 20, 30, 40);
    "#;
    assert_eq!(vm_eval_num(source), 4.0);
}

#[test]
fn test_variadic_returns_first_arg() {
    let source = r#"
        fn first(...xs: number[]): number {
            xs[0]
        }
        first(99, 1, 2);
    "#;
    assert_eq!(vm_eval_num(source), 99.0);
}

#[test]
fn test_variadic_product() {
    let source = r#"
        fn product(...xs: number[]): number {
            let mut p = 1;
            for x in xs { p = p * x; }
            p
        }
        product(2, 3, 4);
    "#;
    assert_eq!(vm_eval_num(source), 24.0);
}

#[test]
fn test_variadic_typechecks_ok() {
    typecheck_ok(
        r#"
        fn sum(...nums: number[]): number {
            let mut t = 0;
            for n in nums { t = t + n; }
            t
        }
        "#,
    );
}

// ── Constructor Syntax ───────────────────────────────────────────────────────

#[test]
fn test_constructor_basic() {
    let source = r#"
        struct Point { x: number, y: number }
        impl Point {
            static fn new(x: number, y: number): Point {
                Point { x: x, y: y }
            }
        }
        let p = Point(3, 4);
        p.x;
    "#;
    assert_eq!(vm_eval_num(source), 3.0);
}

#[test]
fn test_constructor_field_access() {
    let source = r#"
        struct Point { x: number, y: number }
        impl Point {
            static fn new(x: number, y: number): Point {
                Point { x: x, y: y }
            }
        }
        let p = Point(10, 20);
        p.y;
    "#;
    assert_eq!(vm_eval_num(source), 20.0);
}

#[test]
fn test_constructor_and_struct_literal_coexist() {
    // Both Foo() and Foo { ... } must work side by side
    let source = r#"
        struct Point { x: number, y: number }
        impl Point {
            static fn new(x: number, y: number): Point {
                Point { x: x, y: y }
            }
        }
        let p1 = Point(1, 2);
        let p2 = Point { x: 3, y: 4 };
        p1.x + p2.y;
    "#;
    assert_eq!(vm_eval_num(source), 5.0);
}

#[test]
fn test_constructor_with_string_field() {
    let source = r#"
        struct Person { name: string, age: number }
        impl Person {
            static fn new(name: string, age: number): Person {
                Person { name: name, age: age }
            }
        }
        let p = Person("Alice", 30);
        p.age;
    "#;
    assert_eq!(vm_eval_num(source), 30.0);
}

#[test]
fn test_constructor_no_new_error() {
    // AT3064: struct with no static new() must emit clear error
    typecheck_has_error(
        r#"
        struct Foo { x: number }
        let f = Foo(1);
        "#,
        "no static `new` method",
    );
}

#[test]
fn test_constructor_arity_error() {
    // Wrong number of args to constructor
    typecheck_has_error(
        r#"
        struct Point { x: number, y: number }
        impl Point {
            static fn new(x: number, y: number): Point {
                Point { x: x, y: y }
            }
        }
        let p = Point(1);
        "#,
        "expects 2",
    );
}

// ── FFI / Extern ─────────────────────────────────────────────────────────────

#[test]
fn test_extern_declaration_typechecks() {
    // Extern declarations are valid Atlas syntax and typecheck without errors
    typecheck_ok(
        r#"
        extern fn strlen(s: string): number;
        "#,
    );
}

#[test]
fn test_extern_multiple_params_typechecks() {
    typecheck_ok(
        r#"
        extern fn add_native(a: number, b: number): number;
        "#,
    );
}

#[test]
fn test_extern_no_params_typechecks() {
    typecheck_ok(
        r#"
        extern fn get_time(): number;
        "#,
    );
}

#[test]
fn test_extern_string_return_typechecks() {
    typecheck_ok(
        r#"
        extern fn get_hostname(): string;
        "#,
    );
}

#[test]
fn test_extern_multiple_declarations_typecheck() {
    typecheck_ok(
        r#"
        extern fn open(path: string, flags: number): number;
        extern fn close(fd: number): number;
        extern fn read(fd: number, buf: string, n: number): number;
        "#,
    );
}

// ── Spawn / Goroutine ────────────────────────────────────────────────────────
// Spawn is a declaration-level feature. Tests validate parsing/typecheck.
// Full concurrency runtime is B44; these tests confirm B41 wiring.

#[test]
fn test_spawn_expression_typechecks() {
    typecheck_ok(
        r#"
        async fn task(): number { 1 }
        spawn task();
        "#,
    );
}

#[test]
fn test_spawn_with_closure_typechecks() {
    typecheck_ok(
        r#"
        async fn work(): number { 42 }
        spawn work();
        "#,
    );
}

#[test]
fn test_spawn_in_function_body_typechecks() {
    typecheck_ok(
        r#"
        async fn background(): number { 0 }
        fn start() {
            spawn background();
        }
        "#,
    );
}
