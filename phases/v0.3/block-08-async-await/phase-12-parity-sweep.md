# Phase 12: Parity Sweep

## Dependencies

**Required:** Phase 09 (interpreter), Phase 10 (VM), Phase 11 (stdlib wiring)

**Verification:**
```bash
# Run parity test suite
cargo nextest run -p atlas-runtime -E 'test(parity)'
cargo nextest run -p atlas-runtime -E 'test(async)'
```

---

## Objective

Exhaustive parity verification: every async Atlas program must produce identical output in both interpreter and VM. Fix all divergences before proceeding.

---

## Files

**Update:** `crates/atlas-runtime/tests/async_runtime.rs` — add parity test harness for all async features
**Create:** `crates/atlas-runtime/tests/async_parity.rs` (if async_runtime.rs exceeds 12KB)

**Total new code:** ~150 lines parity tests

---

## Parity Test Matrix

For each program below, run in both interpreter and VM and assert identical output:

**Core async programs:**
1. Minimal async fn + await
2. Async fn returning each primitive type (number, string, bool, null)
3. Nested async calls (3 levels deep)
4. Async fn with early return
5. Async fn with if/else branches
6. Async fn with for loop
7. Async fn with while loop
8. Async fn with closures
9. Async fn with struct method call
10. Top-level await at program entry

**Stdlib async:**
11. `await sleep(0)`
12. `await all([f1, f2, f3])`
13. `await race([f1, f2])`
14. `await spawn(fn)` + collect result
15. `await read_file_async` / `write_file_async`

**Error handling:**
16. async fn returning `Result<T, E>` + await + `?`
17. `await` on failed future — error propagation
18. `timeout` that times out — error identical
19. AT4001 error message identical
20. AT4002 error message identical

**Edge cases:**
21. Empty async fn body
22. Async fn with no explicit return (void)
23. `await` result discarded (used as statement)
24. `await` in binary expression
25. Multiple futures awaited sequentially

---

## Parity Test Harness

```rust
fn run_parity(atlas_src: &str) -> (String, String) {
    let interp_output = run_interpreter(atlas_src);
    let vm_output = run_vm(atlas_src);
    (interp_output, vm_output)
}

#[test]
fn test_async_parity_basic() {
    let (i, v) = run_parity(r#"
        async fn greet(name: string) -> string {
            return "Hello " + name
        }
        let result = await greet("Atlas")
        print(result)
    "#);
    assert_eq!(i, v, "Parity divergence in basic async");
}
```

---

## Acceptance Criteria

- ✅ 25+ parity tests defined
- ✅ All 25 pass with identical interpreter/VM output
- ✅ Zero parity divergences remaining
- ✅ Error messages identical in both engines
- ✅ `cargo check -p atlas-runtime` clean
- ✅ File size of async test files ≤ 12KB (split if needed)

---

## References

**Decision Logs:** D-004 (parity is sacred), D-030
**Spec:** compiler-quality/parity.md in auto-memory
**Related phases:** Phase 13 (battle tests build on parity baseline)
