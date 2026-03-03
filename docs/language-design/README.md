# Atlas Language Design

**Purpose:** Canonical design authority for Atlas grammar and syntax.
**Status:** Authoritative — all grammar changes must align with these documents.

---

## For AI Agents

**Single Source of Truth:** `atlas-track decisions` — run this FIRST.

**Supporting documentation (read after checking atlas-track):**

1. `PRINCIPLES.md` — The design rules. Non-negotiable.
2. `ANTI-PATTERNS.md` — Mistakes made in v0.1-v0.2. Never repeat.
3. `rationale/` — Extended rationale and migration guidance for decisions.

**Do not:**
- Add syntax without creating a decision in `atlas-track` first
- Justify changes as "AI-first" without citing specific principles
- Add "deprecated but supported" features
- Create multiple paths to the same outcome
- Treat rationale/ or patterns/ as authoritative over atlas-track

---

## Document Index

| File | Purpose |
|------|---------|
| `PRINCIPLES.md` | Core design principles (derived from PRD) |
| `ANTI-PATTERNS.md` | What NOT to do (learned from v0.2 mistakes) |
| `rationale/` | Extended rationale for decisions (D-XXX format, references atlas-track) |
| `migration/` | How to update code for breaking changes |

---

## Decision Records

**Source of truth:** `atlas-track decisions`

| ID | Title | Rationale File |
|----|-------|----------------|
| D-006 | Variable Syntax (`let`/`let mut`, remove `var`) | `rationale/D-006-variable-syntax.md` |
| D-007 | Loop Syntax (remove C-style for, `++`/`--`) | `rationale/D-007-loop-syntax.md` |
| D-008 | Function Syntax (remove arrow functions) | `rationale/D-008-function-syntax.md` |
| D-009 | Struct/Object Literals (`record` keyword) | `rationale/D-009-struct-object-literals.md` |
| D-010 | Type::Unknown Semantics (error state, not wildcard) | `rationale/D-010-type-unknown-semantics.md` |

---

## Adding New Decisions

1. **Create decision in atlas-track:** `atlas-track add-decision "Title" comp "Rule" "Rationale"`
2. Get the assigned D-XXX ID
3. **If decision has enforceable syntax** → Add pattern to `~/.claude/hooks/atlas/decision-patterns.json`
4. Create `rationale/D-XXX-short-name.md` with extended rationale
5. Add header referencing atlas-track as source of truth
6. Update this README index

**Step 3 is MANDATORY for syntax/grammar decisions.** The Guardian hook blocks violations automatically.

---

## Relationship to Other Docs

| Document | Role |
|----------|------|
| **This folder** | DESIGN authority — what SHOULD exist |
| `docs/specification/` | IMPLEMENTATION tracking — what DOES exist |
| `docs/internal/PRD.md` | SOURCE of principles |
| `CLAUDE.md` | AI agent operating instructions |

Design docs guide implementation. Spec docs describe implementation.
They are not the same.

---

## Decision System

**`atlas-track` is THE single source of truth for all decisions.**

Supporting documentation exists but is NOT authoritative:

| Location | Format | Purpose |
|----------|--------|---------|
| `atlas-track decisions` | D-XXX | **AUTHORITATIVE** — decision status, summary |
| `docs/language-design/rationale/` | D-XXX-name.md | Extended rationale, migration guides |
| `.claude/memory/patterns/` | P-XXX | Implementation patterns, code examples |

### Workflow

```
1. atlas-track decisions     ← Check what's decided (START HERE)
2. rationale/D-XXX-*.md      ← Read extended context if needed
3. patterns/*.md             ← Check implementation patterns
```

### Creating Decisions

```bash
# Step 1: Create in atlas-track (gets D-XXX ID)
atlas-track add-decision "Title" "Description"

# Step 2: Optionally add extended rationale
# Create rationale/D-XXX-short-name.md with header:
# > **Tracking:** `atlas-track decision D-XXX` — source of truth
```

### Rules

- **Never** create rationale/ or patterns/ entries without a corresponding atlas-track decision
- **Always** check `atlas-track decisions` before reading supporting docs
- **If conflicts exist:** atlas-track wins, update the supporting doc

---

## v0.3 Grammar Rewrite Status

These decisions define the v0.3 grammar cleanup:

- [ ] D-006: Remove `var` keyword
- [ ] D-007: Remove C-style for, `++`/`--`
- [ ] D-008: Remove arrow functions
- [ ] D-009: Disambiguate `{}` with `record` keyword
- [ ] D-010: Fix `Type::Unknown` semantics

**Check status:** `atlas-track decisions` | **Implementation:** `STATUS.md`
