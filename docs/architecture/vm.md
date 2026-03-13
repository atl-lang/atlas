# Atlas Virtual Machine

The Atlas VM is a stack-based bytecode interpreter. It is the only execution engine — there is no interpreter. All Atlas programs run through the compiler to bytecode and then through this VM.

**Design decision D-052:** Unified execution path. Compiler + VM only.

---

## Architecture Split: VM vs VMContext

The VM is split into two layers (D-057):

### VMContext — Per-Thread Execution State

**Source:** `crates/atlas-runtime/src/vm/context.rs`

`VMContext` is `Clone` and holds everything private to one thread of execution:

```rust
pub struct VMContext {
    stack: Vec<Value>,             // Operand stack, capacity 1024
    frames: Vec<CallFrame>,        // Call frame stack
    ip: usize,                     // Instruction pointer (index into bytecode.instructions)
    defer_stacks: Vec<Vec<(usize, usize)>>, // Per-frame deferred blocks
    string_buffer: String,         // Scratch buffer for string operations (reused)
    struct_type_names: HashMap<usize, String>, // HashMap pointer → struct type name
    debug_pause_pending: bool,
    runtime_warnings: Vec<Diagnostic>,
    // Debug builds only:
    value_origins: Vec<Option<StackValueOrigin>>, // Tracks which local/global produced each stack value
    consumed_slots: Vec<Vec<bool>>,               // Per-frame ownership tracking
    consumed_globals: HashSet<String>,
}
```

Cloning a `VMContext` forks a fresh independent execution context. Workers use this to start isolated from the main VM at startup, then execute tasks independently.

### VM — Shared Resources + Orchestrator

**Source:** `crates/atlas-runtime/src/vm/mod.rs`

```rust
pub struct VM {
    pub ctx: VMContext,             // Per-thread execution state
    globals: HashMap<String, Value>, // Global variable table (shared within one VM instance)
    bytecode: Bytecode,             // Read-only after compilation
    profiler: Option<Profiler>,
    debugger: Option<Debugger>,
    current_security: Option<Arc<SecurityContext>>,
    execution_limits: Option<Arc<ExecutionLimits>>,
    output_writer: OutputWriter,    // Arc<Mutex<Box<dyn Write+Send>>>
    library_loader: LibraryLoader,
    extern_functions: HashMap<String, ExternFunction>,
    jit: Option<Box<dyn JitCompiler>>,
}
```

Worker VMs are created via `VM::new_for_worker()` which clones the bytecode and globals from the main VM. Workers do NOT share globals at runtime — each worker VM has its own `HashMap<String, Value>`. Cross-worker communication uses channels.

---

## Call Frames

**Source:** `crates/atlas-runtime/src/vm/frame.rs`

```rust
pub struct CallFrame {
    pub function_name: String,  // For diagnostics and stack traces
    pub return_ip: usize,       // IP to restore when this frame returns
    pub stack_base: usize,      // Index into ctx.stack where this frame's locals start
    pub local_count: usize,     // Number of locals in this frame
    pub upvalues: Arc<Vec<Value>>, // Captured values for closures (empty for plain functions)
}
```

Stack layout with two frames (main called `add`):

```
ctx.stack:
 index: [  0  ][  1  ][  2  ][  3  ][  4  ]
        [local0][loc1 ][fnptr][arg0 ][arg1 ]
                        ^
                        add frame: stack_base = 2

main frame: stack_base = 0
add frame:  stack_base = 2

GetLocal 0 in main → ctx.stack[0 + 0]
GetLocal 0 in add  → ctx.stack[2 + 0]
```

The initial `<main>` frame is created with `stack_base = 0` and `local_count = bytecode.top_level_local_count`. The defer stack has one entry per frame: `ctx.defer_stacks[frame_idx]`.

---

## Dispatch Loop

**Source:** `crates/atlas-runtime/src/vm/dispatch.rs` + `vm/mod.rs`

Dispatch uses a static `OPCODE_TABLE: [Option<Opcode>; 256]` for O(1) lookup indexed by raw opcode byte — avoids the branch-predictor overhead of a `match` on 256 arms.

