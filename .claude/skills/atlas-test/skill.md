---
name: atlas-test
description: Atlas testing workflow. Use when writing tests, adding coverage, investigating test failures, organizing test domains, writing parity tests, or adding corpus tests. NOT for bug fixing (use atlas-bugfix) or battle testing (use atlas-battle).
---

# Atlas — Testing Workflow

**Prerequisite:** Core `atlas` skill must be active. If not, activate it first.

---

## AI Continuity — Non-Negotiable (100% AI-maintained project)

The user is architect only. You own all implementation, tracking, and continuity.

**Never narrate — act or file. These are the only two options:**
- ❌ "The next agent should add tests for X"
- ✅ `atlas-track add "Missing test coverage: X" P2 "what's untested, why it matters"` then move on.

**Proactive filing:** Find a gap in coverage? Missing corpus test? File it before moving on.

**Before restructuring test domains or changing test strategy — run the decision gate:**
```bash
atlas-track decisions infra   # test organization decisions live here
# 2 seconds. Prevents re-litigating already-decided test structure.
```
New test strategy decision not covered — log it: `atlas-track add-decision "Title" infra "Rule" "Rationale"`.

**Block tracking (if testing is part of a phase):**
```bash
atlas-track phase-done B<N>
atlas-track complete-block B<N> "tests added, coverage areas"  # final phase only
```

**Session close — write `.atlas-handoff.md` FIRST (MANDATORY):**
Write: what tests were added/fixed, coverage gaps found (file them if not already), next action. See core `atlas` skill for the full template. Commit it, then run `atlas-track done`.

---

## Two-Tier System (NEVER VIOLATE)

| Tier | When | What | Time |
|------|------|------|------|
| **1: Pre-commit** | Every commit (automatic) | `cargo fmt` + `cargo clippy` only | <15s |
| **2: Nightly CI** | 2am or `atlas-track run-ci` | Full nextest suite | ~20min |

**Banned during development — all nextest except one:**
```bash
cargo nextest run -p atlas-runtime -E 'test(exact_name)'  # ✅ TDD only, exact name
cargo nextest run --workspace                              # ❌ BANNED
cargo nextest run -p atlas-runtime                        # ❌ BANNED
```

**Fastest verification path (prefer this):**
```bash
atlas run /tmp/test.atlas    # instant — use before any nextest invocation
```

---

## Where Tests Live (lazy-load full table)

**Full domain → file mapping:** `.claude/rules/atlas-testing.md` (auto-loaded on test files)
**Full patterns:** `memory/testing-patterns.md`

**Quick reference — most common domains:**

| What you're testing | Where to add |
|---------------------|-------------|
| New language feature, end-to-end | `tests/corpus/pass/` (preferred) |
| Expected error/rejection | `tests/corpus/fail/` |
| Interpreter behavior | `tests/interpreter/integration/` |
| VM behavior | `tests/vm/integration.rs` |
| Parity (interpreter = VM) | `tests/bytecode/parity.rs` OR `assert_parity!` macro |
| Stdlib function | `tests/stdlib/` (pick matching subdomain) |
| Type inference | `tests/typesystem/inference/` |
| Async/futures | `tests/async_runtime/` |
| Regression (bug reproduction) | `tests/regression.rs` |
| Battle test (real-world program) | `battle-test/atlas-full-audit/domains/<domain>/` |

**Cardinal rule: never create a new test file.** Every new file = new binary = slower CI.
Add to existing domain file. Exception requires explicit approval.

**Size limit: 12KB max per test file.**
```bash
du -sh <target-file>   # check before touching
```
> 12KB = split first. See `atlas-testing.md` for split protocol.

---

## Preferred: Corpus Tests

For any new language behavior, write a corpus test — not a Rust test:

```bash
# Create test
echo 'let x = 42; print(str(x));' > crates/atlas-runtime/tests/corpus/pass/my_feature.atlas
echo '42' > crates/atlas-runtime/tests/corpus/pass/my_feature.stdout

# Generate expected output (runs both engines, auto-verifies parity):
UPDATE_CORPUS=1 cargo nextest run -p atlas-runtime --test corpus

# Verify
cargo nextest run -p atlas-runtime --test corpus -E 'test(my_feature)'
```

Corpus tests automatically verify interpreter/VM parity. Use them for:
- Feature coverage (new syntax, new builtins)
- Regression tests (a bug was fixed — prove it stays fixed)
- Documentation (readable Atlas programs showing correct behavior)

---

## Parity Tests

When implementing anything in interpreter or VM:

```rust
// In any test file — asserts identical output from both engines
assert_parity!(r#"
    let x = [1, 2, 3];
    print(str(len(x)));
"#, "3");

// Or use the helper directly:
fn assert_parity(source: &str, expected: &str) {
    let interp = run_interpreter(source);
    let vm = run_vm(source);
    assert_eq!(interp, vm, "parity divergence");
    assert_eq!(interp.trim(), expected.trim(), "wrong output");
}
```

**Full parity sweep (nightly CI handles this — don't run manually):**
```bash
cargo nextest run -p atlas-runtime -E 'test(parity)'   # ❌ run via CI only
```

---

## Battle Tests vs Unit Tests

| Use | When |
|-----|------|
| **Corpus test** | Single feature, controlled input/output |
| **Unit/integration test** | Internal Rust behavior, no Atlas source needed |
| **Battle test** | Real-world Atlas programs, multi-feature, full parity |

Battle test suite: `battle-test/atlas-full-audit/` — 47 programs, 10 domains.
```bash
bash battle-test/atlas-full-audit/run.sh   # validates all 47 programs
```

Add a new battle test when:
- A language feature is fully implemented (end-to-end, both engines)
- A bug was fixed that previously caused a real-world program to fail
- A new domain needs coverage (new struct in domain dir + Rust harness entry)

---

## Test Naming Conventions

```rust
// Rust tests — use issue number for regressions:
fn test_h110_enum_match_fn_body()  // regression for H-110
fn test_async_basic_await()        // feature coverage

// Corpus files:
tests/corpus/pass/async_basic.atlas         // feature name, snake_case
tests/corpus/fail/type_mismatch_result.atlas  // what fails, snake_case
```

---

## Deeper Reference
- Full domain table: `.claude/rules/atlas-testing.md` (auto-loads on test files)
- Full test patterns: `memory/testing-patterns.md`
- Parity contract: `.claude/rules/atlas-parity.md` (auto-loads on interpreter/VM/compiler files)
- Battle test strategy: `memory/compiler-quality/battle-testing.md`
- AI compiler lessons: `memory/compiler-quality/ai-compiler.md`
