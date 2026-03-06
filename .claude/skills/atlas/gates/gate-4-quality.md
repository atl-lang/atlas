# GATE 4: Quality Gates

**Condition:** Implementation complete, parity verified

---

## Action

1. **Run clippy:**
   ```bash
   cargo clippy --workspace -- -D warnings
   ```
   **MUST:** Zero warnings

2. **Run formatter:**
   ```bash
   cargo fmt --check
   ```
   **MUST:** All files formatted

3. **Run CodeRabbit review (local CLI):**
   ```bash
   coderabbit review
   ```
   Reviews staged/changed code against `.coderabbit.yaml` rules (parity enforcement, no stubs, no bare TODOs, diagnostic code validation). Fix any issues flagged before proceeding.

---

**Note:** Run crate-scoped tests here, not full workspace. See `gates/test-partitioning.md` for scope rules. Full suite is GATE 6 only.

---

**BLOCKING:** Both must pass. No exceptions.

---

## Decision

- All pass → GATE 5
- Any fail → Fix → Retry

---

**Next:** GATE 5
