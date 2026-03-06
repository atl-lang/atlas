---
name: atlas-bugfix
description: Atlas bug fix workflow with TDD. Use for fixing bugs, resolving issues, hardening sessions. Strict test-first development — write failing test, then fix.
---

# Atlas — Bug Fix Workflow

**Prerequisite:** Core `atlas` skill must be active. If not, activate it first.

---

## TDD Protocol (MANDATORY for bug fixes)

### Step 1: Write Failing Test (RED)
```rust
#[test]
fn test_issue_h_xxx_description() {
    // Reproduces the bug
    assert_parity(r#"code that triggers bug"#, "expected_output");
}
```

### Step 2: Verify Test Fails (BLOCKING)
```bash
cargo nextest run -p atlas-runtime -- test_issue_h_xxx
```
If the test passes → you misunderstand the bug. Re-investigate.

### Step 3: Locate Root Cause
- Read error output completely
- Grep for relevant code patterns
- Check if it's an interpreter issue, VM issue, or both

### Step 4: Minimal Fix
Fix only what's broken. Don't refactor surrounding code.

### Step 5: Verify Test Passes (GREEN)
```bash
cargo nextest run -p atlas-runtime -- test_issue_h_xxx
```

### Step 6: Full Suite + Parity
```bash
cargo nextest run --workspace                              # No regressions
cargo nextest run -p atlas-runtime -E 'test(parity)'       # Parity intact
cargo fmt --check && cargo clippy --workspace -- -D warnings
coderabbit review                                          # Quality check
```

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

## Failure Triage

If debugging exceeds 30 minutes on a single failure:
1. Identify root cause precisely
2. Run `atlas-track open-issue` to document the blocker
3. Commit partial work with clear message
4. Next session picks it up via `atlas-track go`

---

## Quality Gates (after fix)

```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo nextest run --workspace
cargo nextest run -p atlas-runtime -E 'test(parity)'    # Parity sweep (ALWAYS)
coderabbit review
```

**If fix touches runtime/stdlib/VM/compiler** — also run battle tests:
```bash
for f in battle-test/hydra-v2/**/*.atlas; do
    atlas run "$f" 2>&1 || echo "BATTLE TEST FAILED: $f"
done
```

All must pass. Commit only after green.

---

## Test Scope (Token Efficiency)

Run TARGETED tests during TDD, not full workspace. See `gates/test-partitioning.md`.

```bash
# RED/GREEN phases: single test only
cargo nextest run -p atlas-runtime -E 'test(test_issue_h_xxx)'

# After fix: crate-scoped
cargo nextest run -p <affected-crate>

# Final verification: full workspace (once, at the end)
cargo nextest run --workspace
```

## Oracle Verification

If the expected behavior is unclear, verify against Rust/TypeScript. See `gates/oracle-testing.md`.

---

## Deeper Reference
- Parity practices: auto-memory `compiler-quality/parity.md`
- AI compiler lessons: auto-memory `compiler-quality/ai-compiler.md`
- Oracle testing: `gates/oracle-testing.md`
- Test partitioning: `gates/test-partitioning.md`
