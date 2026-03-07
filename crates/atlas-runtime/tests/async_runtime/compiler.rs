// Phase 08: Compiler — bytecode shape tests for async/await
//
// Verifies that the Atlas compiler emits correct bytecode for async constructs:
// - async fn metadata (FunctionRef.is_async)
// - WrapFuture before Return in async fn bodies
// - Await opcode emission
// - AsyncCall dispatch for known async callees
// - Sync fn regression (no AsyncCall emitted)

use atlas_runtime::binder::Binder;
use atlas_runtime::bytecode::Opcode;
use atlas_runtime::compiler::Compiler;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::typechecker::TypeChecker;
use atlas_runtime::value::Value;

fn compile_to_bytecode(source: &str) -> atlas_runtime::bytecode::Bytecode {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);
    let mut compiler = Compiler::new();
    compiler.compile(&program).expect("Compilation failed")
}

/// 1. `async fn foo()` produces a FunctionRef with `is_async: true`.
#[test]
fn test_async_fn_produces_is_async_true() {
    let bytecode = compile_to_bytecode(
        r#"
        async fn foo() -> number {
            return 42;
        }
        "#,
    );

    let has_async_fn_ref = bytecode.constants.iter().any(|c| {
        if let Value::Function(f) = c {
            f.name.contains("foo") && f.is_async
        } else {
            false
        }
    });
    assert!(
        has_async_fn_ref,
        "Expected FunctionRef with is_async=true for async fn foo; constants: {:?}",
        bytecode
            .constants
            .iter()
            .filter_map(|c| if let Value::Function(f) = c {
                Some(format!("{}(is_async={})", f.name, f.is_async))
            } else {
                None
            })
            .collect::<Vec<_>>()
    );
}

/// 2. A sync `fn bar()` does NOT get `is_async: true` — regression guard.
#[test]
fn test_sync_fn_is_async_false() {
    let bytecode = compile_to_bytecode(
        r#"
        fn bar() -> number {
            return 1;
        }
        "#,
    );

    let has_wrong_async_ref = bytecode.constants.iter().any(|c| {
        if let Value::Function(f) = c {
            f.name.contains("bar") && f.is_async
        } else {
            false
        }
    });
    assert!(
        !has_wrong_async_ref,
        "Sync fn bar should have is_async=false"
    );
}

/// 3. An async fn body emits WrapFuture (0xA2) before the final Return (0x61).
#[test]
fn test_async_fn_body_emits_wrap_future_before_return() {
    let bytecode = compile_to_bytecode(
        r#"
        async fn compute() -> number {
            return 10;
        }
        "#,
    );

    let instrs = &bytecode.instructions;

    // Find a WrapFuture (0xA2) immediately followed by Return (0x61).
    let has_wrap_before_return = instrs
        .windows(2)
        .any(|w| w[0] == Opcode::WrapFuture as u8 && w[1] == Opcode::Return as u8);

    assert!(
        has_wrap_before_return,
        "Expected WrapFuture (0xA2) immediately before Return (0x61) in async fn body"
    );
}

/// 4. `await expr` emits Await (0xA1) after the inner expression.
#[test]
fn test_await_emits_await_opcode() {
    let bytecode = compile_to_bytecode(
        r#"
        async fn fetch() -> number {
            return 1;
        }
        async fn main() -> number {
            let x = await fetch();
            return x;
        }
        "#,
    );

    let has_await = bytecode.instructions.contains(&(Opcode::Await as u8));

    assert!(
        has_await,
        "Expected Await opcode (0xA1) in compiled bytecode"
    );
}

/// 5. An async fn call site emits AsyncCall (0xA0), not Call (0x60).
#[test]
fn test_async_call_dispatch() {
    let bytecode = compile_to_bytecode(
        r#"
        async fn work() -> number {
            return 7;
        }
        async fn main() -> number {
            return await work();
        }
        "#,
    );

    let has_async_call = bytecode.instructions.contains(&(Opcode::AsyncCall as u8));

    assert!(
        has_async_call,
        "Expected AsyncCall (0xA0) for call to async fn work"
    );
}
