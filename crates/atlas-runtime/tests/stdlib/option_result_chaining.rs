//! Option and Result method chaining tests (H-328, B42-P03)
//!
//! Verifies: map, andThen, orElse, unwrapOrElse on Option<T> and Result<T,E>.
//! D-052: VM-only execution path.

use super::*;
use atlas_runtime::binder::Binder;
use atlas_runtime::compiler::Compiler;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::value::Value;
use atlas_runtime::vm::VM;
use pretty_assertions::assert_eq;

/// Evaluate Atlas source via VM. Discards parse errors (tail-expression style).
fn vm_eval(source: &str) -> Option<Value> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = atlas_runtime::typechecker::TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).expect("compilation failed");
    let mut vm = VM::new(bytecode);
    vm.run(&SecurityContext::allow_all()).expect("VM failed")
}

fn vm_num(source: &str) -> f64 {
    match vm_eval(source) {
        Some(Value::Number(n)) => n,
        other => panic!("expected number, got {:?}", other),
    }
}

fn vm_option_some_num(source: &str) -> f64 {
    match vm_eval(source) {
        Some(Value::Option(Some(v))) => match *v {
            Value::Number(n) => n,
            other => panic!("expected Some(number), got Some({:?})", other),
        },
        other => panic!("expected Some(number), got {:?}", other),
    }
}

fn vm_option_none(source: &str) {
    match vm_eval(source) {
        Some(Value::Option(None)) => {}
        other => panic!("expected None, got {:?}", other),
    }
}

fn vm_result_ok_num(source: &str) -> f64 {
    match vm_eval(source) {
        Some(Value::Result(Ok(v))) => match *v {
            Value::Number(n) => n,
            other => panic!("expected Ok(number), got Ok({:?})", other),
        },
        other => panic!("expected Ok(number), got {:?}", other),
    }
}

fn vm_result_err(source: &str) {
    match vm_eval(source) {
        Some(Value::Result(Err(_))) => {}
        other => panic!("expected Err(...), got {:?}", other),
    }
}

// ── Option.map ────────────────────────────────────────────────────────────────

#[test]
fn test_option_map_some() {
    assert_eq!(
        vm_option_some_num(
            r#"
        let opt: Option<number> = Some(5);
        opt.map(fn(x: number): number { x * 2 });
    "#
        ),
        10.0
    );
}

#[test]
fn test_option_map_none() {
    vm_option_none(
        r#"
        let opt: Option<number> = None;
        opt.map(fn(x: number): number { x * 2 });
    "#,
    );
}

#[test]
fn test_option_map_chain() {
    assert_eq!(
        vm_option_some_num(
            r#"
        let opt: Option<number> = Some(3);
        opt.map(fn(x: number): number { x + 1 })
           .map(fn(x: number): number { x * 10 });
    "#
        ),
        40.0
    );
}

// ── Option.andThen ────────────────────────────────────────────────────────────

#[test]
fn test_option_and_then_some_returns_some() {
    assert_eq!(
        vm_option_some_num(
            r#"
        let opt: Option<number> = Some(5);
        opt.andThen(fn(x: number): Option<number> { Some(x + 1) });
    "#
        ),
        6.0
    );
}

#[test]
fn test_option_and_then_some_returns_none() {
    vm_option_none(
        r#"
        let opt: Option<number> = Some(5);
        opt.andThen(fn(x: number): Option<number> { None });
    "#,
    );
}

#[test]
fn test_option_and_then_none_short_circuits() {
    vm_option_none(
        r#"
        let opt: Option<number> = None;
        opt.andThen(fn(x: number): Option<number> { Some(x * 99) });
    "#,
    );
}

#[test]
fn test_option_and_then_chain() {
    assert_eq!(
        vm_option_some_num(
            r#"
        let opt: Option<number> = Some(2);
        opt.andThen(fn(x: number): Option<number> { Some(x * 3) })
           .andThen(fn(x: number): Option<number> { Some(x + 1) });
    "#
        ),
        7.0
    );
}

// ── Option.orElse ─────────────────────────────────────────────────────────────

#[test]
fn test_option_or_else_some_passes_through() {
    assert_eq!(
        vm_option_some_num(
            r#"
        let opt: Option<number> = Some(42);
        opt.orElse(fn(): Option<number> { Some(99) });
    "#
        ),
        42.0
    );
}

#[test]
fn test_option_or_else_none_uses_fallback() {
    assert_eq!(
        vm_option_some_num(
            r#"
        let opt: Option<number> = None;
        opt.orElse(fn(): Option<number> { Some(99) });
    "#
        ),
        99.0
    );
}

#[test]
fn test_option_or_else_none_fallback_none() {
    vm_option_none(
        r#"
        let opt: Option<number> = None;
        opt.orElse(fn(): Option<number> { None });
    "#,
    );
}

