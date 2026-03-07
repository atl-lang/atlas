# GATE 7 — Memory File Size Limits (Lazy-Loaded)

**Load when:** File size check fails, or need to split/archive.

---

| File | Max Lines | If Exceeded |
|------|-----------|------------|
| MEMORY.md | 55 | Split content to topic files |
| patterns.md | 30 | This is an INDEX — content goes in `patterns/*.md` |
| patterns/*.md (each) | 80 | Split the topic file further |
| testing-patterns.md | 300 | Archive old → `archive/YYYY-MM-testing-patterns.md` |
| domain-prereqs.md | 100 | Archive old content |

---

## Memory Structure

```
memory/
├── MEMORY.md              # Index ONLY (pointers, not content)
├── patterns.md            # INDEX → points to patterns/*.md topic files
├── patterns/              # Topic files (runtime, collections, traits, etc.)
├── compiler-quality/      # Parity, battle testing, AI compiler lessons
├── domain-prereqs.md      # Domain verification checklist
├── testing-patterns.md    # Test patterns and conventions
└── archive/               # Old stuff goes here
```

Decisions go to `pt` (D-XXX), NOT memory files.

---

## How to Split

1. Create `archive/YYYY-MM-{file}.md` with old/stable content
2. Keep only actively-referenced content in the main file
3. Update MEMORY.md index if needed

---

## Rules

- **Surgical updates.** One-liner patterns, not paragraphs.
- **Verify before writing.** Confirm against codebase.
- **Archive, don't delete.** Move to `archive/YYYY-MM-{file}.md`.
