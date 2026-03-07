---
name: atlas-battle
description: Atlas battle testing and validation. Use for regression testing, parity sweeps, validating real-world Atlas programs, and hardening the compiler.
---

# Atlas — Battle Testing & Validation

**Prerequisite:** Core `atlas` skill must be active. If not, activate it first.

---

## AI Continuity — Non-Negotiable (100% AI-maintained project)

The user is architect only. You own all implementation, tracking, and continuity.

**Never narrate — act or file. These are the only two options:**
- ❌ "The next agent should know that X pattern breaks Y"
- ❌ "Worth noting this battle test found Z"
- ✅ `pt add "Bug: X breaks Y" P1 "battle-test file ref, workaround, fix risk"` then move on.
Anything said to the user that isn't architecture = gone after session ends.

**Proactive filing during battle tests:** Every failure, workaround, and friction point found MUST be filed before moving to the next test. Include: the exact `.atlas` file path, the error code, the workaround used, and the fix risk. This is the data that drives language improvement.

**Full audit suite:** `battle-test/atlas-full-audit/` — 47 programs, 10 domains, interpreter+VM parity. Run: `bash battle-test/atlas-full-audit/run.sh`. Use as regression net before/after any typechecker or runtime fix.

**Before any architectural change triggered by battle test findings — run the decision gate:**
```bash
pt decisions <component>   # parser|typechecker|vm|interpreter|stdlib|runtime
# 3-8 lines. 2 seconds. A battle test failure may already have a standing fix decision.
```
Follow existing decisions. If findings expose a gap not covered — log it: `pt add-decision`.

**Block tracking (if battle test is the final phase of a block):**
```bash
pt phase-done B<N>
pt complete-block B<N> "47/47 pass, bugs filed H-XXX"  # final phase only
```

**Session close — write `~/.project-tracker/handoffs/atlas-handoff.md` FIRST (MANDATORY):**
Write: battle test results (pass rate, domains), bugs filed (H-XXX with file path), any patterns in failures, next action. See core `atlas` skill for the full template. Commit it, then run `pt done`.

---

## Battle Test Suite

**Location:** `battle-test/hydra-v2/`

```
battle-test/hydra-v2/
├── transport/transport.atlas
├── proxy/proxy.atlas
├── statestore/statestore.atlas
├── watcher/watcher.atlas
├── config/config.atlas
├── integration/integration.atlas
├── sanitizer/sanitizer.atlas
└── supervisor/supervisor.atlas
```

### Run All Battle Tests
```bash
for f in battle-test/hydra-v2/**/*.atlas; do
    echo "Testing: $f"
    atlas run "$f" 2>&1 || echo "BATTLE TEST FAILED: $f"
done
```

Any failure = regression. Fix before committing.

---

## Full Parity Sweep

```bash
# ALL parity tests across all 5 domains
cargo nextest run -p atlas-runtime -E 'test(parity)'
```

**Parity test files:**
- `tests/bytecode/parity.rs` — nested functions, strings, closures
- `tests/stdlib/parity.rs` — stdlib function behavior
- `tests/modules_cases/parity.rs` — module system
- `tests/interpreter/integration/parity_basic.rs` — basic operations
- `tests/typesystem/inference/parity_suite.rs` — type inference

---

## Full Validation Suite (GATE 6 equivalent)

**The pre-commit Guardian hook runs full tests + parity + fmt + clippy automatically on commit.**
You only need to run battle tests manually (Guardian doesn't run these):

```bash
# Battle tests (manual — not in Guardian hook)
for f in battle-test/hydra-v2/**/*.atlas; do
    atlas run "$f" 2>&1 || echo "FAILED: $f"
done

# Then commit — Guardian handles the rest
git commit
```

**NEVER run `cargo nextest run --workspace` manually.** The Guardian does it.

---

## When to Add New Battle Tests

- After implementing a major language feature (closures, traits, generics)
- After completing a block that changes runtime behavior
- When a real-world bug is found — add the failing program as a battle test

### Battle Test Design Rules
- Each test should exercise 3+ language features together
- Tests should be realistic programs, not contrived examples
- Must run successfully in both interpreter and VM

---

## Regression Investigation

When a battle test fails:
1. Identify which language features the test exercises
2. Run the specific `.atlas` file with `atlas run --verbose` (if available) or add debug prints
3. Check if it's a parity issue: run in interpreter-only and VM-only if possible
4. Write a minimal reproduction as a Rust test case
5. Fix via TDD (see `atlas-bugfix` skill)
6. Re-run full battle suite to confirm fix doesn't break others

---

## Oracle Verification

When a battle test fails and the expected output is unclear, use Rust/TypeScript as oracle:
See `gates/oracle-testing.md` for the oracle selection table and methodology.

---

## Deeper Reference
- Parity practices: auto-memory `compiler-quality/parity.md`
- Battle test strategy: auto-memory `compiler-quality/battle-testing.md`
- AI compiler lessons: auto-memory `compiler-quality/ai-compiler.md`
- Oracle testing: `gates/oracle-testing.md`
- Test partitioning: `gates/test-partitioning.md`
