# GATE 0 — Dependency Check (Block/Phase Work Only)

**Load when:** Running `atlas-blocks` skill (structured phase development)

---

## For EACH dependency in phase file:
1. Does it exist in codebase? (grep for implementation)
2. Does it match spec? (compare to `docs/language/` or `docs/stdlib/`)
3. Is it complete? (`atlas-track block B<N>` shows block AC; grep tests for coverage)

**Before implementing anything:** Search for similar existing code. Follow established patterns. Check auto-memory `patterns.md` for constraints.

**Status per dependency:**
- ✅ Exists, complete, spec-compliant → Proceed
- ⚠️ Exists but incomplete → Flag, may need to finish first
- 🚫 Doesn't exist → BLOCKING. Do NOT ask the user. Resolve autonomously:
  - Missing spec definition: check `docs/language/` or `docs/stdlib/` — spec is authority
  - Missing implementation that should exist: implement as prerequisite phase (commit, then continue)
  - Dependency outside v0.3 scope: `atlas-track add "Gap: X" P1 "reason"` to document, skip phase noting why
  - User is never the answer to a missing dependency
