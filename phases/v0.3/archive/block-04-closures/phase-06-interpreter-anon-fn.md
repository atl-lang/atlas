# Phase 06: Interpreter — Evaluate Anonymous Functions

**Block:** 4 (Closures + Anonymous Functions)
**Depends on:** Phase 04 complete (can run parallel to Phase 05)

## Current State (verified 2026-02-23)

`interpreter/expr.rs` has a stub arm for `Expr::AnonFn` returning an error (added Phase 01).
The interpreter handles `Value::Closure` calls at `interpreter/mod.rs:655` — calling a closure works, creating one from an anonymous expression does not.
The interpreter uses dynamic scoping: outer variables are accessed via the live scope stack at call time, not captured at closure creation.

## Goal

Evaluate `Expr::AnonFn` in the interpreter, returning a `Value::Closure` that captures the current environment.

## Implementation

In `interpreter/expr.rs`, fill the `Expr::AnonFn` arm:

```rust
Expr::AnonFn { params, return_type, body, span } => {
    // Build a FunctionRef from the anon fn definition
    let func = FunctionRef {
        name: format!("__anon_{}", span.start),
        arity: params.len(),
        // Interpreter closures don't use bytecode — store params + body
        // for re-evaluation at call time
        params: params.clone(),
        body: Some(body.clone()),  // store AST body for interpreter eval
        ..Default::default()
    };

    // Capture current scope snapshot for non-dynamic variables
    // For Copy types: snapshot the value now
    // For non-Copy types: the interpreter's dynamic scoping handles it
    let upvalues = Arc::new(Vec::new()); // interpreter uses live scope; upvalues empty

    Ok(Value::Closure(ClosureRef { func, upvalues }))
}
```

### Interpreter closure call

When the interpreter calls a `Value::Closure` whose `func.body` is `Some(ast_body)`:
- Push a new scope
- Bind params to argument values
- Eval the body expression
- Pop scope
- Return the result

Verify `interpreter/mod.rs:655` handles this path. If it only handles bytecode closures, extend it to handle AST-body closures.

### Dynamic vs snapshot semantics

The interpreter's strength is live scope access — outer `var` mutations are visible to the closure at call time. This is correct interpreter behavior. The VM snapshots at creation time. Phase 07 reconciles these where they must agree.

## Acceptance Criteria

- [ ] `let f = fn(x: number) -> number { x + 1; }; f(5);` interpreter returns `6`
- [ ] `let f = (x) => x * 2; f(3);` interpreter returns `6`
- [ ] Closure captures outer `let` correctly
- [ ] `cargo test` passes (interpreter path)
