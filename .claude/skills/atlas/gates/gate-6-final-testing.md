# GATE 6: Final Testing

**Condition:** Implementation complete, ready to commit

---

## Action

**GATE 6 IS A COMMIT. The Guardian hook runs all checks automatically.**

```bash
cargo fmt                          # Run formatter (not --check, just fix it)
git add <files>
git commit -m "feat/fix(...): ..." # Guardian runs: fmt + clippy + tests + parity + battle
```

The Guardian hook handles:
- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- Targeted test suite (based on staged files)
- Parity sweep (if interpreter/VM/compiler touched)
- Battle tests (if runtime touched)

**DO NOT run `cargo nextest run --workspace` or `cargo build --workspace` manually.** Guardian handles it. Manually running these wastes 5-20 minutes per invocation and the hook does it anyway.

---

## Pass Requirement: 100%

- ✅ Guardian passes → Commit created → Proceed to GATE 7
- ❌ Guardian fails → Read the inline failure output → Fix → Commit again

Flaky tests and overly strict assertions are bugs to fix, not excuses to ship.

---

## Failure Triage

1. **Read Guardian output** — it now shows which specific tests failed inline
2. **Reproduce** — `cargo nextest run -p atlas-runtime -E 'test(exact_failing_test)'`
3. **Classify** — wrong output / panic / parity break / flaky / assertion too strict
4. **Fix** — minimal fix, don't refactor unrelated code
5. **Commit** — Guardian confirms 100%

**30 minute limit per failure.** If exceeded: identify root cause, `atlas-track open-issue`, commit partial work with clear message. Next session picks it up.

---

## Coverage (lazy-load `gates/gate-6-coverage.md` for per-crate floors)

**Rule:** Every phase adding new code MUST add tests covering that code. Patch coverage floor: 80% of new lines.

---

**Next:** GATE 7 (Memory Check)