```rust
static OPCODE_TABLE: [Option<Opcode>; 256] = { /* compile-time init */ };

// Inner execute loop (simplified):
loop {
    let byte = bytecode.instructions[ip];
    ip += 1;
    let opcode = OPCODE_TABLE[byte as usize].ok_or(/* unknown opcode error */)?;
    match opcode {
        Opcode::Add => { let b = stack.pop(); let a = stack.pop(); stack.push(a + b); }
        Opcode::Call => { /* push call frame, jump to function body */ }
        Opcode::Return => { /* pop frame, restore ip */ }
        Opcode::Halt => break,
        // ...
    }
}
```

The loop is monolithic by design (ARCH-EXCEPTION on file size). Splitting it would add function call overhead on every instruction.

---

## Value Representation

**Source:** `crates/atlas-runtime/src/value.rs`

The `Value` enum is the universal runtime type. All stack values are `Value`:

```rust
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(Arc<String>),          // Immutable, ref-counted
    Array(Arc<Vec<Value>>),       // CoW via Arc::make_mut
    HashMap(Arc<AtlasHashMap>),   // CoW via Arc::make_mut
    HashSet(Arc<HashSet<Value>>), // CoW via Arc::make_mut
    Tuple(Arc<Vec<Value>>),       // Immutable first-class tuple
    Function(FunctionRef),        // Compiled function reference
    Closure(Arc<ClosureRef>),     // Function + captured upvalues
    Future(AtlasFuture),          // Async result (pending/resolved/rejected)
    TaskHandle(Arc<TaskHandle>),  // Spawned task handle
    Option(Option<Box<Value>>),   // Some(v) / None
    Result(Result<Box<Value>, Box<Value>>), // Ok(v) / Err(e)
    EnumValue { enum_name, variant_name, data: Arc<Vec<Value>> },
    ProcessOutput(Arc<ProcessOutput>),
    // ...
}
```

`FunctionRef` carries:
```rust
pub struct FunctionRef {
    pub name: String,
    pub arity: usize,
    pub required_arity: usize,     // arity - (params with defaults)
    pub bytecode_offset: usize,    // Absolute offset in bytecode.instructions
    pub local_count: usize,        // Total locals (including params)
    pub param_names: Vec<String>,
    pub defaults: Vec<Option<Value>>,
    pub param_ownership: Vec<Ownership>,
    pub return_ownership: Option<Ownership>,
    pub is_async: bool,
    pub has_rest_param: bool,
}
```

`ClosureRef` wraps a `FunctionRef` plus `upvalues: Vec<Value>`.

All collections (`Array`, `HashMap`, `HashSet`) use Copy-on-Write semantics via `Arc::make_mut`. The CoW write-back pattern: collection mutation builtins return an updated collection, and the VM writes it back to the caller's variable (see runtime.md patterns).

`Value` is `Send` — enforced by a compile-time assertion in `async_runtime/mod.rs`. This is required for values to cross thread boundaries in the worker pool.

---

## Bytecode Format

**Source:** `crates/atlas-runtime/src/bytecode/opcode.rs`

Stack-based, 67 opcodes. Each opcode is one `u8`. Operands are encoded inline after the opcode byte in big-endian order.

### Opcode Reference

#### Constants (0x01–0x04)
| Opcode | Byte | Operands | Description |
|--------|------|----------|-------------|
| `Constant` | 0x01 | u16 idx | Push constants[idx] |
| `Null` | 0x02 | — | Push null |
| `True` | 0x03 | — | Push true |
| `False` | 0x04 | — | Push false |

#### Variables (0x10–0x16)
| Opcode | Byte | Operands | Description |
|--------|------|----------|-------------|
| `GetLocal` | 0x10 | u16 idx | Push stack[stack_base + idx] |
| `SetLocal` | 0x11 | u16 idx | Pop → stack[stack_base + idx] |
| `GetGlobal` | 0x12 | u16 name_idx | Push globals[constants[name_idx]] |
| `SetGlobal` | 0x13 | u16 name_idx | Pop → globals[constants[name_idx]] |
| `MakeClosure` | 0x14 | u16 func_idx, u16 n_upvalues | Pop n_upvalues, create Closure |
| `GetUpvalue` | 0x15 | u16 idx | Push frame.upvalues[idx] |
| `SetUpvalue` | 0x16 | u16 idx | Pop → frame.upvalues[idx] |

