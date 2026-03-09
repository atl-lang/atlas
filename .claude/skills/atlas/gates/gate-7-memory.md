# GATE 7: Memory Check (After Every Phase)

**Condition:** Phase complete, ready to commit and close session

---

## Quick Self-Check (10 seconds)

**Objective trigger:** "Could a future agent, with no session context, make a different choice here?" If yes → log it.

1. **API surprise?** → Update `.claude/rules/atlas-*.md` AND `patterns.md`
2. **Future agent might choose differently?** → `pt add-decision`
3. **Crate-specific pattern?** → Update `testing-patterns.md` or `patterns.md`
4. **Added/renamed AST node, Type variant, opcode, function, error code?** → Run `atlas-doc-auditor`
5. **Block completion phase?** → **Always run `atlas-doc-auditor`** (full sweep)
6. **ALWAYS — close session with `pt done S-XXX success "summary" "next action"` — next agent sees it in `pt go`**

---

## Closed-Loop Verification

```bash
# Rule files referenced in MEMORY.md must exist
ls .claude/lazy/architecture.md .claude/rules/atlas-testing.md \
   .claude/rules/atlas-parity.md 2>&1 | grep "No such file"
# MEMORY.md within limit
wc -l /Users/proxikal/.claude/projects/-Users-proxikal-dev-projects-atlas/memory/MEMORY.md
```

Any missing file → create before committing. MEMORY.md > 55 → split before committing.

---

## When to Update

| ✅ DO | ❌ DON'T |
|-------|---------|
| Undocumented API quirk that cost time | Following existing patterns |
| Crate-specific testing pattern | Phase-specific one-time work |
| Architectural choice between alternatives | Obvious/trivial changes |
| New constraint or rule | Implementation details |
| Ambiguity resolved in spec | Temporary workarounds |

**Decisions → `pt add-decision`**, NOT memory files.
**Patterns → `patterns.md`** (index → `patterns/*.md` topic files).

---

## File Size Limits (BLOCKING — lazy-load `gates/gate-7-limits.md` for full table)

Quick check:
```bash
wc -l /Users/proxikal/.claude/projects/-Users-proxikal-dev-projects-atlas/memory/*.md 2>/dev/null | grep -v total
```

MEMORY.md ≤ 55 | patterns.md ≤ 30 (index only) | topic files ≤ 80 each

**If exceeded → MUST split/archive before committing. NO EXCEPTIONS.**

---

## Required Output (MANDATORY)

```markdown
### Memory
- Updated: `patterns.md` (added X)
- Decision: `pt add-decision` → D-XXX (reason)
```
OR: `- No updates needed`

**This is NOT optional.** Visible accountability prevents drift.

---

**Next:** Commit. See `gates/git-workflow.md`.
