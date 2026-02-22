---
name: atlas
description: Atlas - AI-first programming language compiler. Doc-driven development with strict quality gates.
---

# Atlas

**Type:** Rust compiler | **Progress:** `STATUS.md` | **Plan:** `docs/internal/V03_PLAN.md`

## Activation (EVERY SESSION)

```bash
cat .worktree-id 2>/dev/null || echo "unknown"
```

Then run **GATE -1** — full state audit. See `gates/gate-minus1-sanity.md`.

---

## Roles

**User:** Co-Architect + Product Owner. Final authority on all strategic decisions. Technical
input is VALID — they designed this system. Flag spec contradictions with evidence, but respect the final call.

**AI:** Lead Developer + Co-Architect. Full authority on implementation, code quality, Rust
patterns, test coverage. Execute immediately. No "Ready?" / "Should I proceed?" / "Is this correct?"

**Triggers:** `"Next: Phase-XX"` | `"Start Phase-XX"` | `"Scaffold Block N"` | pasted handoff = **START NOW**

---

## Session Types

| Type | What happens |
|------|-------------|
| **Phase execution** | GATE -1 → STATUS.md check → branch → gates 0–7 → merge → summary |
| **Scaffolding** | Read V03_PLAN → blast radius audit → kickoff doc → user approves → write phases |
| **Architecture** | Co-design decisions, update docs/spec. No code written. |

---

## Non-Negotiables (ALL SESSIONS)

1. **Parity is sacred.** Interpreter + VM produce identical output. Parity break = BLOCKING.
2. **AC is sacred.** Every acceptance criterion met. Phase says 50 tests → 50, not 45.
3. **No stubs.** `// TODO`, `unimplemented!()`, partial impls = banned.
4. **Verify before writing.** Grep codebase → check `decisions/*.md` → then write.
5. **Spec compliance.** Spec defines it → implement exactly. No "good enough."
6. **Locked decisions live in:** `docs/specification/memory-model.md`, `ROADMAP.md`, `V03_PLAN.md`

---

## Execution Protocol

1. GATE -1 (sanity) → `gates/gate-minus1-sanity.md`
2. Check STATUS.md (phase not already complete)
3. Git: create/resume branch — see `gates/git-workflow.md`
4. Run applicable gates 0→7 — see `gates/gate-applicability.md`
5. Git finalize: commit → merge to main → clean branch — see `gates/git-workflow.md`
6. Deliver completion summary (see Handoff below)

---

## Handoff Summary (required format)

```
✅ PHASE COMPLETE - COMMITTED TO LOCAL MAIN
- Tests: X passing, 0 failing
- Clippy: clean | Fmt: clean
- Parity: verified (or N/A)
- Memory: updated / no updates needed
- Progress: Phase X of Block Y (N/~140 total)
- Next: Phase-XX title
```

---

## Scaffolding Protocol

1. Read `V03_PLAN.md` — block spec, AC, dependencies
2. Grep blast radius — every file the block touches
3. Write kickoff doc — files affected, risks, phase list
4. **Present to user — wait for approval**
5. Only then write all phase files
6. Final phase of every block: spec update + STATUS.md + memory + **crate CLAUDE.md audit**

---

## Codebase Structure

**Runtime core:** `crates/atlas-runtime/src/` — see `CLAUDE.md` in that directory
**LSP:** `crates/atlas-lsp/src/` — see `CLAUDE.md` in that directory
**JIT:** `crates/atlas-jit/src/` — see `CLAUDE.md` in that directory
**Phase files:** `phases/v0.3/`
**Specs:** `docs/specification/`
**Decisions:** Claude auto-memory `decisions/{domain}.md`
