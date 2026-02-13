# Atlas Bytecode Specification

**Version:** v0.2 (Draft)
**Status:** Living document

---

## Overview

Atlas bytecode is a stack-based instruction set for the Atlas VM. Programs are compiled from AST to bytecode for faster execution compared to tree-walking interpretation.

**Key principle:** Bytecode execution must be identical to interpreter execution (parity requirement).

---

## VM Architecture

### Stack-Based

- Operands pushed to stack
- Operations pop operands, push results
- Local variables accessed by index
- Function calls use stack frames

### Components

1. **Bytecode:** Sequence of instructions
2. **Constant pool:** Literals (numbers, strings)
3. **Stack:** Operand stack (grows/shrinks during execution)
4. **Globals:** Global variable storage
5. **Call stack:** Function activation records
6. **Debug info:** Source location mapping

---

## Instruction Set

### Core Instructions (v0.1)

**Constants:**
- `PUSH_CONST <index>` - Push constant from pool

**Variables:**
- `LOAD_LOCAL <index>` - Push local variable
- `STORE_LOCAL <index>` - Pop and store to local
- `LOAD_GLOBAL <index>` - Push global variable
- `STORE_GLOBAL <index>` - Pop and store to global

**Arithmetic:**
- `ADD` - Pop b, pop a, push a+b
- `SUB` - Pop b, pop a, push a-b
- `MUL` - Pop b, pop a, push a*b
- `DIV` - Pop b, pop a, push a/b
- `MOD` - Pop b, pop a, push a%b
- `NEGATE` - Pop a, push -a

**Comparison:**
- `EQ` - Pop b, pop a, push a==b
- `NE` - Pop b, pop a, push a!=b
- `LT` - Pop b, pop a, push a<b
- `LE` - Pop b, pop a, push a<=b
- `GT` - Pop b, pop a, push a>b
- `GE` - Pop b, pop a, push a>=b

**Logical:**
- `NOT` - Pop a, push !a
- `AND` - Short-circuit logical AND
- `OR` - Short-circuit logical OR

**Control Flow:**
- `JMP <offset>` - Unconditional jump
- `JMP_IF_FALSE <offset>` - Pop value, jump if false
- `JMP_IF_TRUE <offset>` - Pop value, jump if true

**Functions:**
- `CALL <arg_count>` - Call function (top of stack)
- `RET` - Return from function
- `PUSH_FN <index>` - Push function reference

**Arrays:**
- `BUILD_ARRAY <count>` - Pop count elements, build array
- `GET_INDEX` - Pop index, pop target, push target[index]
- `SET_INDEX` - Pop value, pop index, pop target, set target[index]=value

**Other:**
- `POP` - Discard top of stack
- `DUP` - Duplicate top of stack
- `PRINT` - Pop and print value

### Extended Instructions (v0.2+)

**Pattern Matching:**
- `MATCH_CONSTRUCTOR <tag>` - Match constructor pattern
- `MATCH_LITERAL <value>` - Match literal pattern
- `BIND_PATTERN <index>` - Bind pattern variable

**Generics:**
- `SPECIALIZE <type_args>` - Instantiate generic function

---

## Constant Pool

### Purpose

Store compile-time constants to avoid duplicating values.

### Contents

- Numbers (f64)
- Strings (Rc<String>)
- Function metadata (name, arity, bytecode offset)

### Encoding

Each constant has an index. `PUSH_CONST <index>` references pool entry.

**Example:**
```
Pool:
  0: 42.0 (number)
  1: "hello" (string)
  2: add (function)

Bytecode:
  PUSH_CONST 0  // Push 42.0
  PUSH_CONST 1  // Push "hello"
```

---

## Stack Frames

### Call Frame Structure

```rust
struct CallFrame {
    function: FunctionRef,    // Which function
    ip: usize,                // Instruction pointer
    stack_base: usize,        // Base of this frame's stack
    locals: Vec<Value>,       // Local variables
}
```

### Function Calls

1. Push arguments to stack
2. Push function reference
3. `CALL <arg_count>` instruction
4. VM creates new frame:
   - Save current IP
   - Set stack_base
   - Allocate locals
   - Bind parameters
   - Jump to function bytecode
