# Atlas Codebase Patterns

**Purpose:** Active patterns for AI reference. Archived stable patterns in `archive/2026-02-patterns-v1.md`.

---

## Runtime API

```rust
let atlas = Atlas::new();
let atlas = Atlas::new_with_security(SecurityContext::allow_all());
let result: RuntimeResult<Value> = atlas.eval("let x = 1;");
```

**Test helpers** (`tests/common/mod.rs`):
```rust
common::assert_eval_number("1 + 2", 3.0);
common::assert_eval_string(r#""hello""#, "hello");
common::assert_error_code("bad code", "AT0001");
common::compile_source("let x = 1;");
common::run_bytecode(bytecode);
```

---

## Collection Types (CoW — Phase 12–15)

All collections use CoW wrappers (Arc, no Mutex):
```rust
Value::Array(ValueArray)          // Arc<Vec<Value>> — .as_slice(), .len(), .push(), .set()
Value::HashMap(ValueHashMap)      // Arc<AtlasHashMap> — .inner(), .inner_mut()
Value::HashSet(ValueHashSet)      // Arc<AtlasHashSet> — .inner(), .inner_mut()
Value::Queue(ValueQueue)          // Arc<VecDeque<Value>>
Value::Stack(ValueStack)          // Arc<Vec<Value>>
```

**NEVER use `.lock().unwrap()`** — Mutex is gone. Use CoW API.

**Mutation builtins return new collection** (or `[extracted, new_col]` for remove/pop/dequeue).
Interpreter and VM automatically write-back to first-arg variable via CoW write-back.

**Array method syntax** (Phase 16):
- `arr.push(x)` → `arrayPush`, mutating (writes back to receiver)
- `arr.pop()` → `arrayPop`, RETURNS_PAIR `[removed, new_arr]`, writes back new_arr, returns removed
- `arr.shift()` → `arrayShift`, same as pop pattern
- `arr.unshift(x)` → `arrayUnshift`, mutating (writes back)
- `arr.reverse()` → `arrayReverse`, mutating (writes back)
- `arr.sort()` → `arraySort`, NON-mutating (returns sorted copy, receiver unchanged)
- TypeTag::Array in method_dispatch.rs; typechecker sets it in check_member
- Dynamic fallback in eval_member when type_tag is None (runtime check on Value::Array)

**HashKey** (not String) for HashMap/HashSet keys:
```rust
HashKey::String(Arc::new("x".to_string()))
HashKey::Number(OrderedFloat(1.0))
```

---

## Stdlib Function Pattern

Register in `stdlib/mod.rs`:
- `is_builtin(name) -> bool`
- `call_builtin(name, args, span, security) -> Result<Value, RuntimeError>`

```rust
fn expect_string(value: &Value, arg_name: &str, span: Span) -> Result<String, RuntimeError> {
    match value {
        Value::String(s) => Ok((**s).clone()),
        _ => Err(RuntimeError::TypeError { msg: "...", span })
    }
}
```

---

## Error Handling

Use struct variants (NOT `::new()`):
```rust
RuntimeError::TypeError { msg: "message".to_string(), span }
```

---

## CLI Patterns

### P-001: Full Permissions by Default — CRITICAL

`atlas run` grants full system permissions. NO permission flags.

```rust
let runtime = Atlas::new_with_security(SecurityContext::allow_all());
```

**Industry standard:** Go, Rust, Python, Node.js all grant full access.
**NOT like Deno:** No `--allow-net`, `--allow-read` flags.

**SecurityContext is for embedding** (running untrusted code in apps), NOT CLI.

**DO NOT CHANGE:** Future AI must NOT add permission flags to CLI.

### P-002: Testing Architecture — Rust/Go Model — ARCHITECTURE

Testing follows `cargo test` / `go test` model:
- **Stdlib:** Assertion primitives only (`assert()`, `assertEqual()`)
- **CLI:** Full test runner (discovery, execution, reporting)

**NO separate testing framework.** Stdlib = assertions, CLI = orchestration.

**DO NOT:** Create duplicate test runners.

---

## Language Core Patterns