#### Arithmetic (0x20–0x25)
`Add`, `Sub`, `Mul`, `Div`, `Mod` — pop b, pop a, push result. `Negate` — pop a, push -a. Arithmetic checks for NaN/Infinity.

#### Comparison (0x30–0x35)
`Equal`, `NotEqual`, `Less`, `LessEqual`, `Greater`, `GreaterEqual` — pop b, pop a, push bool.

#### Logical (0x40–0x42)
`Not`, `And` (short-circuit skip-next-if-false), `Or` (short-circuit skip-next-if-true).

#### Control Flow (0x50–0x52)
| Opcode | Byte | Operands | Description |
|--------|------|----------|-------------|
| `Jump` | 0x50 | i16 offset | ip += offset |
| `JumpIfFalse` | 0x51 | i16 offset | Pop, if false: ip += offset |
| `Loop` | 0x52 | i16 offset | ip += offset (backward jump) |

#### Functions (0x60–0x62)
| Opcode | Byte | Operands | Description |
|--------|------|----------|-------------|
| `Call` | 0x60 | u8 arg_count | Pop func + args, push call frame |
| `Return` | 0x61 | — | Pop value, restore frame, push return value |
| `TraitDispatch` | 0x62 | u16 trait_idx, u16 method_idx, u8 arg_count | Dynamic trait method dispatch |

#### Collections (0x70–0x7D)
| Opcode | Byte | Operands | Description |
|--------|------|----------|-------------|
| `Array` | 0x70 | u16 count | Pop count values, push Array |
| `GetIndex` | 0x71 | — | Pop index, pop array, push array[index] |
| `SetIndex` | 0x72 | — | Pop value, pop index, pop array, push mutated array |
| `HashMap` | 0x73 | u16 pair_count | Pop key-value pairs, push HashMap |
| `Slice` | 0x74 | — | Pop end, pop start, pop array, push slice |
| `SliceFrom` | 0x75 | — | Pop start, pop array, push array[start..] |
| `SliceTo` | 0x76 | — | Pop end, pop array, push array[..end] |
| `SliceFull` | 0x77 | — | Pop array, push array[..] (clone) |
| `GetField` | 0x78 | — | Pop key, pop map/struct, push value |
| `SetField` | 0x79 | — | Pop value, pop key, pop map, push mutated map |
| `Range` | 0x7A | — | Pop end, pop start, push Range value |
| `Struct` | 0x7B | u16 name_idx, u16 field_count | Pop key-value pairs, push named Struct |
| `Tuple` | 0x7C | u16 count | Pop count values, push Tuple |
| `TupleGet` | 0x7D | u16 idx | Pop Tuple, push tuple[idx] |

#### Stack (0x80–0x84)
`Pop`, `Dup` (duplicate TOS), `Dup2` ([a,b] → [a,b,a,b]), `Rot3` ([a,b,c] → [b,c,a]), `ToString` (convert TOS to string).

#### Pattern Matching (0x90–0x9C)
`IsOptionSome`, `IsOptionNone`, `IsResultOk`, `IsResultErr`, `ExtractOptionValue`, `ExtractResultValue`, `IsArray`, `GetArrayLen`, `EnumVariant` (u8 arg_count), `CheckEnumVariant`, `ExtractEnumData`, `IsStruct`, `CheckStructType` (u16 name_idx).

#### Async (0xA0–0xA3)
| Opcode | Byte | Operands | Description |
|--------|------|----------|-------------|
| `AsyncCall` | 0xA0 | u16 fn_idx, u8 arg_count | Invoke async fn, push Future |
| `Await` | 0xA1 | — | Pop Future, block until resolved, push inner value |
| `WrapFuture` | 0xA2 | — | Pop value, push resolved Future(value) |
| `SpawnTask` | 0xA3 | u16 fn_idx, u8 arg_count | Spawn on worker pool, push Future handle |

#### Defer (0xB0–0xB1)
| Opcode | Byte | Operands | Description |
|--------|------|----------|-------------|
| `DeferPush` | 0xB0 | u16 jump_offset | Register deferred block; jump_offset points past body |
| `DeferExec` | 0xB1 | — | Execute all deferred blocks for current frame (LIFO) |

