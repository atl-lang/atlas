# Atlas Compiler Pipeline

**Atlas execution path: source text → Compiler → Bytecode → VM → result.**
There is no interpreter. Every Atlas program runs through this exact sequence.

---

## Pipeline Overview

```
Source Text (.atlas / .atl)
        │
        ▼
    ┌─────────┐
    │  Lexer  │  source → Vec<Token>  (atlas-runtime/src/lexer/)
    └─────────┘
        │
        ▼
    ┌─────────┐
    │ Parser  │  tokens → Program (AST)  (atlas-runtime/src/parser/)
    └─────────┘
        │
        ▼
    ┌──────────────┐
    │ Binder       │  name resolution  (atlas-runtime/src/binder.rs)
    └──────────────┘
        │
        ▼
    ┌─────────────┐
    │ TypeChecker │  type resolution + inference  (atlas-runtime/src/typechecker/)
    └─────────────┘
        │
        ▼
    ┌──────────────┐
    │   Compiler   │  AST → Bytecode  (atlas-runtime/src/compiler/)
    │  (mod.rs,    │
    │  expr.rs,    │
    │  stmt.rs)    │
    └──────────────┘
        │
        ▼
    ┌──────────────┐
    │  Optimizer   │  Optional: constant folding, DCE, peephole
    └──────────────┘  (atlas-runtime/src/optimizer/)
        │
        ▼
    ┌──────────────┐
    │   Bytecode   │  flat instruction stream + constant pool
    └──────────────┘  (atlas-runtime/src/bytecode/)
        │
        ▼
    ┌──────────┐
    │    VM    │  bytecode execution  (atlas-runtime/src/vm/)
    └──────────┘
        │
        ▼
      Value
```

---

## Stage 1: Lexer

**Source:** `crates/atlas-runtime/src/lexer/mod.rs` + `lexer/literals.rs`

The `Lexer` struct converts a source string into a flat `Vec<Token>`. Key state:

- `source: String` — original source
- `chars: Vec<char>` — pre-expanded character array for O(1) indexed access
- `current: usize` — character position
- `line: u32`, `column: u32` — tracked for span accuracy
- Interpolation context stack for nested string interpolation (`InterpolationContext`)

Tokens carry `Span` (file ID, byte offset, line, column) produced from `crate::span`. The file ID is interned via `intern_file()` so spans are cheap to copy.

Error tokens produce `Diagnostic` values with AT-prefixed error codes (e.g. `UNTERMINATED_STRING`, `INVALID_ESCAPE`, `INVALID_NUMBER`).

---

## Stage 2: Parser

**Source:** `crates/atlas-runtime/src/parser/mod.rs`

Recursive-descent parser. Consumes tokens and produces a `Program` (AST root from `crates/atlas-runtime/src/ast.rs`).

Key AST types:
- `Program { items: Vec<Item> }` — top-level items
- `Item` — `Function | Statement | Import | Export | Trait | Impl | Struct | Enum | Const | TypeAlias | Extern`
- `FunctionDecl { name, params: Vec<Param>, return_type: Option<TypeRef>, body: Block, is_async: bool, span }`
- `Param { name, type_ref, ownership, mutable, default_value, is_rest, span }`
- `Stmt` — `VarDecl | Assign | If | While | For | ForIn | Return | Block | Expr | LetDestructure | ...`
- `Expr` — `Literal | Identifier | Binary | Unary | Call | MethodCall | Index | Field | ...`

---

## Stage 3: Binder

**Source:** `crates/atlas-runtime/src/binder.rs`

Name resolution pass. Resolves imports to their source module paths via `ModuleRegistry`. Identifies which names are in scope at each point. The binder also resolves `import` declarations to file paths and populates the `ModuleRegistry` used by the compiler for cross-module symbol access.

---

## Stage 4: Type Checker

**Source:** `crates/atlas-runtime/src/typechecker/`

