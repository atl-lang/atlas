# Runtime Patterns

## P-001: Memory Model — Value Semantics + Explicit Ownership
**Status:** LOCKED — v0.3 target | **Updated:** 2026-02-21

Arc<Mutex<Value>> (v0.1–v0.2 bootstrap model) is REPLACED in v0.3 with:
- **Copy-on-write value types** for arrays, maps, objects
- **`own`/`borrow`/`shared` parameter annotations** — explicit ownership in syntax
- **`shared<T>` wrapper** for opt-in reference semantics (replaces implicit Arc)
- **No GC. Ever.** Deterministic allocation is a hard requirement for systems goal.
- **No implicit borrow checker.** Ownership in syntax, not inferred by context.

Full spec: `docs/specification/memory-model.md`
v0.3 block plan: `docs/internal/V03_PLAN.md` (Block 1 = memory model migration)

## P-002: Raw Pointer Threading for SecurityContext
**Status:** Active

`SecurityContext` passed via raw pointer (`Option<*const SecurityContext>`).
Avoids lifetime complexity. Valid for entire eval()/run() scope.

## P-003: Dynamic Scoping vs Lexical Closures
**Status:** Active | **Phase:** v02-completion-05

Atlas uses **dynamic scoping** in the interpreter and **GetGlobal for cross-scope vars** in VM.

- Interpreter: `get_variable()` walks the live scope stack (innermost→outermost + globals)
- VM: top-level let/var = globals (GetGlobal) ✅; outer function locals = stack-allocated → inner fns emit GetGlobal for them which FAILS at runtime
- Result: inner fns CAN access outer fn's locals in interpreter (dynamic scoping), CANNOT in VM

**Pattern:** Only use top-level let/var for cross-function state sharing (both engines).

**Atlas syntax note:** Expression statements REQUIRE semicolons. `f(41)` without `;` fails to parse. All top-level expressions in tests must end with `;`.

**v0.3 plan (Block 4):** Proper lexical closures with CoW value capture semantics.
Type inference is Block 5 (local variables + return types only — NOT full H-M).

## P-B01-01: ValueArray Representation
**Status:** LOCKED | **Date:** 2026-02-21 (Block 1 complete)

`Arc<Vec<Value>>` with `Arc::make_mut` for CoW. Cheap clone (refcount bump), in-place mutation
when exclusively owned, copy-before-mutate when shared.

## P-B01-02: CoW Wrapper Types (all collections)
| Type | Rust | CoW |
|------|------|-----|
| `array<T>` | `ValueArray(Arc<Vec<Value>>)` | `Arc::make_mut` |
| `map<K,V>` | `ValueHashMap(Arc<AtlasHashMap>)` | `Arc::make_mut` |
| `set<T>` | `ValueHashSet(Arc<AtlasHashSet>)` | `Arc::make_mut` |
| `Queue<T>` | `ValueQueue(Arc<VecDeque<Value>>)` | `Arc::make_mut` |
| `Stack<T>` | `ValueStack(Arc<Vec<Value>>)` | `Arc::make_mut` |

## P-B01-03: Shared<T> Implementation
`Arc<Mutex<T>>` for explicit reference semantics. Only used when program annotates `shared<T>`.

## P-B01-04: Equality Semantics
- CoW types (Array, HashMap, etc.): content equality
- Reference types (NativeFunction, Future, async runtime types): pointer equality
- `Shared<T>`: pointer equality (reference semantics by design)

## P-B03-01: Trait Method Mangled Names (Static Dispatch)
**Status:** LOCKED | **Date:** 2026-02-22 (Block 3 scaffold)

Impl methods compile to top-level functions with mangled names:
`__impl__{TypeName}__{TraitName}__{MethodName}`
Example: `impl Display for number` → `fn display` → `__impl__number__Display__display`

- Compiler emits these as regular named functions (no new opcodes)
- `Call` opcode reused — static dispatch, resolved at compile time
- Interpreter uses a `HashMap<(type_name, method_name), ImplMethod>` (no mangling needed)
- The mangled name format is the canonical compile-time key

## P-B03-02: Block 3 Trait Architecture Patterns
**Status:** LOCKED | **Date:** 2026-02-22

- **Static dispatch only** — vtable/trait objects = v0.4
- **`Drop` explicit only** — auto scope-exit invocation = v0.4 (scope tracking not ready)
- **`:` vs `extends`** — separate AST fields: `trait_bounds: Vec<TraitBound>` (new) vs `bound: Option<TypeRef>` (existing `extends`)
- **Copy = all built-in value types** — number/string/bool/null/array/map are Copy; user types default to Move
- **`Display` ≠ `str()`** — Display trait and str() stdlib are independent in Block 3; integration in v0.4
- **AT3001–AT3010** — full diagnostic code range for trait violations