#### Special
| Opcode | Byte | Description |
|--------|------|-------------|
| `Halt` | 0xFF | End execution |

---

## Function Call Mechanism

When `Call(arg_count)` executes:
1. Pop `arg_count` args from stack (in order).
2. Pop the function value from stack.
3. For a `Function(ref)`: push a new `CallFrame` with `stack_base = stack.len()`, `return_ip = ip + 1`, `local_count = ref.local_count`, `upvalues = empty`.
4. Reserve local slots: push `local_count - arg_count` Null values onto stack to make room for locals beyond the parameters.
5. Set `ip = ref.bytecode_offset`.

When `Return` executes:
1. Pop the return value.
2. Truncate the stack to `frame.stack_base` (discards all locals).
3. Restore `ip = frame.return_ip`.
4. Pop the `CallFrame`.
5. Push the return value.

For `Closure`, the same mechanism applies but the `CallFrame.upvalues` is set to the closure's captured upvalue array.

### Default Parameters

`required_arity` and `defaults` in `FunctionRef` enable optional parameters. On call: if `arg_count < arity`, the VM fills missing arguments from `defaults` (pre-evaluated at compile time via `eval_const_expr`). Rest parameters (`has_rest_param = true`) collect trailing arguments into an Array.

### Trait Dispatch

`TraitDispatch(trait_name_idx, method_name_idx, arg_count)` resolves the correct impl at runtime:
- Peek at the receiver (TOS - arg_count).
- Look up the receiver's type name.
- Construct mangled name: `__impl__{TypeName}__{TraitName}__{MethodName}`.
- Retrieve from globals and call.

---

## Stdlib Dispatch

Stdlib functions are registered as globals before execution. When the VM sees `GetGlobal("math.sqrt")` + `Call(1)`, it pops the function value (a native Rust function pointer wrapped as a Value variant) and dispatches to the Rust implementation.

The stdlib lives in `crates/atlas-runtime/src/stdlib/` — 23 modules, 513 dispatch entries (as of B35). All stdlib calls go through `GetGlobal` + `Call` or `TraitDispatch` — there is no special "native call" opcode. Native functions are `Value::NativeFunction(fn_ptr)` resolved through the same globals table.

---

## Async Execution Integration

`Await` blocks the current execution context until the `AtlasFuture` resolves. Internally:
- If the future is already resolved/rejected, the value is pushed immediately.
- If pending, execution blocks via `block_on(future)` on the Tokio runtime.

`SpawnTask` dispatches to the worker pool (`async_runtime::worker_pool()`). The task runs on a worker's `LocalSet`, and the spawning code gets back a `Value::Future` handle. The caller typically follows with `Await` to retrieve the result.

See `concurrency/CONCURRENCY_INDEX.md` for full concurrency architecture.

---

## Profiler Integration

**Source:** `crates/atlas-runtime/src/profiler/` (re-exported as `vm::Profiler`)

`VM::with_profiling()` enables the profiler. The profiler tracks per-function call counts and execution time. The JIT engine uses these counts for hotspot detection — see `jit.md`.

---

## Debugger Integration

**Source:** `crates/atlas-runtime/src/vm/debugger.rs`

`VM::with_debugging()` enables the debugger. `VM::run_debuggable()` returns `VmRunResult::Paused { ip }` when a breakpoint or step condition is met. The VM state is fully preserved so execution can resume.

`DebugHook` is a trait that callers implement to receive step/breakpoint events. `DebugAction` controls whether to continue, step, or abort.

---

## Security Context

The VM checks `current_security: Option<Arc<SecurityContext>>` on operations that may be restricted (file I/O, network, process exec). `ExecutionLimits` (in `api/config.rs`) can cap execution time and stack depth.

---

## Module Merging at Runtime

When loading multi-module programs via `VM::load_module(new_bc)`, the VM merges the new module's bytecode into the existing combined stream. Instruction pointer and frame state reset to the new module's entry point while globals accumulated from prior modules persist. See `compiler-pipeline.md` Stage "Multi-Module Compilation" for details.
