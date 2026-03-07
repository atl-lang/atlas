# Phase 15: AC Verification + Block Close

## Dependencies

**Required:** All phases 01–14 complete

**Verification:**
```bash
atlas-track block B8
cargo check --workspace
atlas-track ci-status
```

---

## Objective

Final acceptance criteria gate: verify all B8 deliverables are complete, CI is green, spec is accurate, and block is closed.

---

## Acceptance Criteria Checklist

### Language Features
- ✅ `async fn` syntax parses and executes in interpreter and VM
- ✅ `await` expression syntax parses and executes in both engines
- ✅ `Future<T>` as first-class type
- ✅ Top-level `await` works without async wrapper
- ✅ `async fn` return type implicitly `Future<T>` (or explicit `-> Future<T>`)
- ✅ `await` only valid inside async fn or at top-level (AT4001 enforced)
- ✅ `await` on non-Future emits AT4002

### Runtime
- ✅ Multi-threaded tokio runtime active (D-030 — `new_multi_thread()`)
- ✅ `Value: Send` compile-time assertion present
- ✅ No deadlock risk in block_on usage
- ✅ `spawn()`, `all()`, `race()`, `sleep()`, `timeout()` work with await

### Parity
- ✅ 100% interpreter/VM parity — zero known divergences
- ✅ Error messages identical in both engines
- ✅ All parity tests pass

### Quality
- ✅ AT4001–AT4010 registered and tested
- ✅ `docs/language/async.md` complete and accurate
- ✅ 0 clippy warnings (`cargo clippy --workspace -- -D warnings`)
- ✅ `cargo fmt --check` passes
- ✅ CI green (or CI run scheduled with known-passing suite)

### Test Coverage
- ✅ 25+ typechecker tests
- ✅ 30+ interpreter async tests
- ✅ 20+ VM async tests
- ✅ 20+ stdlib async tests
- ✅ 25+ parity tests
- ✅ 10+ battle test programs
- ✅ 7+ LSP tests
- ✅ **Total: 137+ new test cases**

### Tracking
- ✅ `atlas-track block B8` shows `complete 15/15`
- ✅ D-030 decision recorded (multi-thread runtime)
- ✅ Session closed: `atlas-track done S-XXX success "..." "..."`
- ✅ MEMORY.md updated if new patterns discovered

---

## Close Sequence

```bash
# 1. Verify block
atlas-track block B8

# 2. Final build check
cargo check --workspace
cargo fmt --check
cargo clippy --workspace -- -D warnings

# 3. Commit
git add -A
git commit -m "feat(async): complete B8 — async/await syntax, Value::Future, multi-thread VM, parity"

# 4. Close session
atlas-track done S-XXX success \
  "B8 Async/Await complete: async fn, await expr, Future<T> type, multi-thread tokio (D-030), 137+ tests, 100% interpreter/VM parity" \
  "B9 Quick Wins is complete. Consider starting B10 or next planned block."
```

---

## References

**Decision Logs:** D-030 (multi-thread async runtime)
**Spec:** docs/language/async.md
**All B8 phases:** Phase 01–14