// ── Option.unwrapOrElse ───────────────────────────────────────────────────────

#[test]
fn test_option_unwrap_or_else_some() {
    assert_eq!(
        vm_num(
            r#"
        let opt: Option<number> = Some(7);
        opt.unwrapOrElse(fn(): number { 0 });
    "#
        ),
        7.0
    );
}

#[test]
fn test_option_unwrap_or_else_none() {
    assert_eq!(
        vm_num(
            r#"
        let opt: Option<number> = None;
        opt.unwrapOrElse(fn(): number { 42 });
    "#
        ),
        42.0
    );
}

// ── Result.map ────────────────────────────────────────────────────────────────

#[test]
fn test_result_map_ok() {
    assert_eq!(
        vm_result_ok_num(
            r#"
        let r: Result<number, string> = Ok(5);
        r.map(fn(x: number): number { x * 10 });
    "#
        ),
        50.0
    );
}

#[test]
fn test_result_map_err_passthrough() {
    vm_result_err(
        r#"
        let r: Result<number, string> = Err("fail");
        r.map(fn(x: number): number { x * 10 });
    "#,
    );
}

#[test]
fn test_result_map_chain() {
    assert_eq!(
        vm_result_ok_num(
            r#"
        let r: Result<number, string> = Ok(3);
        r.map(fn(x: number): number { x + 1 })
         .map(fn(x: number): number { x * 5 });
    "#
        ),
        20.0
    );
}

// ── Result.andThen ────────────────────────────────────────────────────────────

#[test]
fn test_result_and_then_ok_returns_ok() {
    assert_eq!(
        vm_result_ok_num(
            r#"
        let r: Result<number, string> = Ok(10);
        r.andThen(fn(x: number): Result<number, string> { Ok(x + 5) });
    "#
        ),
        15.0
    );
}

#[test]
fn test_result_and_then_ok_returns_err() {
    vm_result_err(
        r#"
        let r: Result<number, string> = Ok(10);
        r.andThen(fn(x: number): Result<number, string> { Err("bad") });
    "#,
    );
}

#[test]
fn test_result_and_then_err_short_circuits() {
    vm_result_err(
        r#"
        let r: Result<number, string> = Err("early");
        r.andThen(fn(x: number): Result<number, string> { Ok(x * 99) });
    "#,
    );
}

#[test]
fn test_result_and_then_chain() {
    assert_eq!(
        vm_result_ok_num(
            r#"
        let r: Result<number, string> = Ok(2);
        r.andThen(fn(x: number): Result<number, string> { Ok(x * 3) })
         .andThen(fn(x: number): Result<number, string> { Ok(x + 1) });
    "#
        ),
        7.0
    );
}

// ── Result.orElse ─────────────────────────────────────────────────────────────

#[test]
fn test_result_or_else_ok_passes_through() {
    assert_eq!(
        vm_result_ok_num(
            r#"
        let r: Result<number, string> = Ok(42);
        r.orElse(fn(e: string): Result<number, string> { Ok(0) });
    "#
        ),
        42.0
    );
}

#[test]
fn test_result_or_else_err_recovers() {
    assert_eq!(
        vm_result_ok_num(
            r#"
        let r: Result<number, string> = Err("oops");
        r.orElse(fn(e: string): Result<number, string> { Ok(99) });
    "#
        ),
        99.0
    );
}

#[test]
fn test_result_or_else_err_stays_err() {
    vm_result_err(
        r#"
        let r: Result<number, string> = Err("oops");
        r.orElse(fn(e: string): Result<number, string> { Err("still bad") });
    "#,
    );
}

// ── Mixed chains ──────────────────────────────────────────────────────────────

#[test]
fn test_option_map_then_unwrap_or_else() {
    assert_eq!(
        vm_num(
            r#"
        let opt: Option<number> = Some(4);
        opt.map(fn(x: number): number { x * x })
           .unwrapOrElse(fn(): number { 0 });
    "#
        ),
        16.0
    );
}

#[test]
fn test_option_and_then_then_or_else() {
    assert_eq!(
        vm_option_some_num(
            r#"
        let opt: Option<number> = Some(0);
        opt.andThen(fn(x: number): Option<number> {
            if x > 0 { return Some(x); } else { return None; }
        }).orElse(fn(): Option<number> { Some(1) });
    "#
        ),
        1.0
    );
}

#[test]
fn test_none_map_or_else_chain() {
    assert_eq!(
        vm_num(
            r#"
        let opt: Option<number> = None;
        opt.map(fn(x: number): number { x + 100 })
           .unwrapOrElse(fn(): number { 7 });
    "#
        ),
        7.0
    );
}
