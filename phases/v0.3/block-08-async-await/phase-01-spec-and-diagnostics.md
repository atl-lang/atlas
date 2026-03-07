# Phase 01: Spec + Diagnostics

## Dependencies

**Required:** None (spec authoring phase)

**Verification:**
```bash
ls docs/language/async.md
grep "AT4001\|AT4002\|AT4003" crates/atlas-runtime/src/diagnostic/error_codes.rs
```

**If missing:** This is phase 1 — no prior dependencies.

---

## Objective

Author the complete `docs/language/async.md` language spec and register all async-related AT diagnostic error codes. All subsequent phases implement exactly what this spec defines — spec is law.

---

## Files

**Create:** `docs/language/async.md` (~150 lines)
**Update:** `crates/atlas-runtime/src/diagnostic/error_codes.rs` (+10 AT codes)
**Tests:** none (spec phase)

**Total new code:** ~160 lines
**Total tests:** 0

---

## Spec Contents (docs/language/async.md)

### Syntax
```atlas
// Async function declaration
async fn fetch(url: string) -> string { ... }

// Await expression (only inside async fn or top-level)
let result = await fetch("https://example.com")

// Top-level await (auto block_on)
let data = await read_file("data.txt")

// Return type: implicit Future<T> wrapping
// async fn foo() -> number  ===  fn foo() -> Future<number>
```

### Semantics
- `async fn` implicitly wraps return value in `Future<T>`
- `await` suspends current async context until future resolves
- Top-level `await` blocks on the tokio runtime via `block_on`
- `await` is only valid inside `async fn` bodies or at top-level
- Error propagation: `await` unwraps `Future<Result<T, E>>` — combine with `?`
- Concurrency model: multi-threaded tokio runtime (D-030)

### Type System
- `Future<T>` is a first-class type (e.g., `let f: Future<number> = async_fn()`)
- `async fn foo() -> T` has type `fn(...) -> Future<T>`
- `await expr` where `expr: Future<T>` yields `T`

### Concurrency
- `spawn(async_fn())` — spawn concurrent task, returns `Future<T>`
- `await all([f1, f2, f3])` — await all futures (stdlib)
- `await race([f1, f2, f3])` — await first to resolve (stdlib)

---

## Diagnostic Error Codes

| Code | Name | When |
|------|------|------|
| AT4001 | AwaitOutsideAsync | `await` used outside `async fn` and outside top-level |
| AT4002 | AwaitNonFuture | `await` applied to non-Future value |
| AT4003 | AsyncReturnTypeMismatch | async fn body returns T but declared return is not Future<T> or T |
| AT4004 | AsyncFnInSyncContext | async fn passed where sync fn expected with no coercion |
| AT4005 | FutureNotAwaited | Future<T> value used as T without await (warning, not error) |
| AT4006 | AsyncMainNotAllowed | `main` fn cannot be `async` (use top-level await instead) |
| AT4007 | SpawnOutsideAsync | `spawn()` called in sync context with no runtime |
| AT4008 | FutureIncompatibleTypes | Future<T> and Future<U> used where T ≠ U |
| AT4009 | AsyncClosureNotSupported | `async` anonymous functions (reserved for future block) |
| AT4010 | AwaitInIterator | `await` inside `for` loop over sync iterator (ambiguous) |

---

## Acceptance Criteria

- ✅ `docs/language/async.md` complete, covers all syntax forms
- ✅ AT4001–AT4010 registered in error_codes.rs
- ✅ Spec reviewed against D-030 (multi-threaded runtime)
- ✅ All subsequent B8 phases can reference this spec as law

---

## References

**Decision Logs:** D-030 (multi-threaded async runtime)
**Related phases:** Phase 02 (Keywords), all B8 phases implement this spec
