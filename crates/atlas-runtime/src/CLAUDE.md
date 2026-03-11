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
| `diagnostic.rs` | Diagnostic struct + Display formatting (`to_human_string()`, JSON output) |
| `diagnostic/descriptor.rs` | DiagnosticDescriptor (const per-code metadata), DiagnosticDomain enum, DiagnosticBuilder API (B17-P01) |
| `diagnostic/error_codes.rs` | Error code registry — add new AT/AW codes with description + help + example here |
| `diagnostic/formatter.rs` | Diagnostic formatting utilities (line/col conversion, snippet extraction) |
| `diagnostic/normalizer.rs` | Diagnostic normalization (consolidates cascades, deduplicates) |
| `diagnostic/warnings.rs` | Warning emission (AW codes) and warning-as-error logic |
| `binder.rs` | Name resolution pass |
| `resolver/` | Module resolution |
| `security/` | Permission model, sandbox |
| `ffi/` | Foreign function interface |
| `async_runtime/` | Tokio integration, AtlasFuture, channels |
| `debugger/` | Breakpoints, stepping, source mapping |
| `optimizer/` | Constant folding, dead code, peephole |
| `api/` | Native binding API (`mod.rs`, `conversion.rs`, `native.rs`, `runtime.rs`, `config.rs`) |
| `profiler/` | Runtime profiling — hotspot detection, perf report, collector |
| `reflect/` | Type reflection API — `type_info.rs`, `value_info.rs` |
| `sourcemap/` | Source map encoding/decoding for debugging (`encoder.rs`, `vlq.rs`) |
| `stack_trace.rs` | Stack trace formatting and inspection |
| `span.rs` | Source code span tracking |
| `symbol.rs` | Symbol table and name management |
| `runtime.rs` | Runtime manager struct + lifecycle |
| `repl.rs` | Read-eval-print loop implementation |
| `method_dispatch.rs` | Method call dispatch mechanism |
| `module_executor.rs` | Module execution coordination |
| `module_loader.rs` | Module discovery and loading |
| `jit_trait.rs` | Trait interface for JIT integration |
| `json_value.rs` | JSON value wrapper |
| `test_utils.rs` | Testing utilities for runtime |
| `typecheck_dump.rs` | Type checker debugging output |
| `types.rs` | Type system core structures |

## Tests

**Location:** `crates/atlas-runtime/tests/`
**Rule: NO new top-level test files for existing domains.** Add to the correct submodule file below.

### ⚠️ Size limit: 12KB maximum per test file

Test files are token-dense. An agent reading a large test file burns significant tokens before adding a line.
**Before adding tests to any file, check its size:**
```bash
du -sh crates/atlas-runtime/tests/<target-file>.rs
```
- **> 12KB:** BLOCKING — split the file first, then add tests
- **10–12KB:** Acceptable — monitor for future split if it grows
- **Target: ~10KB per file**

**Check all file sizes:** `find crates/atlas-runtime/tests -name "*.rs" -not -path "*/target/*" | xargs du -sh | sort -rh`

### Subdirectory-split domains (ADD TESTS TO THE SUBDIR FILES, NOT THE ROUTER)

The `.rs` root files for these domains are **thin routers** (66–201 lines). Opening them and adding tests there is wrong. Go to the subdirectory.