Key files:
- `mod.rs` — top-level `check_function` (starts ~line 876)
- `expr.rs` — call-site type checking
- `inference.rs` — `infer_return_type(body) -> InferredReturn` for optional return annotations
- `generics.rs` — `Monomorphizer` for generic function instantiation

The type checker validates types and reports `Diagnostic` errors. It does **not** transform the AST.

---

## Stage 5: Compiler (AST → Bytecode)

**Source:** `crates/atlas-runtime/src/compiler/`

The `Compiler` struct is the core. It walks the AST and emits opcodes into a `Bytecode` object. Architecture:

### Compiler State

```rust
pub struct Compiler {
    bytecode: Bytecode,
    locals: Vec<Local>,         // Stack slots: name, scope depth, mutability
    scope_depth: usize,
    loops: Vec<LoopContext>,    // break/continue jump patch targets
    optimizer: Option<Optimizer>,
    monomorphizer: Monomorphizer,
    current_function_base: usize, // Locals index base for current function (nested function support)
    locals_watermark: usize,    // High-water mark for accurate local_count calculation
    upvalue_stack: Vec<UpvalueContext>, // Per-nesting-level closure capture tracking
    global_mutability: HashMap<String, bool>,
    trait_default_methods: HashMap<(String, String), TraitMethodSig>,
    async_fn_names: HashSet<String>, // Functions declared async → emit AsyncCall at call sites
    in_async_fn: bool,           // Inside async fn body → emit WrapFuture on return
    enum_variants: HashMap<String, (String, usize)>, // variant_name → (enum_name, arity)
    const_values: HashMap<String, Value>, // Compile-time constant inlining
    constructor_types: HashSet<String>, // Struct types with static `new` → Foo(args) sugar
}
```

### Entry Point

```rust
pub fn compile(&mut self, program: &Program) -> Result<Bytecode, Vec<Diagnostic>>
```

Pre-pass: collects trait default methods and registers const values for inlining. Then iterates `program.items` calling `compile_item`.

After all items: if a zero-arg `main` is present, emits `GetGlobal("main")` + `Call(0)`. Always terminates with `Halt`.

### Function Compilation Pattern

Functions compile inline — the function body bytecode is interleaved in the single flat instruction stream. The pattern:

```
Constant(placeholder_FunctionRef)  ← adds FunctionRef to constant pool with offset=0
SetGlobal("fn_name")               ← registers function as a global
Pop
Jump → [past body]                 ← skip body during initialization
[function body bytecode here]
Return
[patch Jump target here]
```

The `FunctionRef.bytecode_offset` in the constant pool is patched after the body is emitted with the actual start offset. `local_count` is derived from `locals_watermark` (not `locals.len()`, which may be truncated by match arm cleanup).

### Local Variable Resolution

Locals are indexed by position in `self.locals`. `GetLocal(n)` in bytecode is frame-relative (index from `stack_base`). The compiler resolves names via `resolve_local()` which searches `self.locals` in reverse (newest-first, for shadowing). Globals are resolved by name string stored in the constant pool.

### Upvalue / Closure Capture

Closures capture outer locals via `upvalue_stack: Vec<UpvalueContext>`. Each nesting level gets a context. Captures are either:
- `UpvalueCapture::Local(abs_idx)` — variable is in immediate parent's locals
- `UpvalueCapture::Upvalue(parent_idx)` — multi-level: already registered in parent's upvalue list

At the closure definition site, `MakeClosure(func_const_idx, n_upvalues)` is emitted. The VM pops `n_upvalues` values from the stack and stores them in the closure's upvalue array.

### Async Function Compilation

- At function declaration: if `is_async`, name is added to `async_fn_names` and `FunctionRef.is_async = true`.
- At call sites: `async_fn_names` membership → emit `AsyncCall` instead of `Call`.
- Inside async function body: `in_async_fn = true` → every return path emits `WrapFuture` before `Return`.
- `task.spawn()` calls in stdlib emit `SpawnTask`.

### impl Block Methods (Name Mangling)

Methods compile as top-level functions with mangled names:
- Instance methods: `__impl__{TypeName}__{TraitName}__{MethodName}`
- Static methods: `__static__{TypeName}__{MethodName}`

