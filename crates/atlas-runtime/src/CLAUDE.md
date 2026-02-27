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
| `typechecker/inference.rs` | `infer_return_type(body) -> InferredReturn` — return type inference for optional annotations |
| `compiler/` | AST → bytecode (`mod.rs`, `expr.rs`, `stmt.rs`) |
| `interpreter/` | Tree-walking eval (`mod.rs`, `expr.rs`, `stmt.rs`) |
| `vm/mod.rs` | Bytecode execution engine — **ARCH-EXCEPTION on file** (execute loop is monolithic by design); intrinsics + inline tests are split candidates, scheduled post-Block-4 |
| `bytecode/` | Opcode definitions, serialization |
| `stdlib/` | 25 modules, 300+ functions |
| `typechecker/mod.rs` | Function type resolution — `check_function` at line ~876 |
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
**Rule: NO new top-level test files for existing domains.** Add to the correct submodule file below.

### Subdirectory-split domains (ADD TESTS TO THE SUBDIR FILES, NOT THE ROUTER)

The `.rs` root files for these domains are **thin routers** (66–201 lines). Opening them and adding tests there is wrong. Go to the subdirectory.

| Domain | Add tests to... |
|--------|----------------|
| Stdlib | `tests/stdlib/` → strings, json, io, types, functions, collections, parity, vm_stdlib, integration, docs_verification, array_intrinsics, array_pure, math_basic, math_trig, math_utils_constants |
| Type system | `tests/typesystem/` → inference, constraints, flow, generics, bindings, integration |
| VM behavior | `tests/vm/` → integration, member, complex_programs, regression, performance, functions, nested, for_in, array_intrinsics, array_pure, math_basic, math_trig, math_utils_constants |
| Interpreter | `tests/interpreter/` → member, nested_functions, scope, pattern_matching, assignment, for_in, integration |
| System/stdlib-fs | `tests/system/` → path, filesystem, process, compression |

**How to pick the right file:** match the feature domain (e.g., new string builtin → `tests/stdlib/strings.rs`).

### Single-file domains (add directly)

| Domain | File |
|--------|------|
| Lexer, parser, syntax | `tests/frontend_syntax.rs` |
| Frontend pipeline | `tests/frontend_integration.rs` |
| Collections/CoW | `tests/collections.rs` |
| Pattern matching | `tests/pattern_matching.rs` |
| Closures | `tests/closures.rs` |
| Async | `tests/async_runtime.rs` |
| Regression | `tests/regression.rs` |

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
- `FunctionDecl.return_type: Option<TypeRef>` — `None` means inferred (Block 5); `infer_return_type()` in `typechecker/inference.rs`
- AT3050 (inconsistent returns), AT3051 (uninferrable type param), AT3052 (inferred type incompatible) — registered in `diagnostic/error_codes.rs`
- Expression statements require semicolons — `f(x)` without `;` fails to parse
