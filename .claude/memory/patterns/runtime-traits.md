# Runtime Patterns: Traits

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
