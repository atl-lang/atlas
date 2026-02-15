# Phase 06: Polish & Advanced Features (SQLite)

**Objective:** Export, undo, backup/restore.
**Priority:** LOW | **Depends On:** Phase 5

## Deliverables
- `export markdown` - Generate STATUS.md/trackers from DB (for humans later)
- `export json` - Export DB to JSON
- `undo` - Rollback last operation (using audit_log)
- `backup` - Create DB backup
- `restore` - Restore from backup

## Export to Markdown (Optional)
Generate markdown files from DB for human consumption:
- Query DB → build STATUS.md structure
- Query categories → build trackers/*.md
- Query decisions → build decision-logs/**/*.md

AI never reads these - just for humans/DocManager later.
