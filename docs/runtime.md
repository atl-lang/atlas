# Atlas Runtime Specification

**Purpose:** Comprehensive runtime specification including value model, memory management, bytecode format, standard library, and prelude

**For AI Agents:** This is your single source of truth for runtime behavior. Read this to understand value representation, execution semantics, bytecode format, and built-in functions.

---

## Table of Contents
1. [Value Model](#value-model)
2. [Memory Management](#memory-management)
3. [Bytecode Format](#bytecode-format)
4. [Prelude (Built-in Functions)](#prelude-built-in-functions)
5. [Standard Library](#standard-library)
6. [Execution Model](#execution-model)

---

## Value Model

### Single Value Type

**Design:** Atlas uses a single `Value` enum shared by interpreter and VM.

**Rust Type Definition:**
```rust
pub enum Value {
    Number(f64),
    String(Rc<String>),
    Bool(bool),
    Null,
    Array(Rc<RefCell<Vec<Value>>>),
    Function(FunctionRef),
}
```

### Value Types

**Immediate Values** (stack-allocated):
- `Number(f64)` - 64-bit IEEE 754 floating-point
- `Bool(bool)` - true or false
- `Null` - null value

**Heap Values** (reference-counted):
- `String(Rc<String>)` - Immutable UTF-8 strings
- `Array(Rc<RefCell<Vec<Value>>>)` - Mutable arrays
- `Function(FunctionRef)` - Function references

### Ownership Rules

**Numbers, Bools, Null:**
- Stored directly in `Value` enum
- Copied on assignment
- No heap allocation

**Strings:**
- Heap-allocated, reference-counted (`Rc`)
- Immutable once created
- Assignment copies reference, not string content
- Shared across multiple `Value` instances

**Arrays:**
- Heap-allocated, reference-counted (`Rc`)
- Mutable through `RefCell`
- Assignment copies reference, not array content
- Shared across multiple `Value` instances
- Mutation visible to all aliases

**Functions:**
- Reference to function definition
- Immutable
- Copied by reference

### Mutability Rules

**Immutable Types:**
- Strings are always immutable
- Functions are always immutable
- Numbers, bools, null are immutable values

**Mutable Types:**
- Arrays are mutable (content can change)
- Array references are copied, but content is shared
- Example:
  ```atlas
  let a = [1, 2, 3];
  let b = a;        // b is alias to same array
  a[0] = 99;        // Mutate through a
  // b[0] is now 99  // Visible through b
  ```

### Equality Semantics

**Value Equality:**
- Numbers: IEEE 754 equality (NaN != NaN, +0.0 == -0.0)
- Bools: true == true, false == false
- Null: null == null
- Strings: Compare by content (Unicode scalar equality)

**Reference Equality:**
- Arrays: Compare by reference identity (not content)
- Functions: Compare by reference identity
- Example:
  ```atlas
  [1, 2] == [1, 2]    // false (different array objects)
  let a = [1, 2];
  let b = a;
  a == b              // true (same reference)
  ```

**Type Safety:**
- `==` and `!=` require same-type operands
- Different types produce compile-time error `AT0001`

---

## Memory Management

### Reference Counting

**Current Implementation:**
- Uses Rust's `Rc` (reference-counted pointers)
- No garbage collector
- Deterministic deallocation

**Benefits:**
- Predictable memory behavior
- No GC pauses
- Simple implementation

**Limitations:**
- Cannot handle cyclic references
- Reference count overhead

### Future Considerations

**Under Research:**
- Garbage collection for cyclic data structures
- Arena allocation for batch deallocation
- Generational GC for performance

**Current Status:**
- Reference counting is sufficient for current use cases
- No cyclic data structures supported yet
- Will revisit if performance becomes bottleneck

---

## Bytecode Format

### File Extension
`.atb` (Atlas Bytecode)

### Bytecode File Structure

**Header:**
```
- Magic: "ATB1" (4 bytes)
- Version: u16 (bytecode format version)
- Flags: u16 (bitflags, 0 for current implementation)
```

**Sections (in order):**

1. **Constants Pool**
   ```
   - Count: u32
   - Each constant:
     - Type tag: u8
       - 0x01: Number
       - 0x02: String
     - Payload:
       - Number: f64 (8 bytes, IEEE 754)
       - String: u32 length + UTF-8 bytes
   ```

2. **Code Section**
   ```
   - Instruction count: u32
   - Each instruction:
     - Opcode: u8
     - Operands: fixed or variable by opcode
   ```

3. **Debug Info Section**
   ```
   - Span table count: u32
   - Each span:
     - File index: u32
     - Line: u32
     - Column: u32
     - Length: u32
   - Instruction span mapping:
     - u32 per instruction (index into span table)
   ```

4. **File Table**
   ```
   - File count: u32
   - Each file:
     - u32 length + UTF-8 bytes
   ```

### Bytecode Properties

**Endianness:** Little-endian (x86/ARM standard)

**Version Handling:**
- Version mismatch produces diagnostic
- Forward compatibility not guaranteed
- Bytecode regenerated when format changes

**Debug Info:**
- Present by default
- Maps every instruction to source span
- Enables precise error reporting

**See:** `docs/implementation/11-bytecode.md` for opcode listing and details

---

## Prelude (Built-in Functions)

### Definition
Globally available built-in functions that do not require import. Always in scope.

### Current Prelude Functions

#### `print`
**Signature:** `print(value: string|number|bool|null) -> void`

**Behavior:**
- Writes value to stdout
- Adds newline after value
- `null` prints as `"null"`
- No return value

**Examples:**
```atlas
print("hello");      // Output: hello
print(42);           // Output: 42
print(true);         // Output: true
print(null);         // Output: null
```

**Error Cases:**
- Wrong type: Runtime error `AT0102` (invalid stdlib argument)

#### `len`
**Signature:** `len(value: string|T[]) -> number`

**Behavior:**
- Returns length of string or array
- String length is Unicode scalar count (not bytes)
- Array length is element count
- Returns 0 for empty strings/arrays

**Examples:**
```atlas
len("hello")    // 5
len("🌟")       // 1 (one Unicode scalar)
len([1, 2, 3])  // 3
len([])         // 0
```

**Error Cases:**
- Wrong type: Runtime error `AT0102` (invalid stdlib argument)

#### `str`
**Signature:** `str(value: number|bool|null) -> string`

**Behavior:**
- Converts value to string representation
- Numbers: Decimal representation
- Bools: `"true"` or `"false"`
- Null: `"null"`

**Examples:**
```atlas
str(42)      // "42"
str(3.14)    // "3.14"
str(true)    // "true"
str(null)    // "null"
```

**Error Cases:**
- Wrong type: Runtime error `AT0102` (invalid stdlib argument)

### Prelude Rules

**Scope:**
- Prelude functions are always in scope
- No import or declaration needed
- Available in all files and REPL

**Shadowing:**
- User code MAY shadow prelude names in nested scopes
- User code MAY NOT shadow prelude names in global scope
- Global shadowing produces compile-time error `AT1012`

**Examples:**
```atlas
// OK: Shadowing in nested scope
fn foo() {
    let print = 5;  // OK: shadows prelude in function scope
}

// ERROR: Shadowing in global scope
let print = 5;      // AT1012: cannot shadow prelude name
```

---

## Standard Library

### Current Functions

**See:** [Prelude section](#prelude-built-in-functions) for `print`, `len`, `str`

### Expansion Plans

**Future stdlib additions are planned in v0.2 phases:**
- String manipulation (split, join, trim, etc.)
- Array operations (map, filter, slice, etc.)
- Math functions (abs, floor, ceil, etc.)
- JSON handling (parse, stringify)
- File I/O (read, write)
- More...

**See:** `phases/stdlib/` for detailed expansion plans

### Standard Library Properties

**Purity:**
- All stdlib functions are pure **except `print`**
- Pure functions have no side effects
- Same inputs always produce same outputs

**Error Handling:**
- All stdlib errors include span info pointing to callsite
- Clear error messages
- Consistent error codes

**Type Safety:**
- All stdlib functions have explicit type signatures
- Type checking at compile time
- Runtime type checking for safety

---

## Execution Model

### Interpreter and VM Semantics

**Parity Guarantee:**
- Interpreter and VM produce **identical outputs**
- Same input program yields same stdout
- Same diagnostics and error codes
- Same runtime error locations

**Evaluation Order:**
- Left-to-right evaluation
- Example: `f() + g()` evaluates `f()` then `g()`
- Function arguments evaluated left-to-right

**Function Calls:**
- Arguments evaluated left-to-right
- Arguments passed by value
- Heap types (strings, arrays) pass references

### Runtime Errors

**Error Behavior:**
- Runtime errors produce diagnostics with stack traces
- Diagnostics include source location
- Errors include error code (e.g., `AT0005`, `AT0006`)

**REPL Behavior:**
- Runtime errors do NOT abort REPL
- REPL continues accepting input after error
- Error state is isolated per input

**Examples:**
```atlas
>> 1 / 0
runtime error[AT0005]: Divide by zero
  --> <repl>:1:3

>> print("still works")
still works
```

### Edge Case Handling

**Array Indexing:**
- Requires integer values
- Non-integer indices are runtime errors
- Out-of-bounds access is runtime error `AT0006`

**Numeric Results:**
- NaN results are runtime errors `AT0007`
- Infinity results are runtime errors `AT0007`
- Fail fast, not silent corruption

**See:** `docs/LANGUAGE_SEMANTICS.md` for complete edge case documentation

---

## Runtime Architecture

### Components

**Shared Components:**
- `Value` enum (shared by interpreter and VM)
- Diagnostic system (shared error handling)
- Prelude registry (shared built-in functions)

**Interpreter:**
- Tree-walking interpreter
- Evaluates AST directly
- Maintains call stack
- Variable environment per scope

**VM (Virtual Machine):**
- Stack-based bytecode interpreter
- Executes compiled bytecode
- Faster than tree-walking interpreter
- Maintains call frames

**Integration:**
- Both use same `Value` representation
- Both produce same diagnostics
- Both access same prelude
- Parity verified by tests

---

## Implementation References

**For implementation details, see:**
- `docs/implementation/09-value-model.md` - Rust value implementation
- `docs/implementation/10-interpreter.md` - Interpreter implementation
- `docs/implementation/12-vm.md` - VM implementation
- `docs/implementation/11-bytecode.md` - Bytecode compiler
- `docs/implementation/13-stdlib.md` - Standard library implementation

---

**Summary:** Atlas runtime uses reference-counted values, maintains interpreter/VM parity, provides a minimal but complete prelude, and uses a stable bytecode format with debug information.
