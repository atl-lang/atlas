---
paths: ["crates/atlas-runtime/src/vm/**", "crates/atlas-runtime/src/compiler/**", "crates/atlas-runtime/src/bytecode/**"]
---

# Atlas VM / Compiler / Bytecode Quick-Ref

**Verified against:** `bytecode/opcode.rs`, `compiler/mod.rs`
**Update trigger:** Any phase adding opcodes or compiler fields — update at GATE 7.

---

## All Opcodes (bytecode/opcode.rs) — 45 total

```
// Constants
Constant = 0x01  [u16 idx]    Null = 0x02    True = 0x03    False = 0x04

// Variables
GetLocal = 0x10  [u16 idx]    SetLocal = 0x11  [u16 idx]
GetGlobal = 0x12 [u16 idx]    SetGlobal = 0x13 [u16 idx]
MakeClosure = 0x14 [u16 func_const_idx, u16 n_upvalues]
GetUpvalue = 0x15 [u16 idx]   SetUpvalue = 0x16 [u16 idx]

// Arithmetic
Add=0x20  Sub=0x21  Mul=0x22  Div=0x23  Mod=0x24  Negate=0x25

// Comparison
Equal=0x30  NotEqual=0x31  Less=0x32  LessEqual=0x33
Greater=0x34  GreaterEqual=0x35

// Logical
Not=0x40  And=0x41  Or=0x42

// Control Flow
Jump=0x50 [i16]  JumpIfFalse=0x51 [i16]  Loop=0x52 [i16]

// Functions
Call=0x60 [u8 arg_count]  Return=0x61

// Arrays
Array=0x70 [u16 size]  GetIndex=0x71  SetIndex=0x72

// Stack
Pop=0x80  Dup=0x81

// Pattern Matching
IsOptionSome=0x90  IsOptionNone=0x91  IsResultOk=0x92  IsResultErr=0x93
ExtractOptionValue=0x94  ExtractResultValue=0x95
IsArray=0x96  GetArrayLen=0x97

// Special
Halt=0xFF
```

## Compiler Struct Fields (compiler/mod.rs:63)

```rust
pub struct Compiler {
    pub(super) bytecode: Bytecode,
    pub(super) locals: Vec<Local>,
    pub(super) scope_depth: usize,
    pub(super) loops: Vec<LoopContext>,
    optimizer: Option<Optimizer>,
    pub(super) monomorphizer: Monomorphizer,
    next_func_id: usize,
    pub(super) current_function_base: usize,
    pub(super) global_mutability: HashMap<String, bool>,
    pub(super) locals_watermark: usize,
    pub(super) upvalue_stack: Vec<UpvalueContext>,
}
```

## Bytecode emit methods (bytecode/mod.rs)

```rust
bytecode.emit(opcode: Opcode, span: Span)
bytecode.emit_u8(byte: u8)
bytecode.emit_u16(value: u16)
bytecode.emit_i16(value: i16)
```

## Key Supporting Structs

```rust
// Local variable slot
struct Local { name: String, depth: usize, mutable: bool, scoped_name: Option<String> }

// Loop break tracking
struct LoopContext { start_offset: usize, break_jumps: Vec<usize> }

// Closure upvalue capture
enum UpvalueCapture { Local(usize), Upvalue(usize) }   // abs local idx OR parent upvalue idx
struct UpvalueContext { parent_base: usize, captures: Vec<(String, UpvalueCapture)> }
```

## Parity Rule

**Every opcode handled in the VM must have a corresponding emit in the compiler.**
**Every construct handled in the interpreter must have a VM equivalent.**
Parity break = BLOCKING. Never ship a phase with divergence.
