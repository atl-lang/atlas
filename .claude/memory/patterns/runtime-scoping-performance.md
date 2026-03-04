# Runtime Patterns: Scoping + Performance

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

## P-B04-01: Zero-Allocation Interpreter Hot Path — Goal Not Gate (2026-02-23)

**Decision:** The interpreter's eval loop should not allocate on the happy path. This is a documented goal, NOT a CI gate.

**Rationale:** CI-gated allocator checks (custom allocator panic, dhat) would break on every new stdlib function, `format!` in error paths, or innocent `Vec::new()` in utility code. High friction for a constraint that only matters post-v0.3 when Atlas targets embedded/real-time workloads.

**What to do instead:**
- The zero-allocation invariant is documented in `atlas-interpreter.md` as a goal
- Manual spot-check at block completion: `grep -n "Vec::new\\|String::new\\|format!" interpreter/expr.rs`
- Revisit with `dhat` or allocator gate when performance/embedded blocks ship (post-v0.3)

**Does NOT apply to:** error paths, stdlib functions, diagnostics, debug utilities.
