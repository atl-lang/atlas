# Test Partitioning — Fast Feedback During Development (Lazy-Loaded)

**Load when:** Running tests during GATE 2 implementation or GATE 4 quality checks.

---

## Problem

Atlas has 7000+ tests. Running `cargo nextest run --workspace` takes minutes.
During development, you need fast feedback — not full suite.

---

## Strategy: Progressive Test Scope

### During Implementation (GATE 2) — TARGETED ONLY

```bash
# Single test
cargo nextest run -p atlas-runtime -E 'test(exact_test_name)'

# Domain file (e.g., all string tests)
cargo nextest run -p atlas-runtime --test strings

# Related parity tests only
cargo nextest run -p atlas-runtime -E 'test(parity)' --test bytecode_parity
```

**Rule:** Never run full workspace during GATE 2. Test only what you're touching.

### During Quality Check (GATE 4) — PARTITIONED

```bash
# Run tests for ONLY the crate you modified
cargo nextest run -p <modified-crate>

# If touching multiple crates, run each
cargo nextest run -p atlas-runtime -p atlas-cli
```

**Rule:** Crate-scoped, not workspace-scoped. Full workspace is GATE 6 only.

### During Final Testing (GATE 6) — FULL SUITE

```bash
cargo nextest run --workspace                    # Everything
cargo nextest run -p atlas-runtime -E 'test(parity)'  # Parity sweep
```

**This is the ONLY gate where full workspace runs.**

---

## Partitioning for Parallel Agents

If multiple agents work on the same codebase (future):

```bash
# Agent 1: first half of tests
cargo nextest run --workspace --partition count:1/2

# Agent 2: second half
cargo nextest run --workspace --partition count:2/2
```

Deterministic partitioning — each agent covers different tests, full coverage across all agents.

---

## Quick Reference

| Gate | Scope | Command pattern |
|------|-------|----------------|
| GATE 2 | Single test / domain file | `-E 'test(name)'` or `--test file` |
| GATE 4 | Modified crate(s) | `-p <crate>` |
| GATE 6 | Full workspace | `--workspace` |
| Parallel | Partition | `--partition count:N/M` |
