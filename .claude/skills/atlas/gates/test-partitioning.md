# Test Partitioning — Fast Feedback During Development (Lazy-Loaded)

**Load when:** Deciding what tests to run during development.

---

## The Two-Tier System (Single Source of Truth)

**See `testing-workflow.md` in auto-memory for the full rule set. This file is a quick reference.**

**Tier 1: Pre-commit (automatic, < 15s)** — fmt + clippy only
**Tier 2: Nightly CI (2am or `atlas-track run-ci`)** — full suite, parity, battle tests

---

## What Agents Do During Development

```bash
# Step 1: Does it compile? (~0.5s)
cargo check -p atlas-runtime

# Step 2: Write code

# Step 3: Does it still compile?
cargo check -p atlas-runtime

# Step 4: Format + commit
cargo fmt
git add <files>
git commit   # Guardian runs: fmt + clippy (< 15s)

# Step 5: Check CI status
atlas-track go   # shows nightly CI result
```

---

## TDD Exception (bugfix sessions only)

During TDD RED/GREEN cycles for a SINGLE exact test:

```bash
# RED: verify new test fails before fixing
cargo nextest run -p atlas-runtime -E 'test(exact_test_function_name)'

# GREEN: verify new test passes after fixing
cargo nextest run -p atlas-runtime -E 'test(exact_test_function_name)'

# Then commit — done. CI handles the rest.
```

Use the EXACT function name — not a domain keyword.

---

## BANNED Commands

These are BANNED for all agents except the TDD exception above:

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

## CI Failure Triage

If `atlas-track go` shows CI is red:

```bash
atlas-track ci-status    # see what failed
atlas-track run-ci       # re-run full suite to get fresh results
```

Fix the specific failing tests, then commit. CI re-runs nightly.

---

## Quick Reference

| Situation | Command |
|-----------|---------|
| Verify compile | `cargo check -p atlas-runtime` |
| TDD: run one test | `cargo nextest run -p atlas-runtime -E 'test(my_exact_test)'` |
| Pre-commit format | `cargo fmt` |
| Static validation | `git commit` (Guardian: fmt + clippy) |
| Full validation | nightly at 2am, or `atlas-track run-ci` |
| View CI results | `atlas-track ci-status` |