## P-004: CoW Write-Back for Collection Mutation Builtins
**Status:** Active — Phase 15 complete | **Date:** 2026-02-21

Collection mutation builtins (hashMapPut, hashSetAdd, queueEnqueue, stackPush, etc.) return
a NEW collection (CoW semantics). The interpreter and VM must write back to the caller's variable.

**Interpreter** (`interpreter/expr.rs`): `apply_cow_writeback()` + `force_set_collection()` in `mod.rs`.
**VM Compiler** (`compiler/expr.rs`): `emit_cow_writeback_if_needed()` + `emit_force_writeback()`.

**Two patterns:**
- RETURNS_COLLECTION (put/add/enqueue/push/clear): write `result` back to first-arg var, return result
- RETURNS_PAIR (remove/pop/dequeue): returns `[extracted, new_col]`; write `new_col` back, return `extracted`

VM RETURNS_PAIR bytecode: `Dup → Constant(1) → GetIndex → SetVar → Pop → Constant(0) → GetIndex`

**Mutability bypass:** `force_set_collection()` and `emit_force_writeback()` bypass the mutability
check — container content mutation is NOT a variable rebinding. Both `let` and `var` bindings can
have their collection contents mutated via mutation builtins.

## P-B02-01: Ownership Annotation Implementation
**Status:** LOCKED — Block 2 complete | **Date:** 2026-02-22

`OwnershipAnnotation` enum: `Own | Borrow | Shared`
- `Param.ownership: Option<OwnershipAnnotation>` — `None` = unannotated
- `FunctionDecl.return_ownership: Option<OwnershipAnnotation>`
- Runtime enforcement: `debug_assertions` only, zero release overhead

**Enforcement by annotation:**
- `own`: caller binding marked consumed; reuse → runtime error ("consumed")
- `borrow`: NO runtime enforcement — CoW value semantics provide the guarantee
- `shared`: argument must be `Value::SharedValue(_)`, enforced at call time

**Both engines (interpreter + VM) enforce identically** — verified by 22 parity tests.

**v0.4** adds a static dataflow pass over the typed AST — no syntax changes required.
The annotation system is complete and stable.

**LSP (Block 2):**
- Semantic tokens: own/borrow/shared → `KEYWORD` type
- Hover: `find_parameter_hover` shows `(own parameter) name: Type`; `format_function_signature` shows `own name: Type`
- Completion: `ownership_annotation_completions()` + `is_in_param_position()` context gate

---

## P-B03-01: Trait Method Mangling + Static Dispatch

**Decision:** Impl methods are compiled as global functions with mangled names.
**Format:** `__impl__TypeName__TraitName__MethodName`
**Dispatch path:**
1. Typechecker annotates `MemberExpr.trait_dispatch: RefCell<Option<(String, String)>>` via `resolve_trait_method_call_with_info()`
2. Compiler reads annotation → emits `GetGlobal(mangled_name)` + `Call(argc)`
3. Interpreter reads annotation → looks up `function_bodies[mangled_name]` + calls it

**No new opcodes required.** Existing `GetGlobal` + `Call` sufficient.

## P-B03-02: Trait Registry Architecture

**Decision:** `TraitRegistry` in typechecker/mod.rs stores trait declarations separately from `ImplRegistry`.
- `TraitRegistry.traits: HashMap<String, Vec<TraitMethodRecord>>` — trait name → method records
- `ImplRegistry.entries: HashMap<(String, String), ImplEntry>` — (type_name, trait_name) → methods
- `find_trait_with_method(method_name)` — reverse lookup for AT3035 emission

## P-B03-03: LSP Partial AST on Parse Errors

**Decision:** `document.rs` stores partial AST even when parse errors occur.
Previously: returned early with `ast = None` on any parse error.
Now: stores the AST before returning on parse errors, enabling hover/completion for partial code.
This is essential for trait completions (`impl |`) which always involve incomplete source.

## P-B04-01: Zero-Allocation Interpreter Hot Path — Goal Not Gate (2026-02-23)

**Decision:** The interpreter's eval loop should not allocate on the happy path. This is a documented goal, NOT a CI gate.

**Rationale:** CI-gated allocator checks (custom allocator panic, dhat) would break on every new stdlib function, `format!` in error paths, or innocent `Vec::new()` in utility code. High friction for a constraint that only matters post-v0.3 when Atlas targets embedded/real-time workloads.

**What to do instead:**
- The zero-allocation invariant is documented in `atlas-interpreter.md` as a goal
- Manual spot-check at block completion: `grep -n "Vec::new\|String::new\|format!" interpreter/expr.rs`
- Revisit with `dhat` or allocator gate when performance/embedded blocks ship (post-v0.3)

**Does NOT apply to:** error paths, stdlib functions, diagnostics, debug utilities.