| Domain | Add tests to... |
|--------|----------------|
| Stdlib | `tests/stdlib/` → strings, json, io, types, collections, parity, integration, docs_verification, array_intrinsics, array_pure, math_basic, math_trig, math_utils_constants |
| Type system | `tests/typesystem/` → inference, constraints, flow, generics, bindings, integration |
| VM behavior | `tests/vm/` → integration, member, complex_programs, regression, regression_loops, performance, functions, functions_loops, nested, for_in, array_intrinsics, array_pure, math_basic, math_trig, math_utils_constants, async_vm, error_handling, logical, opcodes |
| Interpreter | `tests/interpreter/` → member, nested_functions, nested_functions_loops, scope, pattern_matching, assignment, integration |
| System/stdlib-fs | `tests/system/` → path, filesystem, process, compression |
| Frontend syntax | `tests/frontend_syntax/` → lexer, parser_basics, parser_errors, parser_errors_part2, parser_control_flow, parser_anonymous_structs, parser_ranges, operator_precedence_keywords, generics, modules_warnings_part1, warnings_part2, warnings_attributes, for_in_traits_part1, traits_part2, diagnostic_descriptor |
| Frontend integration | `tests/frontend_integration/` → integration_part_{1-5}, ast_part_{1-2}, bytecode_validator, ownership, traits, anonfn_part_{1-2} |

**How to pick the right file:** match the feature domain (e.g., new string builtin → `tests/stdlib/strings.rs`).

### Single-file domains (add directly)

| Domain | File |
|--------|------|
| Collections/CoW | `tests/collections.rs` |
| Pattern matching | `tests/pattern_matching.rs` |
| Closures | `tests/closures.rs` |
| Async | `tests/async_runtime.rs` |
| Regression | `tests/regression.rs` |

## Critical Rules

**Parity is sacred.** Every behavior change must produce identical output in both
interpreter (`interpreter/mod.rs`) and VM (`vm/mod.rs`). If you touch one, you touch both.
Parity break = BLOCKING. Never ship a phase with parity divergence.

**CoW write-back pattern.** Collection mutation builtins return an updated collection,
and the interpreter (`apply_cow_writeback()`) and VM (`emit_cow_writeback_if_needed()`)
write it back to the caller's variable. **All collections are CoW** — HashMap, Array,
HashSet, Queue, Stack. Both `let` and `var` bindings can be mutated this way — it's
content mutation, not rebinding. See `.claude/memory/patterns/runtime.md`.

**value.rs blast radius.** Adding a new `Value` variant requires updating:
`type_name()`, `Display`, `PartialEq`, equality semantics, bytecode serialization,
interpreter eval, VM execution, all stdlib functions that pattern-match on Value.

## Key Invariants (verified 2026-03-06)

- `Value::Tuple(Arc<Vec<Value>>)` — immutable first-class tuple; Display trails comma for 1-tuples: `(x,)` (B15)
- `Value::ProcessOutput(Arc<ProcessOutput>)` — typed result of `Process.exec()` / `Process.shell()`; fields: `stdout: String`, `stderr: String`, `exit_code: i32`, `success: bool`; methods: `.stdout()`, `.stderr()`, `.exitCode()`, `.success()` (B18)
- `LetDestructure` at `ast.rs` — `let (a, b) = expr;` destructuring statement (B15); `Pattern::Tuple` for match arms
- `TypeRef::Tuple` — tuple type annotation `(T1, T2)` (B15)
- `ValueArray` = `Arc<Vec<Value>>` — CoW via `Arc::make_mut`
- `ValueHashMap` = `Arc<AtlasHashMap>` — CoW via `Arc::make_mut` (same pattern as `ValueArray`)
- `Shared<T>` = `Arc<Mutex<T>>` — explicit reference semantics only
- `FunctionRef` at `value.rs:548` — holds arity, bytecode_offset, local_count
- `Param` at `ast.rs:405` — name, type_ref, ownership, ownership_explicit, mutable, span (mutable added H-089, ownership added Block 2; ownership_explicit added H-209 — true only when own/borrow/share written in source, false for bare params that default to borrow per D-040)
- `FunctionDecl.return_type: Option<TypeRef>` — `None` means inferred (Block 5); `infer_return_type()` in `typechecker/inference.rs`
- AT3050 (inconsistent returns), AT3051 (uninferrable type param), AT3052 (inferred type incompatible) — registered in `diagnostic/error_codes.rs`
- Expression statements require semicolons — `f(x)` without `;` fails to parse
