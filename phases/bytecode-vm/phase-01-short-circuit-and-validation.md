# Phase 01: Bytecode Validator

## Status: Short-circuit ALREADY DONE

Short-circuit evaluation for `&&`/`||` was fully implemented in `compiler/expr.rs`
using `Dup + JumpIfFalse` (AND) and `Dup + Not + JumpIfFalse` (OR). `Opcode::And`
and `Opcode::Or` are dead stubs — compiler never emits them. VM tests pass.

**This phase = bytecode validator only.**

---

## Objective

Create `bytecode/validator.rs` — static analysis of bytecode before VM execution.
Catches malformed bytecode (bad jumps, constant overruns, stack underflow) before
it causes cryptic runtime panics.

---

## Files

**Create:** `crates/atlas-runtime/src/bytecode/validator.rs` (~400 lines)
**Update:** `crates/atlas-runtime/src/bytecode/mod.rs` (~5 lines — add module)
**Tests:** `crates/atlas-runtime/tests/bytecode_validator_tests.rs` (~300 lines)

---

## Validator API

```rust
pub struct ValidationError {
    pub kind: ValidationErrorKind,
    pub offset: usize,
}

pub enum ValidationErrorKind {
    UnknownOpcode(u8),
    JumpOutOfBounds { target: usize, len: usize },
    ConstantIndexOutOfBounds { index: usize, pool_size: usize },
    StackUnderflow { depth: i32, needed: i32, op: &'static str },
    TruncatedInstruction { opcode: &'static str },
    MissingHalt,
}

pub fn validate(bytecode: &Bytecode) -> Result<(), Vec<ValidationError>>;
```

---

## Implementation

### Opcode operand table (from opcode.rs)

| Opcode | Operand bytes | Stack delta |
|--------|--------------|-------------|
| Constant | u16 | +1 |
| Null, True, False | 0 | +1 |
| GetLocal, GetGlobal | u16 | +1 |
| SetLocal, SetGlobal | u16 | 0 (peek, no pop in atlas semantics) |
| Add, Sub, Mul, Div, Mod | 0 | -1 (pop 2, push 1) |
| Negate, Not | 0 | 0 (pop 1, push 1) |
| Equal..GreaterEqual | 0 | -1 |
| And, Or | 0 | -1 (dead but defined) |
| Jump | i16 | 0 |
| JumpIfFalse | i16 | -1 |
| Loop | i16 | 0 |
| Call | u8 | -(arg_count) + 1 (net: -(arg_count-1), but arg_count unknown statically — skip stack check for Call) |
| Return | 0 | drain (exit frame) |
| Array | u16 | -(size) + 1 |
| GetIndex | 0 | -1 |
| SetIndex | 0 | -2 (pops array, index, value; no push) |
| Pop | 0 | -1 |
| Dup | 0 | +1 |
| IsOptionSome..GetArrayLen | 0 | 0 (pop 1, push 1) |
| Halt | 0 | 0 |

### Algorithm

1. **Pass 1 — decode:** Walk instructions byte-by-byte, building `Vec<(offset, Opcode, operand)>`. Emit `UnknownOpcode` or `TruncatedInstruction` on bad bytes. Record all jump operands.
2. **Pass 2 — jump targets:** For each jump, compute absolute target. Emit `JumpOutOfBounds` if outside `[0, instructions.len())`.
3. **Pass 3 — constant refs:** For each `Constant`/`GetGlobal`/`SetGlobal`, verify index < `constants.len()`. Emit `ConstantIndexOutOfBounds`.
4. **Pass 4 — stack depth:** Linear walk, tracking depth as i32. On underflow (depth < 0), emit `StackUnderflow`. Skip stack tracking after `Call` (arity not statically available without type info).
5. **Termination:** Warn if last reachable instruction isn't `Halt` or `Return`.

Collect all errors (don't stop at first), return `Ok(())` or `Err(errors)`.

---

## Gates

### GATE -1: Sanity
```bash
cargo clean && cargo check -p atlas-runtime
```

### GATE 1: Implement + unit tests
```bash
cargo test -p atlas-runtime --test bytecode_validator_tests
```
Minimum: 40 tests.

### GATE 2: Clippy
```bash
cargo clippy -p atlas-runtime -- -D warnings
```

### GATE 3: Format
```bash
cargo fmt -p atlas-runtime
```

### GATE 4: Full suite (pre-existing 6 async failures OK)
```bash
cargo test -p atlas-runtime 2>&1 | grep "test result"
```

---

## Acceptance

- `validate()` catches unknown opcodes
- `validate()` catches jump targets outside bytecode bounds
- `validate()` catches constant pool index overruns
- `validate()` catches stack underflow
- Valid bytecode (from compiler) passes with `Ok(())`
- 40+ tests pass
- Zero clippy warnings
- No changes to VM execution path (validator is opt-in, not wired into VM yet)
