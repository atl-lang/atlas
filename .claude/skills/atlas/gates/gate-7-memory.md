# GATE 7: Memory Check (After Every Phase)

**Condition:** Phase complete, `atlas-track done` run, ready to commit

**Purpose:** Keep AI memory accurate and lean. Prevents drift and bloat.

---

## Quick Self-Check (10 seconds)

**Use the objective trigger, not the subjective one:**
- ❌ Subjective (wrong): "Did I make an architectural decision?"
- ✅ Objective (right): "Could a future agent, with no session context, make a different choice here?"

If yes → log it. The bar is low. Log it anyway.

1. **Did I hit an API surprise?** → Update `.claude/rules/atlas-*.md` AND `patterns.md`
2. **Could a future agent make a different choice about anything I did?** → Run `atlas-track add-decision`
3. **Did I discover a crate-specific pattern?** → Update `testing-patterns.md` or `patterns.md`
4. **Did I add/rename an AST node, Type variant, opcode, function name, or error code?** → Run `atlas-doc-auditor`
5. **Is this a block completion phase?** → **Always run `atlas-doc-auditor`** (full Tier 1+2+3 sweep)
6. **Closed-loop verification — did everything I created actually land?**
   ```bash
   # Rule files referenced in MEMORY.md must exist
   ls .claude/lazy/architecture.md .claude/rules/atlas-testing.md \
      .claude/rules/atlas-parity.md 2>&1 | grep "No such file"
   # decisions/workflow.md must exist
   ls /Users/proxikal/.claude/projects/-Users-proxikal-dev-projects-atlas/memory/patterns.md 2>&1 | grep "No such file"
   # MEMORY.md within limit
   wc -l /Users/proxikal/.claude/projects/-Users-proxikal-dev-projects-atlas/memory/MEMORY.md
   ```
   Any missing file → create it before committing. MEMORY.md > 55 → split before committing.
7. **Staleness greps:**
   ```bash
   grep -c "pull_request" .github/workflows/ci.yml            # must return ≥1
   grep -c "rebase origin/main" .claude/skills/atlas/gates/git-workflow.md  # must return ≥1
   grep -c "\.claude" .github/workflows/ci.yml                 # must return ≥1
   ```
8. **Is any source file approaching size limit?** → Flag in summary, split if blocking

## When to Update Memory

### ✅ DO Update Memory

**patterns.md:**
- Hit an undocumented API quirk that cost time
- Discovered a crate-specific testing pattern
- Found a common error pattern with a fix
- Learned a Rust pattern that's Atlas-specific

**Example:** "LSP tests can't use helper functions due to lifetime issues"

**atlas-track (decisions):**
- Made an architectural choice between alternatives
- Established a new constraint or rule
- Chose an approach that affects future work
- Resolved an ambiguity in the spec

**Example:** `atlas-track add-decision "LSP testing uses inline pattern" "No helpers due to tower-lsp lifetime constraints"` → D-XXX

### ❌ DON'T Update Memory

**Skip if:**
- Just following existing patterns (already documented)
- Phase-specific work (not reusable knowledge)
- Obvious or trivial changes
- Implementation details (not architectural)
- Temporary workarounds

**Example:** Don't document "Created 10 integration tests" (obvious, not reusable)

---

## File Size Limits (BLOCKING)

**Before committing, run this check:**
```bash
wc -l /Users/proxikal/.claude/projects/-Users-proxikal-dev-projects-atlas/memory/*.md 2>/dev/null | grep -v total
```

| File | Max | If Exceeded → MUST DO |
|------|-----|----------------------|
| MEMORY.md | 55 | Split content to topic files |
| patterns.md | 150 | Archive old → `archive/YYYY-MM-patterns.md` |
| testing-patterns.md | 300 | Archive old → `archive/YYYY-MM-testing-patterns.md` |
| domain-prereqs.md | 100 | Archive old content |

**BLOCKING:** If ANY file exceeds limit, you MUST split/archive BEFORE committing.
**NO EXCEPTIONS.** This is not optional. Bloated memory = wasted tokens every message.

---

## Memory Structure

```
memory/
├── MEMORY.md           # Index ONLY (pointers, not content)
├── patterns.md         # All implementation patterns (consolidated)
├── domain-prereqs.md   # Domain verification checklist
├── testing-patterns.md # Test patterns and conventions
└── archive/            # Old stuff goes here

# Decisions go to atlas-track (D-XXX), NOT memory files
```

---

## How to Split patterns.md

When `patterns.md` exceeds 150 lines:
1. Create `archive/YYYY-MM-patterns.md` with old/stable patterns
2. Keep only actively-referenced patterns in `patterns.md`
3. Update MEMORY.md index if needed

**Example split:**
- `patterns.md` → Active patterns (runtime, stdlib, testing)
- `archive/2026-02-patterns.md` → Stable patterns (frontend API, error handling)

---

## Rules

- **Surgical updates.** One-liner patterns, not paragraphs.
- **Verify before writing.** Confirm against codebase.
- **Archive, don't delete.** Move to `archive/YYYY-MM-{file}.md`.
- **Decisions → atlas-track.** Run `atlas-track add-decision`, NOT memory files.
- **Patterns → memory.** New domain patterns go in `patterns.md`.

---

## Required Output (MANDATORY)

**In completion summary, include Memory section:**

```markdown
### Memory
- Updated: `patterns.md` (added X)
- Decision: `atlas-track add-decision` → D-XXX (reason)
- Archived: `patterns.md` → `archive/2026-02-patterns-v1.md`
```

OR if no updates:

```markdown
### Memory
- No updates needed
```

**This is NOT optional.** Visible accountability prevents drift.

---

**Next:** Report completion summary with Memory section. See `gates/git-workflow.md` for commit/PR commands.
