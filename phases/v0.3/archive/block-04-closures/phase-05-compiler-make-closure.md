# Phase 05: Compiler — Emit MakeClosure for Anonymous Functions

**Block:** 4 (Closures + Anonymous Functions)
**Depends on:** Phase 04 complete

## Current State (verified 2026-02-23)

`Opcode::MakeClosure` exists in `bytecode/opcode.rs`.
`compiler/mod.rs` emits `MakeClosure` for named nested functions (`compile_nested_function` at line 22).
`UpvalueContext`, `UpvalueCapture`, `register_upvalue()` are all implemented in `compiler/mod.rs`.
`compiler/expr.rs` has no `Expr::AnonFn` arm — currently returns an error diagnostic (added in Phase 01).

## Goal

Compile `Expr::AnonFn` to bytecode: emit the function body as a nested function constant, push captured upvalues onto the stack, then emit `MakeClosure`.

## Implementation

In `compiler/expr.rs`, fill the `Expr::AnonFn` arm:

```rust
Expr::AnonFn { params, return_type, body, span } => {
    // 1. Generate a synthetic name for the anonymous function constant
    //    e.g., "__anon_<offset>" using current bytecode offset for uniqueness
    let name = format!("__anon_{}", self.bytecode.current_offset());

    // 2. Compile the function body as a nested function (same as compile_nested_function)
    //    Push upvalue context, compile params + body, pop upvalue context
    //    Record the resulting FunctionRef constant

    // 3. Push the function constant index: emit LoadConst
    self.bytecode.emit(Opcode::LoadConst, *span);
    self.bytecode.emit_u16(const_idx as u16);

    // 4. Push captured upvalue values onto the stack (in capture order)
    for (_, capture) in &upvalue_ctx.captures {
        match capture {
            UpvalueCapture::Local(idx) => {
                self.bytecode.emit(Opcode::GetLocal, *span);
                self.bytecode.emit_u16(*idx as u16);
            }
            UpvalueCapture::Upvalue(idx) => {
                self.bytecode.emit(Opcode::GetUpvalue, *span);
                self.bytecode.emit_u16(*idx as u16);
            }
        }
    }

    // 5. Emit MakeClosure with upvalue count
    self.bytecode.emit(Opcode::MakeClosure, *span);
    self.bytecode.emit_u16(n_upvalues as u16);

    Ok(())
}
```

### Copy vs Move capture

Before pushing each upvalue, check if the captured value's type is `Copy` (from typechecker annotation). For `Copy` types, the value is already on the stack as a copy. For non-`Copy` types, the variable in the outer scope should be marked as moved — emit a `SetLocal` with a sentinel or simply rely on the typechecker having already validated it.

### Arrow fn body

Arrow form: `body` is a bare `Expr`, not a `Block`. Compile it as an expression + implicit return. No `Opcode::Pop` after the expression — the value stays on the stack as the return value.

## Acceptance Criteria

- [ ] `let f = fn(x: number) -> number { x + 1; }; f(5);` compiles and VM executes correctly → `6`
- [ ] `let f = (x) => x * 2; f(3);` compiles and VM executes correctly → `6`
- [ ] Closure capturing an outer variable compiles with correct upvalue count
- [ ] `cargo test` passes (VM path)
