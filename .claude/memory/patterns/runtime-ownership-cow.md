# Runtime Patterns: Ownership + CoW

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
