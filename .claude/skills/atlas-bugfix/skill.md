---
name: atlas-bugfix
description: Atlas bug fix workflow with TDD. Use for fixing bugs, resolving issues, hardening sessions. Strict test-first development — write failing test, then fix.
---

# Atlas — Bug Fix Workflow

**Prerequisite:** Core `atlas` skill must be active. If not, activate it first.

---

## ⚠️ CRITICAL: Test Commands (READ BEFORE TOUCHING NEXTEST)

### BANNED — these compile ALL test binaries and take 5-10 minutes each:
```bash
# NEVER USE THESE:
cargo nextest run -p atlas-runtime -E 'test(interpreter)'    # BANNED
cargo nextest run -p atlas-runtime -E 'test(regression)'     # BANNED
cargo nextest run -p atlas-runtime -E 'test(stdlib)'         # BANNED
cargo nextest run -p atlas-runtime -E 'test(corpus)'         # BANNED
cargo nextest run -p atlas-runtime -E 'test(frontend)'       # BANNED
cargo nextest run -p atlas-runtime                           # BANNED (full suite)
cargo nextest run --workspace                                 # BANNED
```

### ALLOWED — target ONE test by exact name:
```bash
# GREEN: these compile only what's needed
cargo check -p atlas-runtime                                  # Type check only, ~0.5s
cargo nextest run -p atlas-runtime -E 'test(my_exact_test_name)'  # Single test
```

**If Guardian fails and you can't see which test failed** → add `--no-fail-fast` flag:
```bash
cargo nextest run -p atlas-runtime -E 'test(regression) + test(interpreter)' --no-fail-fast 2>&1 | grep "^        FAIL"
```
This is a ONE-TIME diagnostic command to find failing test names, not part of normal TDD flow.

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

### Step 5: Verify Test Passes (GREEN)
```bash
cargo nextest run -p atlas-runtime -E 'test(test_issue_h_xxx_description)'
```
**Same exact test name. Nothing broader.**

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

1. **Read the Guardian output** — it shows which phase failed (fmt/clippy/tests/parity)
2. **For test failures:** the output may be truncated. Run this ONE-TIME to find what failed:
   ```bash
   cargo nextest run -p atlas-runtime -E '<same filter Guardian used>' --no-fail-fast 2>&1 | grep "^        FAIL"
   ```
3. **Fix the specific failing test(s)** — update test fixtures if they used wrong syntax
4. `cargo fmt && git add -A && git commit ...` — try again
5. **Never run broad test filters** trying to verify the whole suite. Guardian does that.

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
