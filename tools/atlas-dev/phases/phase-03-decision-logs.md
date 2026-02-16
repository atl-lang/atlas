# Phase 03: Decision Log Integration

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Phase 2 must be 100% complete

**Verification:**
```bash
atlas-dev phase complete --help  # Must show complete command
atlas-dev phase list  # Must return valid JSON
go test ./internal/db/... -v  # All Phase 2 tests must pass
```

**What's needed:**
- Phase 2 phase management system complete
- Database operations working correctly
- Audit logging functional
- Compact JSON output operational

**If missing:** Complete Phase 2 first

---

## Objective
Implement decision log management commands using pure SQLite - enabling AI agents to create, list, search, read, and update architectural decisions with auto-generated IDs, full-text search, and audit tracking - all with < 1ms query performance.

## Files
**Create:** `cmd/atlas-dev/decision.go` (~150 lines - decision command group)
**Create:** `cmd/atlas-dev/decision_create.go` (~200 lines - create command)
**Create:** `cmd/atlas-dev/decision_list.go` (~150 lines - list command)
**Create:** `cmd/atlas-dev/decision_search.go` (~100 lines - search command)
**Create:** `cmd/atlas-dev/decision_read.go` (~100 lines - read command)
**Create:** `cmd/atlas-dev/decision_update.go` (~150 lines - update command)
**Create:** `cmd/atlas-dev/decision_next_id.go` (~80 lines - next-id command)
**Create:** `cmd/atlas-dev/decision_export.go` (~150 lines - export command)
**Create:** `internal/db/decision.go` (~400 lines - decision DB operations)
**Update:** `cmd/atlas-dev/main.go` (add decision command group)

## Dependencies
- Phase 1 infrastructure (DB, transactions, JSON output)
- Phase 2 phase management (audit logging, validation)
- SQLite full-text search (FTS5 virtual table)
- All Phase 1-2 acceptance criteria met

## Implementation

### Decision Create Command
Implement decision create subcommand with flags: --component (required), --title (required), --decision (required), --rationale (required), --alternatives (optional), --status (default: accepted). Auto-generate sequential decision IDs in format DR-001, DR-002, etc by querying max ID from decisions table. Use WithTransaction to insert decision record with all fields, current timestamp, and auto-generated ID. Insert audit log entry recording decision creation. Return compact JSON with decision ID, component, and date. Validate component exists in categories table. **See ARCHITECTURE.md for transaction patterns.**

### Decision List Command
Implement decision list subcommand with optional --component and --status filters. Query decisions table with WHERE clauses for filters, order by date DESC. Support --limit flag for pagination (default: 20). Return array of decisions with: ID, component, title, date, status. Use prepared statements for query performance. Output compact JSON with abbreviated field names: id, comp, title, date, stat. Handle empty results gracefully.

### Decision Search Command
Implement decision search subcommand taking search query argument. Use SQLite LIKE operator to search across title, decision, and rationale fields. Query format: WHERE title LIKE ? OR decision LIKE ? OR rationale LIKE ? with query wrapped in % wildcards. Order results by date DESC, limit to 20. Return array of matching decisions with ID, component, title, date. Use indexed searches for performance. **Reference ARCHITECTURE.md for query patterns.**

### Decision Read Command
Implement decision read subcommand taking decision ID argument. Query decisions table for full decision details by ID using prepared statement. Return all fields: ID, component, title, decision, rationale, alternatives, date, status, created_at, updated_at. Output compact JSON omitting null fields. If decision not found, return structured error with exit code 2 and suggestion of similar IDs.

### Decision Update Command
Implement decision update subcommand taking decision ID argument with flags: --status (to update status), --superseded-by (to mark as superseded). Use WithTransaction to update decision record and insert audit log entry with old values for undo capability. Support status transitions: proposed → accepted/rejected, accepted → superseded. Validate status transitions are valid. Return compact JSON with updated decision ID and new status.

