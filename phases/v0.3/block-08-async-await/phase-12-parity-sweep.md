# Phase 12: Parity Sweep

## Dependencies

**Required:** Phase 11 complete (stdlib wired; both engines and all async features fully implemented)

**Verification:**
```bash
cargo nextest run -p atlas-runtime -E 'test(parity)'
cargo nextest run -p atlas-runtime -E 'test(async)'
```

**If missing:** All of phases 01–11 must be complete — parity sweep is only meaningful once both engines have full async implementations.

---

## Objective

Exhaustive parity verification: every async Atlas program must produce byte-for-byte identical output in both interpreter and VM. Find and fix all divergences before proceeding.

---

## Files

**Update:** `crates/atlas-runtime/tests/async_runtime.rs` — add structured parity test harness if not already present
**Create:** `crates/atlas-runtime/tests/async_parity.rs` if `async_runtime.rs` exceeds 12KB after additions

**Total new code:** ~150 lines parity tests
**Total tests:** ~25 test cases

---

## Dependencies (Components)

- Interpreter async execution (Phase 09)
- VM async execution (Phase 10)
- Stdlib async (Phase 11)
- Parity test harness from `compiler-quality/parity.md` (auto-memory)

---

## Implementation Notes

**Key patterns to analyze:**
- Review `compiler-quality/parity.md` in auto-memory for the established parity test harness pattern used in earlier blocks
- Check the 12KB file size limit — if `async_runtime.rs` is already large from Phases 09/10/11, split before adding more tests

**Critical requirements:**
- Parity test function runs the same Atlas source through both engines and asserts identical string output
- Every parity failure is a bug — there is no "acceptable divergence"
- Error messages must be identical — not just error codes but the full user-facing string
- Check file sizes before adding tests:  `du -sh crates/atlas-runtime/tests/async_runtime.rs`

**What to check for known divergence sources:**
- Future resolution order (concurrent tasks may resolve in different orders — tests must not assert ordering unless deterministic)
- Error message formatting — interpreter and VM may use different code paths to the same error code
- `type_name()` and `Display` — these read from `value.rs` directly, so identical, but verify
- Void return handling — async fn returning void may differ if one engine pushes Null and the other pushes a different sentinel

**Integration points:**
- Uses: both engines via the parity harness
- Gate for: Phase 13 (battle tests assume parity is solid)

---

## Tests (TDD Approach)

Run each program through both engines; assert output equality.

**Core async programs** (10 programs)
1. Minimal async fn returning a number — await it
2. Async fn returning each primitive type: string, bool, null
3. Nested async: three levels deep
4. Async fn with early return
5. Async fn with if/else branch — both branches tested
6. Async fn with for loop body
7. Async fn with a closure
8. Top-level await at program entry
9. Async fn returning void
10. Async fn with `?` on a Result

**Stdlib async programs** (8 programs)
11. `await sleep(0)`
12. `await all([f1, f2, f3])` with deterministic futures
13. `await race([f1, f2])` where winner is deterministic
14. `await spawn(fn)` + await the handle
15. `await read_file_async` / `write_file_async` round-trip
16. Timeout that fires — error message identical
17. Timeout that does not fire — value identical
18. `await fetch` with a local mock

**Error cases** (7 programs)
19. AT4001 error message — identical in both engines
20. AT4002 error message — identical in both engines
21. Async fn returning RuntimeError — propagation identical
22. Nested error with `?` — final error string identical
23. `await` on null — error identical
24. Future that panics internally — contained error identical
25. Race with one erroring future — behavior identical

**Minimum test count:** 25 programs (25 parity tests)

---

## Acceptance Criteria

- ✅ 25+ parity test programs defined
- ✅ All 25 pass — interpreter output == VM output for every program
- ✅ Zero known parity divergences
- ✅ Error messages identical in both engines
- ✅ File sizes within 12KB limit
- ✅ `cargo check -p atlas-runtime` clean

---

## References

**Decision Logs:** D-004 (parity is sacred), D-030
**Specifications:** auto-memory `compiler-quality/parity.md`
**Related phases:** Phase 11 (all implementations), Phase 13 (battle tests build on this parity baseline)
