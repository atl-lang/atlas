# atlas-runtime/src/

The core compiler + runtime. 95% of all Atlas work happens here.

## Directory Map

| Path | What it is |
|------|-----------|
| `value.rs` | **Value enum — all runtime types. Touch this = touch everything.** |
| `ast.rs` | AST nodes: `FunctionDecl`, `Param`, `TypeRef`, `Stmt`, `Expr` |
| `token.rs` | `TokenKind` enum + `is_keyword()` + `as_str()` |
| `lexer/mod.rs` | Tokenizer — keyword map, identifier promotion |
| `parser/mod.rs` | AST construction from token stream |
| `typechecker/` | Type resolution, inference, generics, call-site checks |
| `compiler/` | AST → bytecode (`mod.rs`, `expr.rs`, `stmt.rs`) |
| `interpreter/` | Tree-walking eval (`mod.rs`, `expr.rs`, `stmt.rs`) |
| `vm/mod.rs` | Bytecode execution engine — **ARCH-EXCEPTION on file** (execute loop is monolithic by design); intrinsics + inline tests are split candidates, scheduled post-Block-4 |
| `bytecode/` | Opcode definitions, serialization |
| `stdlib/` | 25 modules, 300+ functions |
| `typechecker/mod.rs` | Function type resolution — `params` at line ~202 |
| `typechecker/expr.rs` | Call-site type checking |
| `diagnostic.rs` | Diagnostic registry — add new codes here |
| `binder.rs` | Name resolution pass |
| `resolver/` | Module resolution |
| `security/` | Permission model, sandbox |
| `ffi/` | Foreign function interface |
| `async_runtime/` | Tokio integration, AtlasFuture, channels |
| `debugger/` | Breakpoints, stepping, source mapping |
| `optimizer/` | Constant folding, dead code, peephole |

## Tests

**Location:** `crates/atlas-runtime/tests/`
**Rule: NO new top-level test files for existing domains.** Add to the correct domain file below.
**Size threshold: 3,000 lines → MUST migrate to subdirectory.** See `atlas-architecture.md`.

| Domain | File | Lines (approx) | Status |
|--------|------|----------------|--------|
| Interpreter behavior | `tests/interpreter.rs` | 5,485 | 🔴 Needs subdirectory |
| VM behavior | `tests/vm.rs` | 5,708 | 🔴 Needs subdirectory |
| Type system | `tests/typesystem.rs` | 6,807 | 🔴 Needs subdirectory |
| Stdlib | **`tests/stdlib/`** (migration in progress; `tests/stdlib.rs` pending removal) | ~14,400 | 🔴 Migration pending |
| System | `tests/system.rs` | ~4,000 | 🔴 Needs subdirectory |
| Frontend/parse | `tests/frontend_integration.rs` | 3,166 | 🔴 Needs subdirectory |
| Frontend syntax | `tests/frontend_syntax.rs` | 3,094 | 🔴 Needs subdirectory |
| Collections/CoW | `tests/collections.rs` | ~1,800 | ✅ |
| Pattern matching | `tests/pattern_matching.rs` | ~1,600 | ✅ |
| Closures | `tests/closures.rs` | — | ✅ |
| Async | `tests/async_runtime.rs` | ~2,300 | ✅ |
| Parity tests | Add to the relevant domain file with both engines | — | — |

**Subdirectory migration pattern:**
`tests/stdlib.rs` → `tests/stdlib/mod.rs` + `tests/stdlib/strings.rs` + `tests/stdlib/math.rs` etc.
Do NOT create new top-level `.rs` files for domains already in this table.

## Critical Rules

**Parity is sacred.** Every behavior change must produce identical output in both
interpreter (`interpreter/mod.rs`) and VM (`vm/mod.rs`). If you touch one, you touch both.
Parity break = BLOCKING. Never ship a phase with parity divergence.

**CoW write-back pattern.** Collection mutation builtins return a NEW collection.
The interpreter (`apply_cow_writeback()`) and VM (`emit_cow_writeback_if_needed()`) write
the result back to the caller's variable. Both `let` and `var` bindings can be mutated
this way — it's content mutation, not rebinding. See DR-004 in auto-memory decisions/runtime.md.

**value.rs blast radius.** Adding a new `Value` variant requires updating:
`type_name()`, `Display`, `PartialEq`, equality semantics, bytecode serialization,
interpreter eval, VM execution, all stdlib functions that pattern-match on Value.

## Key Invariants (verified 2026-02-21)

- `ValueArray` = `Arc<Vec<Value>>` — CoW via `Arc::make_mut`
- `ValueHashMap` = `Arc<AtlasHashMap>` — CoW via `Arc::make_mut`
- `Shared<T>` = `Arc<Mutex<T>>` — explicit reference semantics only
- `FunctionRef` at `value.rs:464` — holds arity, bytecode_offset, local_count
- `Param` at `ast.rs:187` — name, type_ref, ownership, span (ownership added Block 2)
- Expression statements require semicolons — `f(x)` without `;` fails to parse
