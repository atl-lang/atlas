# Phase 15: AC Verification + Block Close

## Dependencies

**Required:** Phase 14 complete (all phases 01–14 done)

**Verification:**
```bash
atlas-track block B8
cargo check --workspace
cargo fmt --check
cargo clippy --workspace -- -D warnings
atlas-track ci-status
```

**If missing:** All prior phases must be complete and committed before this gate runs.

---

## Objective

Final acceptance criteria gate: verify all B8 deliverables are complete, all tests pass, spec is accurate, and the block is closed in atlas-track.

---

## Files

No new code. This phase is a verification and close sequence only.

**Total new code:** 0
**Total tests:** 0

---

## Dependencies (Components)

- All phases 01–14
- `atlas-track` CLI
- `cargo check`, `cargo fmt`, `cargo clippy`

---

## Implementation Notes

**Key patterns to analyze:**
- Review the AC verification pattern from a completed block (e.g., B6 or B7) in `phases/v0.3/` to match the close sequence

**AC checklist to verify:**

Language features:
- `async fn` syntax parses and executes in both interpreter and VM
- `await` expression syntax parses and executes in both engines
- `Future<T>` is a first-class type
- Top-level `await` works without an async fn wrapper
- Implicit `Future<T>` wrapping on async fn return type
- AT4001 enforced: await only valid inside async fn or at top-level
- AT4002 enforced: await only valid on Future values

Runtime:
- Multi-threaded tokio runtime active — `new_multi_thread()` (D-030)
- `Value: Send` compile-time assertion present
- No known deadlock paths in block_on usage
- `spawn`, `all`, `race`, `sleep`, `timeout` work with `await`

Parity:
- 100% interpreter/VM parity — zero known divergences
- Error messages identical in both engines
- All parity tests pass

Quality:
- AT4001–AT4010 registered, all tested
- `docs/language/async.md` complete and accurate against the implementation
- Zero clippy warnings
- `cargo fmt --check` passes
- CI run triggered or scheduled

Test coverage:
- 25+ typechecker tests
- 30+ interpreter async tests
- 20+ VM async tests
- 20+ stdlib async tests
- 25+ parity tests
- 14+ battle test programs
- 7+ LSP tests
- Total: 141+ new test cases

**Error handling:**
- Any failed AC item is a blocker — do not close the block with open failures

**Integration points:**
- Uses: atlas-track for block status update and session close

---

## Tests (TDD Approach)

No new test cases. Run existing suite:
```bash
cargo nextest run -p atlas-runtime -E 'test(async)'
cargo nextest run -p atlas-runtime -E 'test(parity)'
cargo nextest run -p atlas-lsp -E 'test(async)'
```

**Minimum test count:** 0 new (all prior phases supply the tests)

---

## Close Sequence

1. Run full verification checklist above
2. Confirm `atlas-track block B8` shows `15/15` phases done
3. Run `cargo check --workspace`, `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`
4. Commit with message: `feat(async): complete B8 — async/await, Future<T>, multi-thread VM, 141+ tests`
5. Close session: `atlas-track done S-XXX success "summary" "next steps"`
6. Update MEMORY.md if new patterns were discovered during B8

---

## Acceptance Criteria

- ✅ All language features from spec implemented and verified
- ✅ D-030 satisfied: multi-thread runtime, Value: Send confirmed
- ✅ 100% parity: zero known interpreter/VM divergences
- ✅ 141+ new test cases across all test files
- ✅ Zero clippy warnings, fmt clean
- ✅ `atlas-track block B8` shows `complete 15/15`
- ✅ Session closed via `atlas-track done`
- ✅ MEMORY.md updated if new patterns found

---

## References

**Decision Logs:** D-030 (multi-thread async runtime)
**Specifications:** docs/language/async.md
**Related phases:** All B8 phases (01–14)