Trait default methods not overridden by an impl block are automatically compiled for each implementing type.

### Drop Emission

Locals with `drop_type: Some(type_name)` get drop calls emitted at scope exit (B37-P02). Order is LIFO. Drop calls emit:
```
GetLocal(rel_idx)
GetGlobal("__impl__TypeName__Drop__drop")
Call(1)
Pop
```

### Const Inlining

`const` declarations are folded at compile time via `eval_const_expr`. Supported: number/string/bool literals, unary negation/not, binary arithmetic, references to other consts. No bytecode is emitted for const declarations — usages inline the value directly as `Constant(idx)`.

---

## Stage 6: Optimizer (Optional)

**Source:** `crates/atlas-runtime/src/optimizer/`

Three passes applied when `Compiler::with_optimization()` is used:

1. **ConstantFoldingPass** — folds constant arithmetic at compile time
2. **DeadCodeEliminationPass** — removes unreachable code after unconditional jumps/returns
3. **PeepholePass** — local instruction pattern rewrites (e.g. `Push + Pop` → nothing)

Optimizer is disabled by default. Enabled via `Compiler::with_optimization()` or `set_optimizer(Some(...))`.

---

## Stage 7: Bytecode Format

**Source:** `crates/atlas-runtime/src/bytecode/`

`Bytecode` struct:
```rust
pub struct Bytecode {
    pub instructions: Vec<u8>,         // flat instruction stream
    pub constants: Vec<Value>,         // constant pool (strings, numbers, FunctionRefs, ...)
    pub debug_info: Vec<DebugInfo>,    // instruction_offset → Span mapping
    pub top_level_local_count: usize,  // max locals seen at top level
}
```

Opcodes are `u8` values. Operands are encoded inline immediately after the opcode byte in big-endian order. Types: `u8`, `u16` (big-endian), `i16` (signed, for jump offsets).

See `vm.md` for the full opcode reference.

---

## Multi-Module Compilation

When a program imports other modules, `Runtime::eval_file()` uses `ModuleLoader` to collect all modules in dependency order. Each module is compiled independently to its own `Bytecode`, then loaded into the VM sequentially via `VM::load_module()`.

`load_module` **merges** the new module's bytecode into the existing combined instruction stream:
1. Adjusts all `FunctionRef.bytecode_offset` values in the new module's constants by `instr_base` (offset of where new instructions start in the combined stream).
2. Adjusts all constant pool index operands in the new module's instructions by `const_base`.
3. Merges debug info with shifted offsets.
4. Appends adjusted instructions.
5. Resets execution state to start at the new module's entry point.

This ensures cross-module function calls work: a function defined in module A has its `bytecode_offset` valid in the merged stream when module B calls it.

---

## Error Propagation

All compiler stages return `Result<T, Vec<Diagnostic>>`. Errors accumulate (not fail-fast) where possible so the user sees all errors at once. `Diagnostic` carries:
- Error code (`AT####` / `AW####`)
- Source span
- Human-readable message
- Optional context / help text

Diagnostic codes are registered in `crates/atlas-runtime/src/diagnostic/error_codes.rs`.

---

## Crate Ownership

| Component | Crate | Path |
|-----------|-------|------|
| Lexer, Parser, AST | `atlas-runtime` | `src/lexer/`, `src/parser/`, `src/ast.rs` |
| Binder | `atlas-runtime` | `src/binder.rs` |
| Type Checker | `atlas-runtime` | `src/typechecker/` |
| Compiler | `atlas-runtime` | `src/compiler/` |
| Optimizer | `atlas-runtime` | `src/optimizer/` |
| Bytecode | `atlas-runtime` | `src/bytecode/` |
| VM | `atlas-runtime` | `src/vm/` |
| JIT | `atlas-jit` | `src/` |
| Async Runtime | `atlas-runtime` | `src/async_runtime/` |
| CLI entry | `atlas-cli` | `src/main.rs` |
