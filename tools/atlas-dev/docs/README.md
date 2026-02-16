# Atlas-Dev Documentation

**All reference documentation for atlas-dev tool.**

---

## Structure

```
docs/
├── README.md (this file)
├── schema/
│   └── DATABASE-SCHEMA.md    # SQL DDL reference (8 tables, indexes, triggers)
├── guides/
│   ├── AI-OPTIMIZATION.md    # AI usage guide (post-migration)
│   ├── TOKEN-EFFICIENCY.md   # Output standards (compact JSON)
│   └── WHEN-TO-USE.md        # Decision tree for AI
└── migration/
    └── MIGRATION.md          # One-time bootstrap (STATUS.md → SQLite)
```

---

## Quick Reference

### For Implementation
- **Start here:** `/ARCHITECTURE.md` (canonical patterns)
- **Then read:** `/phases/phase-01-core-infrastructure.md`
- **Schema reference:** `schema/DATABASE-SCHEMA.md`

### For AI Agents (Post-Migration)
- **Usage guide:** `guides/AI-OPTIMIZATION.md`
- **Output format:** `guides/TOKEN-EFFICIENCY.md`
- **Decision tree:** `guides/WHEN-TO-USE.md`

### For Migration
- **One-time only:** `migration/MIGRATION.md`
- **After migration:** Delete STATUS.md, trackers/*.md (no longer needed)

---

## Important Notes

### Pure SQLite Architecture
- **Database = source of truth** - All tracking in `atlas-dev.db`
- **Phase files = instructions** - Never change, tell AI what to build
- **No markdown tracking** - No STATUS.md, no trackers/, no decision-logs/
- **SQL only** - All updates via CLI → database (indexed, < 1ms)

### One-Time Migration
- ✅ Run once: `atlas-dev migrate bootstrap`
- ✅ Populates database from STATUS.md
- ✅ Backs up markdown files
- ❌ Never run again - SQL only after migration

### No File Caching
- Database IS the cache (indexed SQL)
- No phase-index.json, decision-index.json, etc.
- All queries < 10ms (prepared statements)
- AI guides assume post-migration (pure SQL)

---

## Files Overview

| File | Purpose | When to Read |
|------|---------|--------------|
| `schema/DATABASE-SCHEMA.md` | SQL DDL reference | Phase 1 implementation |
| `guides/AI-OPTIMIZATION.md` | AI usage patterns | After migration |
| `guides/TOKEN-EFFICIENCY.md` | Output standards | Phase 1-2 implementation |
| `guides/WHEN-TO-USE.md` | Decision tree | When unsure about approach |
| `migration/MIGRATION.md` | Bootstrap guide | Before Phase 1 (one-time) |

---

**See `/ARCHITECTURE.md` for implementation patterns.**
**See `/phases/` for phase-by-phase implementation.**
