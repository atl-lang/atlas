# GATE 6: Final Testing

**Condition:** Implementation complete, ready to commit

**Source of truth:** auto-memory `testing-patterns.md` for crate-specific testing protocols

---

## Action

Run the full test suite for the package you modified:

```bash
cargo build --workspace                          # Full build — MUST be clean
cargo nextest run -p <package>                   # e.g., atlas-lsp, atlas-runtime, atlas-cli
cargo clippy -p <package> -- -D warnings         # Zero warnings
cargo fmt --check -p <package>                   # Perfectly formatted
```

**All four must pass. No exceptions.**

---

## Pass Requirement: 100%

**There is no acceptable failure rate.** Every test must pass before committing.

- ✅ 100% pass → Proceed to GATE 7
- ❌ Any failure → Fix it. Do not proceed.

**If a test is genuinely flaky** (non-deterministic, timing-dependent): fix the test first, then commit. Flaky tests are not an excuse to ship — they are a bug to fix.

**If a test has an overly strict assertion**: fix the assertion to match the correct behavior, then commit. A wrong test is still a failure.

---

## Failure Triage

**When tests fail, work through this in order:**

1. **Understand it** — read the failure output completely
2. **Reproduce it** — run the specific test in isolation: `cargo nextest run -p <package> -- test_name`
3. **Classify it:**
   - Wrong output → bug in implementation, fix the code
   - Panic/crash → bug in implementation, fix the code
   - Parity break → CRITICAL, both engines must match, fix both
   - Flaky (intermittent) → fix the test
   - Assertion too strict → fix the assertion to match correct behavior
4. **Fix it** — minimal fix, don't refactor unrelated code
5. **Re-run full suite** — confirm 100%

**Time limit:** If debugging a single failure exceeds 30 minutes, stop and escalate to user. Do not ship with failures.

---

## Examples

### ✅ Correct: Proceed

```
7151 tests: 7151 passed
Build: clean
Clippy: 0 warnings
Fmt: clean
```

### ❌ Wrong: Stop and Fix

```
277 tests: 273 passed, 4 failed
Failures: test_code_actions, test_hover, test_completion, test_inlay_hints
```

4 failures = 4 bugs. Fix them. "The feature works" is not a valid reason to ship failing tests.

---

---

## Coverage Awareness (Per-Phase)

Coverage is enforced by Codecov on CI — you don't manually run tarpaulin locally (too slow). But you are responsible for not shipping phases that **actively decrease** coverage:

**Rule:** Every phase that adds new code MUST add tests that cover that code. This is implicit in the AC criteria, but the explicit floor is:

| Crate | Minimum floor | If below → STOP |
|-------|--------------|-----------------|
| `atlas-runtime` | 70% | Fix before merge |
| `atlas-cli` | 50% | Fix before merge |
| `atlas-formatter` | 60% | Fix before merge |
| `atlas-lsp` | 40% | Fix before merge |
| `atlas-jit` | 25% | Fix before merge |
| `atlas-config` | 60% | Fix before merge |
| `atlas-build` | 40% | Fix before merge |
| `atlas-package` | 40% | Fix before merge |

You will not see coverage numbers locally until CI runs. The check is: **did I write tests for every code path I added?** If yes, CI will confirm. If no, fix now.

**Patch coverage floor:** 80% of new lines must be covered. Unreachable/dead code paths are the only valid exception.

---

**Next:** GATE 7 (Memory Check)
