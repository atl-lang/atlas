# GATE 6: Final Testing

**Condition:** Implementation complete, ready to commit

---

## Action

```bash
cargo build --workspace                          # Full build — MUST be clean
cargo nextest run --workspace                    # All crates, full coverage
cargo clippy --workspace -- -D warnings          # Zero warnings
cargo fmt --check                                # Perfectly formatted
```

**All four must pass. No exceptions.**

### Parity Sweep (if runtime/VM/compiler touched)

```bash
cargo nextest run -p atlas-runtime -E 'test(parity)'
```

### Battle Test Regression (if runtime touched)

```bash
for f in battle-test/hydra-v2/**/*.atlas; do
    atlas run "$f" 2>&1 || echo "BATTLE TEST FAILED: $f"
done
```

---

## Pass Requirement: 100%

- ✅ 100% pass → Proceed to GATE 7
- ❌ Any failure → Fix it. Do not proceed.

Flaky tests and overly strict assertions are bugs to fix, not excuses to ship.

---

## Failure Triage

1. **Understand** — read failure output completely
2. **Reproduce** — `cargo nextest run -p <package> -- test_name`
3. **Classify** — wrong output / panic / parity break / flaky / assertion too strict
4. **Fix** — minimal fix, don't refactor unrelated code
5. **Re-run full suite** — confirm 100%

**30 minute limit per failure.** If exceeded: identify root cause, `atlas-track open-issue`, commit partial work with clear message. Next session picks it up.

---

## Coverage (lazy-load `gates/gate-6-coverage.md` for per-crate floors)

**Rule:** Every phase adding new code MUST add tests covering that code. Patch coverage floor: 80% of new lines.

---

**Next:** GATE 7 (Memory Check)
