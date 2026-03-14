---
name: atlas-bugfix
description: Atlas bug fix workflow with TDD. Use for fixing bugs, resolving issues, hardening sessions. Strict test-first development — write failing test, then fix.
---

# Atlas — Bug Fix Workflow

**Prerequisite:** Core `atlas` skill must be active. If not, activate it first.

---

## Testing (Two-Tier System — see CLAUDE.md for full rules)

**FASTEST:** `atlas run /tmp/test.atlas` — use before any nextest invocation.
**ALLOWED:** `cargo check -p atlas-runtime` + ONE exact TDD nextest (see below).
**BANNED:** All other nextest invocations. CLI confirmation supersedes nextest — don't run both.

Decision tree:
1. `atlas run /tmp/test.atlas` confirms it → commit, skip nextest
2. Pure Rust unit test with no CLI equivalent → nextest with exact name only
3. Never run nextest "just to be sure" after CLI confirmed

---

## SYSTEMIC SCOPE AUDIT (MANDATORY — run BEFORE writing any code)

**This is Step 0. You cannot skip it. Partial fixes are worse than no fix.**

When you claim an issue, the first thing you do is determine the FULL scope:

```bash
# 1. Identify the broken pattern
#    Example: "Type::Unknown returned where a real type should be"

# 2. Grep for ALL instances across the ENTIRE codebase
rg "pattern_that_matches_broken_behavior" crates/
rg "alternate_pattern" crates/

# 3. Count and list every site — write them down

# 4. Fix ALL of them — not just the one that triggered the report

# 5. Document in pt fix arg 4:
#    "grepped for X in crates/, found N sites in files A+B+C, fixed all N"
#    OR: "single-site — verified no other callers: rg 'pattern' crates/ → 1 match"
```

**Scope expansion rules — if the bug exists in one place, CHECK THESE TOO:**
- Bug in `eval_expr`? → also check `eval_stmt`, `eval_value`, and all equivalent dispatch paths
- Namespace method returns wrong type? → audit ALL methods in that namespace block
- Stdlib function has unregistered param types? → audit ALL stdlib functions in that registrar
- Type X has wrong inference in context A? → verify context B, C, D also handle type X correctly

**`pt fix` REQUIRES a 4th scope audit arg — it will be REJECTED without 20+ chars explaining scope.**

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

## pt Gates — See CLAUDE.md "Mandatory pt Gates"

All universal gates (session start, pt next, pt decisions, pt claim, pt fix, pt phase-done, handoff, pt done) are defined in CLAUDE.md and apply here without exception.

**Bugfix-specific reminders:**
- Run `pt decisions <component>` before any fix touching internal architecture
- Discover a second bug mid-fix? `pt add` it immediately — 30 seconds now saves hours
- Include in issue: file ref, workaround used, fix risk for next agent
- Block tracking if this fix closes a phase: `pt phase-done B<N>-P<XX> "outcome"`

---

## When to Use Parity Tests

**Always** if the bug involves:
- VM/compiler behavior
- Stdlib functions
- Any runtime output

**Skip parity only for:** LSP, CLI, JIT, formatter, config (not compiler+VM pipeline)

---

## When Guardian Commit Fails

1. **Read the Guardian output** — it shows which phase failed (fmt or clippy)
2. **For fmt failures:** run `cargo fmt` then commit again
3. **For clippy failures:** fix the warnings shown, then commit again
4. `cargo fmt && git add -A && git commit ...` — try again
5. **Test failures appear in nightly CI, not pre-commit.** Run `pt ci-status` to see CI results.
6. If CI is failing: `pt run-ci` to get details, fix the specific failing tests.

---

## Failure Triage

If debugging exceeds 30 minutes on a single failure:
1. Identify root cause precisely
2. Document the blocker: `pt add "Title" P0 "what's blocking and why"`
3. Commit partial work with clear message
4. Next session picks it up via `pt go`

---

## Deeper Reference
- Parity practices: auto-memory `compiler-quality/parity.md`
- AI compiler lessons: auto-memory `compiler-quality/ai-compiler.md`
- Oracle testing: `gates/oracle-testing.md`
- Test partitioning: `gates/test-partitioning.md`
