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

## File Size Limits (ENFORCED)

| File | Max | If Exceeded |
|------|-----|-------------|
| MEMORY.md | 50 lines | Move to topic files |
| patterns.md | 150 lines | Archive old patterns |
| decisions/{x}.md | 100 lines | Split or archive |

**Check sizes:** `wc -l ~/.claude/projects/*/memory/*.md`

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
