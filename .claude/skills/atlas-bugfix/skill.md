---
name: atlas-bugfix
description: Atlas bug fix workflow with TDD. Use for fixing bugs, resolving issues, hardening sessions. Strict test-first development — write failing test, then fix.
---

# Atlas — Bug Fix Workflow

**Prerequisite:** Core `atlas` skill must be active. If not, activate it first.

---

## Testing — Two-Tier System (CRITICAL)

### Tier 1: Pre-commit (automatic, < 15s)
- `cargo fmt --check` + `cargo clippy` only — NO nextest, by design

### Tier 2: Nightly CI (2am via launchd, or `atlas-track run-ci`)
- Full suite, parity, battle tests → results in `tracking/ci-status.json`

### BANNED — all nextest except ONE exact TDD test:
```bash
cargo nextest run -p atlas-runtime -E 'test(interpreter)'    # ❌ BANNED
cargo nextest run -p atlas-runtime -E 'test(regression)'     # ❌ BANNED
cargo nextest run -p atlas-runtime -E 'test(stdlib)'         # ❌ BANNED
cargo nextest run -p atlas-runtime -E 'test(corpus)'         # ❌ BANNED
cargo nextest run -p atlas-runtime                           # ❌ BANNED (full suite)
cargo nextest run --workspace                                 # ❌ BANNED
```

### ALLOWED — cargo check + ONE exact TDD test only:
```bash
cargo check -p atlas-runtime                                           # ✅ ~0.5s always fine
cargo nextest run -p atlas-runtime -E 'test(my_exact_test_name)'      # ✅ TDD only, exact name
atlas run /tmp/test.atlas                                              # ✅ FASTEST — use this first
```

### ⛔ STOP — CLI confirmation supersedes nextest

**If you already confirmed correctness with `atlas run file.atlas` → DO NOT run nextest.**

Nextest compiles every test binary in the crate before running one (~1-5 min due to aws-lc-sys).
Running it after a CLI-confirmed fix wastes the entire time for zero additional information.

**Decision tree:**
1. Can I verify with `atlas run /tmp/test.atlas`? → Yes → do that, skip nextest, commit
2. Is this a Rust unit test with no CLI equivalent? → Then use nextest with exact name only
3. Never run nextest "just to be sure" after CLI already confirmed it

---

## TDD Protocol (MANDATORY for bug fixes)

### Step 1: Write Failing Test (RED)
```rust
#[test]
fn test_issue_h_xxx_description() {
    // Reproduces the bug
    assert_eval_number(r#"code that triggers bug"#, expected);
}
```

### Step 2: Verify Test Fails (BLOCKING)
```bash
cargo nextest run -p atlas-runtime -E 'test(test_issue_h_xxx_description)'
```
If the test passes → you misunderstand the bug. Re-investigate.
**Use exact test name, not a broad filter.**

### Step 3: Locate Root Cause
- `cargo check -p atlas-runtime` to verify compile (never nextest for this)
- Grep for relevant code patterns
- Read error output completely

### Step 4: Minimal Fix
Fix only what's broken. Don't refactor surrounding code.

### Step 5: Verify Fix Works (GREEN)

**Prefer CLI verification — it's instant:**
```bash
atlas run /tmp/repro.atlas   # ✅ If output is correct → done, skip nextest
```

**Only use nextest if there's no CLI equivalent (pure Rust unit test):**
```bash
cargo nextest run -p atlas-runtime -E 'test(test_issue_h_xxx_description)'
```
**Same exact test name. Nothing broader. And ONLY if CLI can't confirm it.**

### Step 6: Pre-commit checks then commit
```bash
cargo fmt                                                    # Format (not --check, just fix it)
git add <files>
git commit -m "fix(...): description"                        # Guardian hook runs full suite + parity
```

**DO NOT run any nextest between Step 5 and commit.** Guardian handles it.
**If commit fails:** read Guardian output carefully for which test failed, fix it, repeat from Step 6.

---

## Issue Lifecycle
```bash
atlas-track claim H-001              # Before starting
# ... TDD fix cycle ...
atlas-track fix H-001 "Root cause (10+ chars)" "Fix applied (10+ chars)"
```

---

## When to Use Parity Tests

**Always** if the bug involves:
- Interpreter behavior
- VM/compiler behavior
- Stdlib functions (they run in both engines)
- Any runtime output

**Skip parity only for:** LSP, CLI, JIT, formatter, config (single-engine components)

---

## When Guardian Commit Fails

1. **Read the Guardian output** — it shows which phase failed (fmt or clippy)
2. **For fmt failures:** run `cargo fmt` then commit again
3. **For clippy failures:** fix the warnings shown, then commit again
4. `cargo fmt && git add -A && git commit ...` — try again
5. **Test failures appear in nightly CI, not pre-commit.** Run `atlas-track ci-status` to see CI results.
6. If CI is failing: `atlas-track run-ci` to get details, fix the specific failing tests.

---

## Failure Triage

If debugging exceeds 30 minutes on a single failure:
1. Identify root cause precisely
2. Run `atlas-track open-issue` to document the blocker
3. Commit partial work with clear message
4. Next session picks it up via `atlas-track go`

---

## Deeper Reference
- Parity practices: auto-memory `compiler-quality/parity.md`
- AI compiler lessons: auto-memory `compiler-quality/ai-compiler.md`
- Oracle testing: `gates/oracle-testing.md`
- Test partitioning: `gates/test-partitioning.md`
