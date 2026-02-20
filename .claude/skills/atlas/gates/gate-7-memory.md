# GATE 7: Memory Check (After Every Phase)

**Condition:** Phase complete, STATUS.md updated, ready to commit

**Purpose:** Keep AI memory accurate and lean. Prevents drift and bloat.

---

## Quick Self-Check (10 seconds)

1. **Did I hit an API surprise?** → Update `patterns.md`
2. **Did I make an architectural decision?** → Update `decisions/{domain}.md`
3. **Is anything in memory wrong?** → Fix or archive it
4. **Is any file approaching size limit?** → Split or archive

---

## File Size Limits (BLOCKING)

**Before committing, run this check:**
```bash
wc -l ~/.claude/projects/*/memory/*.md ~/.claude/projects/*/memory/decisions/*.md 2>/dev/null | grep -v total
```

| File | Max | If Exceeded → MUST DO |
|------|-----|----------------------|
| MEMORY.md | 50 | Split content to topic files |
| patterns.md | 150 | Archive old → `archive/YYYY-MM-patterns.md` |
| decisions/{x}.md | 100 | Split into sub-files |

**BLOCKING:** If ANY file exceeds limit, you MUST split/archive BEFORE committing.
**NO EXCEPTIONS.** This is not optional. Bloated memory = wasted tokens every message.

---

## Memory Structure

```
memory/
├── MEMORY.md           # Index ONLY (pointers, not content)
├── patterns.md         # Active patterns
├── decisions/          # Split by domain
│   ├── language.md
│   ├── runtime.md
│   ├── stdlib.md
│   ├── cli.md          # CRITICAL decisions here
│   ├── typechecker.md
│   ├── vm.md
│   └── {new-domain}.md # Add as needed
└── archive/            # Old stuff goes here
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
- **Split by domain.** New domain = new file in `decisions/`.

---

## Required Output (MANDATORY)

**In completion summary, include Memory section:**

```markdown
### Memory
- Updated: `patterns.md` (added X)
- Updated: `decisions/cli.md` (DR-003: reason)
- Archived: `patterns.md` → `archive/2026-02-patterns-v1.md`
```

OR if no updates:

```markdown
### Memory
- No updates needed
```

**This is NOT optional.** Visible accountability prevents drift.

---

## Git Finalization (After Memory Check)

1. `git add -A && git commit -m "feat(category): description"`
2. `git push -u origin HEAD && gh pr create && gh pr merge --squash --auto`
3. Walk away - automation handles merge
4. Next session syncs main automatically

**Next:** Report completion summary with Memory section.
