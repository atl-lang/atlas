# Phase 01: Spec + Diagnostics

## Dependencies

**Required:** None — this is the first phase.

**Verification:**
```bash
ls docs/language/async.md
grep "AT4001" crates/atlas-runtime/src/diagnostic/error_codes.rs
```

**If missing:** Nothing to check — this phase creates all artifacts from scratch.

---

## Objective

Author the complete `docs/language/async.md` language spec and register all async-related AT diagnostic error codes. Every subsequent B8 phase implements exactly what this spec defines — spec is law.

---

## Files

**Create:** `docs/language/async.md` (~150 lines)
**Update:** `crates/atlas-runtime/src/diagnostic/error_codes.rs` (~10 new AT codes)
**Tests:** none (spec phase)

**Total new code:** ~160 lines
**Total tests:** 0

---

## Dependencies (Components)

- Diagnostic error code registry (existing — `error_codes.rs`)
- Language spec directory (existing — `docs/language/`)

---

## Implementation Notes

**Key patterns to analyze:**
- Examine existing AT code ranges in `error_codes.rs` to find the next available block (AT4001+ reserved for async)
- Review `docs/language/functions.md` for the spec style and section structure to match
- Reference D-030 as the concurrency model for the spec's concurrency section

**Spec must cover:**
- `async fn` declaration syntax and semantics
- `await` expression syntax and position rules (inside async fn body or at top-level only)
- `Future<T>` as a first-class named type
- Implicit `Future<T>` wrapping: `async fn` always returns `Future<T>` regardless of whether `-> Future<T>` is written explicitly
- Top-level await: allowed without an async wrapper — the runtime executes it via block_on
- Concurrency model: multi-threaded tokio runtime per D-030 — this is the semantic contract
- Error propagation: combining `await` with `?` on `Future<Result<T, E>>`
- Concurrency primitives: `spawn()`, `all()`, `race()` and their relationship to `await`

**AT codes to register (AT4001–AT4010):**
- AT4001: await used outside async fn and outside top-level scope
- AT4002: await applied to a non-Future value
- AT4003: async fn body return type incompatible with declared return type
- AT4004: async fn passed where a sync fn parameter is expected
- AT4005: Future value used as its resolved inner type without await (warning, not error)
- AT4006: main fn declared async — forbidden, use top-level await instead
- AT4007: spawn called in a sync context with no active runtime
- AT4008: Future type parameter mismatch (Future<T> where Future<U> expected, T ≠ U)
- AT4009: async anonymous function — reserved for a future block, not yet supported
- AT4010: await inside a sync for-loop body — ambiguous evaluation order

**Error handling:**
- Each code needs a short human-readable message and a longer hint string
- Follow the exact format used by AT3001–AT3052 in the existing registry

**Integration points:**
- Uses: `error_codes.rs` registry (existing)
- Creates: `docs/language/async.md` (new)

---

## Tests (TDD Approach)

No runtime tests in this phase. The spec and registered codes are verified by later phases.

**Minimum test count:** 0

---

## Acceptance Criteria

- ✅ `docs/language/async.md` complete — all syntax forms, semantics, type rules, concurrency model documented
- ✅ AT4001–AT4010 registered in `error_codes.rs` with messages and hints
- ✅ Spec consistent with D-030 (multi-threaded runtime)
- ✅ No clippy warnings introduced in `error_codes.rs`
- ✅ All subsequent B8 phases can treat this spec as the single source of truth

---

## References

**Decision Logs:** D-030 (multi-threaded async runtime)
**Specifications:** docs/language/functions.md (style reference)
**Related phases:** Phase 02 (keywords), all B8 phases implement this spec