> **v0.3 Grammar Rewrite:** Major syntax changes tracked in `atlas-track decisions`.
> Run `atlas-track decisions` for D-006 to D-010. Extended rationale in
> `docs/language-design/rationale/`. Migration guides in `docs/language-design/migration/`.
>
> Changes: Remove `var`, `++/--`, C-style for, arrow functions. Add `record` keyword.

### P-001: Strict Type System with No Implicit Coercion

No implicit type coercion. `"5" + 3` → type error. Use `toNumber()`, `toString()`.

### P-002: Scientific Notation for Number Literals

Support `1.5e10`, `3e-5`. Token-efficient, industry standard.

### P-003: Method Call Syntax — Rust-Style Desugaring

`value.method(args)` desugars to `Type::method(value, args)`. Both syntaxes valid.

### P-004: Prelude with Shadowing Protection

Built-ins always available. Shadowing prelude names → compile error (AT1012).

---

## LSP Patterns

*DRs 001–006 archived to `archive/2026-02-decisions-lsp-v1.md`.*

### P-LSP-007: Workspace Symbol Performance Optimizations

**Context:** Phase 05C - large workspace performance

**Decision:** Three-tier optimization strategy:
1. **LRU Query Cache** — Cache search results (key: query + kind + limit as strings)
2. **Memory Bounds** — Max 100k symbols, evict oldest document when exceeded
3. **Batch Indexing** — Single cache invalidation for multiple documents

**Rationale:**
- SymbolKind doesn't impl Hash → use format!("{:?}") for cache keys
- Program doesn't impl Sync → batch indexing sequential, not parallel
- Prevents OOM on large workspaces, 10-100x speedup on cached queries

**Status:** Implemented with 21 tests (11 workspace search + 10 performance)

### P-B03-03: Partial AST on Parse Errors

**Decision:** `document.rs` stores the partial AST even when parse errors occur.
Previously: returned early with `ast = None` on any parse error.
Now: stores the AST before returning on parse errors, enabling hover/completion for partial code
(critical for `impl |` completions).

**Notes:**
- Testing Pattern: LSP tests use inline server creation (see testing-patterns.md — lifetime issues prevent helper functions)
- Cross-File Support: Phase 05 added workspace-wide symbol search, references, call hierarchy.
- Type Integration: Refactorings don't use type information yet. Future enhancement could enable type-aware refactorings.

---

## Runtime Patterns: Scoping and Performance

### P-003: Dynamic Scoping vs Lexical Closures

Atlas uses **dynamic scoping** in the interpreter and **GetGlobal for cross-scope vars** in VM.

- Interpreter: `get_variable()` walks the live scope stack (innermost→outermost + globals)
- VM: top-level let/var = globals (GetGlobal) ✅; outer function locals = stack-allocated → inner fns emit GetGlobal for them which FAILS at runtime
- Result: inner fns CAN access outer fn's locals in interpreter (dynamic scoping), CANNOT in VM

**Pattern:** Only use top-level let/var for cross-function state sharing (both engines).

**Atlas syntax note:** Expression statements REQUIRE semicolons. `f(41)` without `;` fails to parse. All top-level expressions in tests must end with `;`.

**v0.3 plan (Block 4):** Proper lexical closures with CoW value capture semantics.
Type inference is Block 5 (local variables + return types only — NOT full H-M).

### P-B04-01: Zero-Allocation Interpreter Hot Path — Goal Not Gate (2026-02-23)

**Decision:** The interpreter's eval loop should not allocate on the happy path. This is a documented goal, NOT a CI gate.

**Rationale:** CI-gated allocator checks (custom allocator panic, dhat) would break on every new stdlib function, `format!` in error paths, or innocent `Vec::new()` in utility code. High friction for a constraint that only matters post-v0.3 when Atlas targets embedded/real-time workloads.

**What to do instead:**
- The zero-allocation invariant is documented in `atlas-interpreter.md` as a goal
- Manual spot-check at block completion: `grep -n "Vec::new\|String::new\|format!" interpreter/expr.rs`
- Revisit with `dhat` or allocator gate when performance/embedded blocks ship (post-v0.3)

**Does NOT apply to:** error paths, stdlib functions, diagnostics, debug utilities.

---

## Runtime Patterns: Traits

### P-B03-01: Trait Method Mangled Names (Static Dispatch) — LOCKED (2026-02-22)

