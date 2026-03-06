# Test Partitioning — Fast Feedback During Development (Lazy-Loaded)

**Load when:** Deciding what tests to run during development.

---

## The Rule (Single Source of Truth)

**See `testing-workflow.md` in auto-memory for the full rule set. This file is a quick reference.**

**`cargo check` is the development tool. `git commit` triggers Guardian for real validation.**

---

## What To Run (and When)

### During TDD (write code → verify)

```bash
# Step 1: Does it compile? (~0.5s)
cargo check -p atlas-runtime

# Step 2: Does my specific test pass?
cargo nextest run -p atlas-runtime -E 'test(exact_test_function_name)'
# Use the EXACT function name — not a domain keyword
```

### Before commit

```bash
cargo fmt    # Run formatter — NOT --check, just fix it
git add <files>
git commit   # Guardian runs: fmt + clippy + targeted suite + parity + battle tests
```

### If commit fails (Guardian shows which test failed)

```bash
# Fix the specific failing test, then:
cargo fmt && git add -A && git commit
# Do NOT run broad tests to "verify" — Guardian does that on next commit
```

---

## BANNED Commands (cause 5-20 minute hangs)

These force cargo to compile ALL test binaries before running any:

```bash
# ❌ NEVER:
cargo nextest run -p atlas-runtime -E 'test(interpreter)'
cargo nextest run -p atlas-runtime -E 'test(regression)'
cargo nextest run -p atlas-runtime -E 'test(stdlib)'
cargo nextest run -p atlas-runtime -E 'test(corpus)'
cargo nextest run -p atlas-runtime -E 'test(frontend)'
cargo nextest run -p atlas-runtime -E 'test(type)'
cargo nextest run -p atlas-runtime -E 'test(vm)'
cargo nextest run -p atlas-runtime              # entire crate
cargo nextest run --workspace                   # entire workspace
cargo nextest run -p atlas-runtime --test <any_domain_file>
```

---

## One-Time Diagnostic (if Guardian output is unclear)

If a commit fails and you can't tell which test from the Guardian output:

```bash
# Run ONCE to find the failing test name:
cargo nextest run -p atlas-runtime -E 'test(regression) + test(interpreter)' --no-fail-fast 2>&1 | grep "^        FAIL"
# Then fix THAT specific test, then commit again
```

---

## Quick Reference

| Situation | Command |
|-----------|---------|
| Verify compile | `cargo check -p atlas-runtime` |
| TDD: run one test | `cargo nextest run -p atlas-runtime -E 'test(my_exact_test)'` |
| Pre-commit format | `cargo fmt` |
| Full validation | `git commit` (Guardian runs everything) |
| Parity sweep only | `cargo nextest run -p atlas-runtime -E 'test(parity)'` |