### Decision Next ID Command
Implement decision next-id subcommand to preview the next auto-generated ID. Query decisions table for maximum ID number: SELECT MAX(CAST(SUBSTR(id, 4) AS INTEGER)) FROM decisions. Return next ID in format DR-XXX with zero padding. Useful for AI agents to preview ID before creating decision. Return compact JSON with next_id field.

### Decision Export Command
Implement optional decision export subcommand to generate markdown files from database. Query all decisions grouped by component. Generate markdown file per component with decision records formatted as markdown tables. Export to docs/decisions/ directory. Include all decision fields, format alternatives as bullet lists. Return JSON with count of exported files and total decisions.

### Database Operations Layer
Implement all decision operations in internal/db/decision.go using struct-based DB pattern. Use prepared statements for all queries: getDecision, listDecisions, searchDecisions, createDecision, updateDecision. Implement GetNextDecisionID to query max ID and generate next. All operations must use transactions for consistency. Return Go structs that serialize to compact JSON. Handle concurrent decision creation with exclusive locks if needed.

## Tests (TDD)

**Decision create tests:**
1. Create decision with all fields
2. Auto-generate sequential IDs (DR-001, DR-002, DR-003)
3. Required fields validated (component, title)
4. Audit log entry created
5. Invalid component rejected
6. Duplicate ID handling
7. Compact JSON output
8. Transaction rollback on error

**Decision list tests:**
1. List all decisions
2. Filter by component works
3. Filter by status works
4. Combined filters work
5. Ordered by date DESC
6. Limit works (pagination)
7. Empty results handled
8. Compact JSON with abbreviated fields

**Decision search tests:**
1. Search by title finds matches
2. Search by decision text works
3. Search by rationale works
4. Search case-insensitive
5. Multiple results ordered by date
6. No matches returns empty array
7. Query < 10ms

**Decision read tests:**
1. Read decision by ID returns all fields
2. Not found returns error
3. Null fields omitted from JSON
4. Suggestion provided for similar IDs
5. Query < 1ms

**Decision update tests:**
1. Update status works
2. Mark as superseded works
3. Invalid status transition rejected
4. Audit log records old values
5. Transaction atomic
6. Not found handled

**Next ID tests:**
1. First ID is DR-001
2. Sequential IDs generated
3. Zero padding correct (DR-009, DR-010)
4. Concurrent ID generation safe

**Minimum test count:** 35 tests
**Coverage target:** 80%+ on internal/db/decision, cmd/atlas-dev/decision*

## Integration Points
- Uses: Database from Phase 1
- Uses: Transaction handling from Phase 1
- Uses: JSON output from Phase 1
- Uses: Audit logging from Phase 1
- Creates: Decision CRUD operations
- Creates: Auto-generated decision IDs
- Creates: Full-text decision search
- Creates: Decision export capability
- Output: Complete decision log system for tracking architectural choices

## Acceptance
- atlas-dev decision create works end-to-end
- Decision IDs auto-generated (DR-001, DR-002, etc)
- Sequential ID generation concurrent-safe
- atlas-dev decision list filters by component/status
- atlas-dev decision search performs full-text search
- Search results ordered by relevance/date
- atlas-dev decision read returns full details
- atlas-dev decision update changes status/supersedes
- atlas-dev decision next-id previews next ID
- Optional export generates markdown files
- All commands return compact JSON
- Null fields omitted from output
- Abbreviated field names used (comp, stat)
- Exit codes correct (0-6)
- Structured errors with suggestions
- 35+ tests pass
- 80%+ coverage on decision operations
- go test -race passes
- golangci-lint passes
- Benchmark: decision search < 10ms
- Benchmark: decision read < 1ms
- All SQL queries use prepared statements or transactions
- Audit log tracks all decision changes
- JSON output ~30-80 tokens (token-efficient)

**Phase complete when all acceptance criteria met and make test lint passes.**

---

**Note:** Reference ARCHITECTURE.md for all DB patterns. Use struct-based DB, prepared statements, and transaction handling. Follow TOKEN-EFFICIENCY.md for compact JSON output.