5. Execute function body
6. `RET` instruction
7. VM restores frame:
   - Pop return value
   - Restore caller IP
   - Destroy frame
   - Push return value

---

## Compilation

### AST to Bytecode

Direct compilation (no IR in v0.1/v0.2):

1. **Expression compilation:**
   - Emit instructions to compute value
   - Result left on stack top

2. **Statement compilation:**
   - Emit instructions for side effects
   - Clean up stack after

3. **Function compilation:**
   - Add to constant pool
   - Emit separate bytecode chunk

### Example

**Source:**
```atlas
let x = 2 + 3;
print(str(x));
```

**Bytecode:**
```
PUSH_CONST 0     // 2
PUSH_CONST 1     // 3
ADD              // 2+3 = 5
STORE_GLOBAL 0   // x = 5

LOAD_GLOBAL 0    // x
PUSH_FN 0        // str function
CALL 1           // str(x)
PRINT            // print(...)
```

---

## Debug Information

### Source Mapping

Each instruction maps to source location:

```rust
struct DebugInfo {
    offset: usize,    // Bytecode offset
    line: u32,        // Source line
    column: u32,      // Source column
    length: usize,    // Span length
}
```

### Error Reporting

When VM encounters error:
1. Look up current IP in debug info
2. Find source location
3. Emit diagnostic with file/line/column

**See:** `docs/specification/json-formats.md` for debug info format

---

## Bytecode Format (.atb)

### File Structure

```
Magic: "ATB\0" (4 bytes)
Version: 1 (u32)
Constant Pool Count (u32)
  [Constant entries]
Debug Info Count (u32)
  [Debug info entries]
Bytecode Length (u32)
  [Bytecode instructions]
```

### Serialization

- Numbers: Little-endian f64
- Strings: Length-prefixed UTF-8
- Instructions: 1-byte opcode + operands

**See:** `docs/reference/bytecode-format.md` for complete format spec

---

## Optimization (Future)

### Not in v0.2

Current compiler emits naive bytecode (no optimization).

### Planned (v0.3+)

- Constant folding
- Dead code elimination
- Peephole optimization
- Register allocation (if switch to register VM)
- Inline caching
- JIT compilation

---

## Verification

### Bytecode Validator

Before execution, VM validates:

- Stack depth never negative
- All jumps target valid instructions
- Constant pool indices in range
- Local variable indices in range
- Function arity matches call sites

### Safety

Invalid bytecode rejected before execution (fail-fast).

---

## Disassembly

### Debug Tool

`atlas disasm` shows human-readable bytecode:

```
0000: PUSH_CONST 0      // 42
0002: PUSH_CONST 1      // "hello"
0004: ADD
0005: STORE_LOCAL 0     // x
0007: RET
```

**See:** `docs/implementation/12-bytecode-vm.md` for disassembler implementation

---

## Parity Testing

### Requirement

Every program must produce identical output in:
- Interpreter (tree-walking)
- VM (bytecode)

### Test Strategy

1. Write Atlas program
2. Run in interpreter mode
3. Compile to bytecode
4. Run in VM mode
5. Assert outputs match (values, errors, order)

**See:** `docs/guides/testing-guide.md` for parity test patterns

---

## Performance Characteristics

### Bytecode VM (v0.1/v0.2)

- **Faster than interpreter:** ~3-10x speedup
- **Slower than native:** No JIT yet
- **Constant time dispatch:** Jump table for opcodes
- **Stack overhead:** Push/pop for every operation

### Optimization Opportunities

- Peephole optimization: ~10-20% improvement
- Inline caching: ~2-5x for method calls
- JIT compilation: ~10-100x (major effort)

---

## Compiler IR (Future)

### v0.1/v0.2

Direct AST → Bytecode compilation (no IR).

### v0.3+ (Planned)

Introduce intermediate representation (IR) for optimization:

```
AST → High-level IR → Optimized IR → Bytecode
```

**Benefits:**
- Platform-independent optimizations
- Easier to add backends (LLVM, cranelift)
- Better error messages (source mapping through IR)

**See:** `docs/design/compiler-ir.md` for IR design (when added)

---

## Notes

- Bytecode format is unstable in v0.1/v0.2 (subject to change)
- `.atb` files not guaranteed compatible across versions
- Recompile source for each Atlas version
- Cache invalidation automatic (version mismatch detected)
