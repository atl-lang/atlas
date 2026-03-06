---
name: atlas-battle
description: Atlas battle testing and validation. Use for regression testing, parity sweeps, validating real-world Atlas programs, and hardening the compiler.
---

# Atlas — Battle Testing & Validation

**Prerequisite:** Core `atlas` skill must be active. If not, activate it first.

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

Run this as the complete validation:

```bash
# 1. Build
cargo build --workspace

# 2. Full test suite
cargo nextest run --workspace

# 3. Parity sweep
cargo nextest run -p atlas-runtime -E 'test(parity)'

# 4. Quality
cargo clippy --workspace -- -D warnings
cargo fmt --check
coderabbit review

# 5. Battle tests
for f in battle-test/hydra-v2/**/*.atlas; do
    atlas run "$f" 2>&1 || echo "FAILED: $f"
done
```

**All must pass. No exceptions.**

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