Impl methods compile to top-level functions with mangled names:
`__impl__{TypeName}__{TraitName}__{MethodName}`
Example: `impl Display for number` → `fn display` → `__impl__number__Display__display`

- Compiler emits these as regular named functions (no new opcodes)
- `Call` opcode reused — static dispatch, resolved at compile time
- Interpreter uses a `HashMap<(type_name, method_name), ImplMethod>` (no mangling needed)
- The mangled name format is the canonical compile-time key

**Dispatch path:**
1. Typechecker annotates `MemberExpr.trait_dispatch: RefCell<Option<(String, String)>>` via `resolve_trait_method_call_with_info()`
2. Compiler reads annotation → emits `GetGlobal(mangled_name)` + `Call(argc)`
3. Interpreter reads annotation → looks up `function_bodies[mangled_name]` + calls it

**No new opcodes required.** Existing `GetGlobal` + `Call` sufficient.

### P-B03-02: Block 3 Trait Architecture Patterns — LOCKED (2026-02-22)

- **Static dispatch only** — vtable/trait objects = v0.4
- **`Drop` explicit only** — auto scope-exit invocation = v0.4 (scope tracking not ready)
- **`:` vs `extends`** — separate AST fields: `trait_bounds: Vec<TraitBound>` (new) vs `bound: Option<TypeRef>` (existing `extends`)
- **Copy = all built-in value types** — number/string/bool/null/array/map are Copy; user types default to Move
- **`Display` ≠ `str()`** — Display trait and str() stdlib are independent in Block 3; integration in v0.4
- **AT3001–AT3010** — full diagnostic code range for trait violations

### P-B03-02: Trait Registry Architecture

**Decision:** `TraitRegistry` in typechecker/mod.rs stores trait declarations separately from `ImplRegistry`.
- `TraitRegistry.traits: HashMap<String, Vec<TraitMethodRecord>>` — trait name → method records
- `ImplRegistry.entries: HashMap<(String, String), ImplEntry>` — (type_name, trait_name) → methods
- `find_trait_with_method(method_name)` — reverse lookup for AT3035 emission

---

## Stdlib Patterns

### P-001: JsonValue — Controlled Dynamic Typing

`JsonValue` type for dynamic JSON. Exception to strict typing (necessary for AI).

### P-002: Array API — Intrinsics vs Stdlib Split

- **Pure functions:** `stdlib/array.rs` (push, pop, slice)
- **Callback intrinsics:** Interpreter/VM direct (map, filter, forEach)

### P-003: Hash Function Design

Deterministic hashing via `DefaultHasher`.
- Hashable: number, string, bool, null
- Not hashable: array, function, JsonValue → error AT0140

### P-004: Collection Value Representation

Collections use `Arc<Mutex<X>>`:
```rust
Value::HashMap(Arc<Mutex<AtlasHashMap>>)
Value::HashSet(Arc<Mutex<AtlasHashSet>>)
```

### P-005: Collection API Design

Function-based API: `hashMapPut(map, key, value)` not `map.put()`.
Explicit type names, callback-based iteration.

### P-006: HashMap Stdlib Architecture

Use `stdlib/mod.rs` pattern. Register in `is_builtin()`, implement in `call_builtin()`.

---

## Type System Patterns

### P-001: Monomorphization for Generic Types

Rust-style monomorphization. `Option<Number>` and `Option<String>` are separate types.

### P-002: TypeChecker-Owned Usage Tracking

No `used` field on Symbol struct. TypeChecker owns usage tracking separately.

---

## VM Patterns

### P-001: .atb Bytecode Format with Debug Info

Binary bytecode (.atb) with embedded debug info. Source locations for errors.

### P-002: Upvalue Chaining via UpvalueCapture Enum

Closures use `UpvalueCapture::Local(abs_idx)` for direct parent captures and
`UpvalueCapture::Upvalue(parent_idx)` for grandparent+ captures. `register_upvalue_at_depth`
recursively chains through `UpvalueContext` entries. At definition site, emit `GetLocal` for
`Local` captures and `GetUpvalue` for `Upvalue` captures. `upvalue_stack: Vec<UpvalueContext>`
stores `parent_base` (= `prev_local_base` when pushed) + `captures`.
