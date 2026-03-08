---
name: atlas-test
description: Atlas testing workflow. Use when writing tests, adding coverage, investigating test failures, organizing test domains, writing parity tests, or adding corpus tests. NOT for bug fixing (use atlas-bugfix) or battle testing (use atlas-battle).
---

# Atlas — Testing Workflow

**Prerequisite:** Core `atlas` skill must be active. If not, activate it first.

---

## pt Gates — See CLAUDE.md "Mandatory pt Gates"

All universal gates apply without exception. Testing-specific reminders:

- **Coverage gap found?** `pt add "Missing test coverage: X" P2 "what's untested, why it matters"` — immediately
- **Before restructuring test domains:** `pt decisions infra` (test organization decisions live here)
- **Block tracking:** `pt phase-done B<N>-P<XX> "outcome"` if testing is part of a phase
- **Handoff:** include what tests were added, gaps found and filed, next action

## Two-Tier System — See CLAUDE.md for full rules

**Fastest:** `atlas run /tmp/test.atlas` — always before nextest.
**Allowed nextest:** ONE exact test name only (TDD or corpus update).
**Banned:** all nextest invocations except those two cases.

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
