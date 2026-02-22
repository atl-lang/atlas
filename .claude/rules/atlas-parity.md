---
paths:
  - "**/interpreter/**/*.rs"
  - "**/vm/**/*.rs"
  - "**/compiler/**/*.rs"
---

# Parity is Sacred

You are working in an Atlas execution engine. Every behavior change MUST produce
identical output in both the interpreter (`interpreter/`) and the VM (`vm/`).

**Parity break = BLOCKING. Do not ship. Do not move to next phase.**

## The Rule

If you change behavior in the interpreter → change it in the VM.
If you change behavior in the VM → change it in the interpreter.
If you change the compiler → verify both engines still produce identical output.

No exceptions. Not even for "minor" changes.

## Verifying Parity

```rust
// Use the assert_parity pattern in tests
fn assert_parity(source: &str, expected: &str) {
    let interp = eval_interpreter(source);
    let vm = eval_vm(source);
    assert_eq!(interp, vm, "Parity divergence:\nSource: {}", source);
    assert_eq!(interp.unwrap(), expected);
}
```

Every phase that touches interpreter or VM must include parity tests.
Add them to the relevant domain file in `crates/atlas-runtime/tests/` — not a new file.

## CoW Write-Back Pattern (both engines must implement identically)

Collection mutation builtins return a NEW collection. Both engines write it back:
- **Interpreter:** `apply_cow_writeback()` in `interpreter/mod.rs`
- **VM:** `emit_cow_writeback_if_needed()` in `compiler/expr.rs`

If you change how one engine handles collection mutation, update both.

## Ownership Enforcement (Block 2+, debug mode only)

`#[cfg(debug_assertions)]` guards on all ownership checks. Both engines must fire
identical error messages for identical violations. The runtime behavior is the spec
that v0.4's static verifier must match — get it right in both engines.
