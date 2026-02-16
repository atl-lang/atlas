# Phase 06: Polish & Advanced Features

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Phase 5 must be 100% complete

**Verification:**
```bash
atlas-dev context current  # Must return complete context
atlas-dev context phase <path>  # Must parse phase files
go test ./internal/context/... -v  # All Phase 5 tests must pass
```

**What's needed:**
- Phase 5 context system complete
- Phase file parsing working
- Context aggregation functional
- All core commands operational

**If missing:** Complete Phase 5 first

---

## Objective
Implement export, undo, backup/restore, and polish features for production readiness - enabling markdown generation for humans, transaction rollback for safety, disaster recovery, and operational reliability - completing the atlas-dev tool for real-world use.

## Files
**Create:** `cmd/atlas-dev/export.go` (~150 lines - export command group)
**Create:** `cmd/atlas-dev/export_markdown.go` (~100 lines - markdown export)
**Create:** `cmd/atlas-dev/export_json.go` (~100 lines - JSON export)
**Create:** `cmd/atlas-dev/undo.go` (~150 lines - undo command)
**Create:** `cmd/atlas-dev/backup.go` (~100 lines - backup command)
**Create:** `cmd/atlas-dev/restore.go` (~100 lines - restore command)
**Create:** `internal/export/markdown.go` (~300 lines - generate markdown from DB)
**Create:** `internal/export/json.go` (~150 lines - export DB to JSON)
**Create:** `internal/undo/undo.go` (~250 lines - undo logic using audit_log)
**Create:** `internal/undo/operations.go` (~200 lines - operation-specific undo handlers)
**Create:** `internal/backup/backup.go` (~150 lines - create backups)
**Create:** `internal/backup/restore.go` (~100 lines - restore from backups)
**Update:** `cmd/atlas-dev/main.go` (add export/undo/backup commands)

## Dependencies
- Phase 1 infrastructure (audit logging for undo)
- Phase 2 phase management (phase data)
- Phase 3 decision logs (decision data)
- Phase 4 analytics (progress data)
- Phase 5 context (for export enrichment)
- All Phase 1-5 acceptance criteria met

## Implementation

### Export to Markdown (OPTIONAL - For Humans Later)
**CRITICAL:** This is ON-DEMAND export for humans ONLY. AI agents NEVER use these files.

Implement export markdown subcommand to generate human-readable STATUS.md and tracker files from database. Query all categories and phases from database. Generate STATUS.md with: overall progress (X/Y phases, Z%), last completed phase, next phase, category breakdowns (name, completed/total, percentage, status). Generate tracker files in status/trackers/ directory, one per category, with checklist format: checkbox (✅ for completed, ⬜ for pending), phase name, and path. Format markdown with proper headings, lists, and tables. Use template-based generation for consistency. Return JSON with count of files generated and output directory.

**Important:**
- ✅ Generate on-demand: `atlas-dev export markdown` (when humans request it)
- ❌ NOT automatic (not generated on every phase complete)
- ❌ NOT source of truth (database is canonical)
- ❌ AI never reads these (AI queries database directly)
- Use case: User wants to see progress in GitHub/web browser

**See ARCHITECTURE.md for query patterns.**

### Export to JSON
Implement export json subcommand to create complete JSON backup of entire database. Query all tables: categories, phases, decisions, features, metadata, parity_checks, test_coverage, audit_log. For each table, extract all rows with column names. Build structured JSON with table name as key and array of row objects as value. Use json.Encoder with SetIndent for pretty-printed output. Write to file with timestamp in name (backup-YYYYMMDD-HHMMSS.json). Return JSON with backup file path and size. Useful for disaster recovery and migration.

### Undo Command
Implement undo command to rollback last operation using audit_log table. Query most recent audit_log entry ordered by timestamp DESC. Parse action type and old_data JSON. Based on action type, execute appropriate undo operation: phase_complete → restore previous status and clear completed_date, decision_create → delete the decision, decision_update → restore old values, feature_update → restore old values. Use WithTransaction to ensure atomic undo. After successful undo, delete the audit_log entry. Return JSON with action that was undone and restored data. Handle "nothing to undo" gracefully. **See ARCHITECTURE.md for transaction patterns.**

### Backup Command
Implement backup command to create timestamped copy of database file. Generate backup filename with timestamp: atlas-dev-YYYYMMDD-HHMMSS.db. Create .backups/ directory if not exists. Copy database file to backup location using io.Copy. Ensure database is not locked during copy - wait for exclusive access if needed. Return JSON with backup path and size in bytes. Support --auto flag to create backup before risky operations. Keep last N backups (configurable, default 10) and auto-delete old backups.

