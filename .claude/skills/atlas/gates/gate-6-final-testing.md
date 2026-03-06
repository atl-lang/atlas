# GATE 6: Final Testing

**Condition:** Implementation complete, ready to commit

---

## Action

**GATE 6 IS A COMMIT. The Guardian hook runs static analysis automatically.**

```bash
cargo fmt                          # Run formatter (not --check, just fix it)
git add <files>
git commit -m "feat/fix(...): ..." # Guardian runs: fmt + clippy (< 15s)
```

The Guardian hook handles:
- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`

Full test validation (corpus + nextest + parity + battle) runs via **Nightly CI at 2am**.
Check results with `atlas-track go` or `atlas-track ci-status`.

**DO NOT run `cargo nextest run --workspace` or any broad nextest command manually.**
Nightly CI handles it. Manually running these takes 10-20+ minutes for no benefit.

---

## Pass Requirement: 100%

- Guardian passes (fmt + clippy) → Commit created → Proceed to GATE 7
- Guardian fails → Read the inline failure output → Fix → Commit again
- CI red → Fix CI failures before starting new work (P0 blocker)

Flaky tests and overly strict assertions are bugs to fix, not excuses to ship.

---

## Failure Triage

### Guardian (pre-commit) fails:
1. **fmt failure** → `cargo fmt` then commit again
2. **clippy failure** → Fix the warnings shown, commit again

### Nightly CI fails (visible in `atlas-track go`):
1. `atlas-track ci-status` — see what failed
2. For specific test failures: `cargo nextest run -p atlas-runtime -E 'test(exact_failing_test)'`
3. Fix the specific failing test(s)
4. `cargo fmt && git add -A && git commit` — CI re-runs nightly

**30 minute limit per failure.** If exceeded: identify root cause, `atlas-track add "CI: ..." P0 "reason"`, commit partial work with clear message.

---

## Coverage (lazy-load `gates/gate-6-coverage.md` for per-crate floors)

**Rule:** Every phase adding new code MUST add tests covering that code. Patch coverage floor: 80% of new lines.

---

**Next:** GATE 7 (Memory Check)