### Restore Command
Implement restore command taking backup file path argument. Verify backup file exists and is valid SQLite database. Create safety backup of current database before restore. Copy backup file to main database location, overwriting current. Require --confirm flag to prevent accidental data loss. Verify restored database integrity by running quick validation. Return JSON with restored backup path and timestamp. Handle errors gracefully with detailed messages. Restore should be all-or-nothing operation.

### Undo Operation Handlers
Implement operation-specific undo handlers in internal/undo/operations.go. Create handler for each operation type: UndoPhaseComplete (restore status, clear dates, recalculate progress), UndoDecisionCreate (delete decision record), UndoDecisionUpdate (restore old status/fields), UndoFeatureUpdate (restore old feature data). Each handler parses old_data JSON, validates it's safe to undo, executes reversing SQL in transaction. Return detailed undo result with what was changed. Handle edge cases: record already deleted, database state changed, invalid old_data.

### Markdown Generation Logic
Implement markdown generation in internal/export/markdown.go. Create GenerateStatusMD function that queries database and formats as markdown. Create GenerateTrackerFiles function that generates per-category tracker files. Use Go text/template for consistent formatting. Handle empty categories, zero progress, missing data gracefully. Format dates in human-readable format. Use markdown tables for category summaries. Ensure generated markdown is valid and properly formatted. Test output with markdown linter.

### Backup Management
Implement backup management in internal/backup/backup.go. Create ListBackups function to find all backup files in .backups/ directory. Implement CleanupOldBackups to remove backups beyond retention limit. Add GetBackupInfo to read metadata from backup files (size, date, schema version). Support backup compression (optional, gzip). Verify backup integrity by opening with SQLite and running PRAGMA integrity_check. Handle file permissions correctly.

## Tests (TDD)

**Export markdown tests:**
1. Generate STATUS.md with correct format
2. Overall progress calculated correctly
3. Category summaries included
4. Last/next phase shown
5. Tracker files created per category
6. Checkboxes correct (✅/⬜)
7. Empty categories handled
8. Output directory created

**Export JSON tests:**
1. All tables exported
2. All rows included
3. Column names preserved
4. JSON valid and well-formed
5. Pretty-printed output
6. File written correctly
7. Timestamp in filename

**Undo tests:**
1. Undo phase complete restores status
2. Undo decision create deletes record
3. Undo decision update restores old data
4. Nothing to undo handled
5. Transaction atomic
6. Audit log entry deleted after undo
7. Invalid old_data handled

**Backup tests:**
1. Backup file created with timestamp
2. .backups/ directory created
3. Database copied correctly
4. File size matches source
5. Backup valid SQLite database
6. Old backups cleaned up (>N)
7. Auto-backup before risky ops

**Restore tests:**
1. Restore from backup works
2. Current DB backed up before restore
3. --confirm flag required
4. Invalid backup rejected
5. Database integrity verified
6. Atomic operation
7. Error messages clear

**Integration tests:**
1. Complete operation → undo → verify
2. Backup → modify DB → restore → verify
3. Export markdown → check format
4. Export JSON → reimport → verify

**Minimum test count:** 40 tests
**Coverage target:** 80%+ on internal/export, internal/undo, internal/backup

## Integration Points
- Uses: Audit logging from Phase 1 (for undo)
- Uses: All database tables from Phases 1-5
- Uses: Transaction handling from Phase 1
- Creates: Export system (markdown and JSON)
- Creates: Undo system (rollback operations)
- Creates: Backup/restore system (disaster recovery)
- Creates: Production-ready polish features
- Output: Complete operational safety and human-readable exports

## Acceptance
- atlas-dev export markdown generates STATUS.md
- STATUS.md format correct and readable
- Category progress accurate in markdown
- Tracker files generated per category
- Checkboxes show completion status
- atlas-dev export json creates complete backup
- JSON export includes all tables and data
- Export file valid JSON
- atlas-dev undo rollsback last operation
- Undo phase complete restores previous state
- Undo is transactional (atomic)
- Audit log entry deleted after undo
- atlas-dev backup creates timestamped backup
- Backup file valid SQLite database
- Old backups auto-cleaned (keep last 10)
- atlas-dev restore restores from backup
- Restore requires --confirm flag
- Safety backup created before restore
- Database integrity verified after restore
- All commands return compact JSON
- Exit codes correct (0-6)
- Structured errors with suggestions
- 40+ tests pass
- 80%+ coverage on export/undo/backup
- go test -race passes
- golangci-lint passes
- Export markdown matches original format
- Undo operations are reversible
- Backup/restore tested for data integrity
- No data loss in backup/restore cycle

**Phase complete when all acceptance criteria met and make test lint passes.**

---

**Note:** Reference ARCHITECTURE.md for transaction patterns. Export markdown is optional (for humans) - AI never needs it. Undo and backup/restore are critical for operational safety.
