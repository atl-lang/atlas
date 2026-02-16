# Phase 02: Phase Management System

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Phase 1 must be 100% complete

**Verification:**
```bash
atlas-dev version  # Must return valid JSON
atlas-dev migrate schema  # Must create all tables
go test ./internal/db/... -v  # All Phase 1 tests must pass
```

**What's needed:**
- Phase 1 database infrastructure complete
- Struct-based DB with prepared statements working
- Transaction handling correct
- Compact JSON output working

**If missing:** Complete Phase 1 first

---

## Objective
Implement core phase tracking commands using pure SQLite - enabling AI agents to mark phases complete, query current/next phases, list phases by category/status, and validate database consistency - all with atomic transactions and < 1ms query performance.

## Files
**Create:** `cmd/atlas-dev/phase.go` (~150 lines - phase command group)
**Create:** `cmd/atlas-dev/phase_complete.go` (~200 lines - complete command)
**Create:** `cmd/atlas-dev/phase_current.go` (~100 lines - current command)
**Create:** `cmd/atlas-dev/phase_next.go` (~150 lines - next command)
**Create:** `cmd/atlas-dev/phase_info.go` (~100 lines - info command)
**Create:** `cmd/atlas-dev/phase_list.go` (~150 lines - list command)
**Create:** `cmd/atlas-dev/validate.go` (~200 lines - validate command)
**Create:** `internal/db/phase.go` (~400 lines - phase DB operations)
**Create:** `internal/db/validate.go` (~200 lines - validation logic)
**Create:** `internal/git/commit.go` (~150 lines - git integration)
**Create:** `internal/phase/parser.go` (~100 lines - parse phase path)
**Update:** `cmd/atlas-dev/main.go` (add phase command group)

## Dependencies
- Phase 1 infrastructure (DB, transactions, JSON output)
- Git installed and configured
- All Phase 1 acceptance criteria met

## Implementation

### Phase Complete Command
Implement phase complete subcommand with flags: --desc (required), --date (optional, default today), --commit (optional), --dry-run (optional), --tests (optional test count). Parse phase path to extract category and name. Validate phase exists in database. Check phase not already completed. Use WithExclusiveLock + WithTransaction for atomic update. Update phase status to completed, set completed_date, description, test_count. SQL triggers automatically update category progress and metadata. Insert audit log entry with changes JSON. Query updated category and total progress for response. Find next pending phase in same category. If --commit flag, create git commit with message format: "Complete {phase} - {description}". Return compact JSON with: ok=true, phase name, category, progress (category and total as arrays), next phase, modified files, optional commit SHA. **See ARCHITECTURE.md for DB patterns.**

### Phase Current Command
Implement phase current subcommand to show last completed phase. Query database for most recent completed phase ordered by completed_date DESC. Return compact JSON with: phase path, name, category, completed date, description, test count. If no phases completed yet, return ok=true with msg="No phases completed yet". Use prepared statement for < 1ms performance. **Follow TOKEN-EFFICIENCY.md for compact output.**

### Phase Next Command
Implement phase next subcommand with optional --category flag. If category specified, find first pending phase in that category ordered by ID. If no category specified, find next phase in category of last completed phase. If no phases pending in category, check other categories. Return compact JSON with: phase path, name, category, description, dependencies, test target, acceptance criteria. If all phases complete, return msg="All phases complete". Query using prepared statement with category filter.

### Phase Info Command
Implement phase info subcommand taking phase path argument. Query database for phase by path using prepared statement. Return all phase details: ID, path, name, category, status, completed_date (if completed), description, test_count, test_target, dependencies, blockers, files_modified. Return compact JSON omitting null fields. If phase not found, return error with suggestion of similar phase paths.

### Phase List Command
Implement phase list subcommand with optional --category and --status filters. Query phases table with WHERE clauses for filters. Order by category, then ID. Return array of phases with: path, name, category, status, completed_date (if completed). Use compact JSON format with abbreviated fields. Support pagination with --limit and --offset flags for large result sets.

### Validate Command
Implement validate subcommand to check database consistency. Verify category completed counts match actual phase counts (count phases where status=completed and category=X). Verify category percentages calculated correctly (completed/total*100). Verify total_phases metadata matches actual phase count. Verify completed_phases metadata matches actual completed count. Check for orphaned phases (category not in categories table). Check for invalid statuses (not pending/in_progress/completed/blocked). Check all triggers exist and are enabled. Return validation results as JSON with: ok (true if all checks pass), issues array (empty if valid), checks_run count, errors_found count. If validation fails, include details of each issue with fix suggestions.

### Git Integration
Implement git commit helper in internal/git/commit.go. Verify inside git repository (check .git directory). Check git configured (user.name and user.email set). Stage database file (git add atlas-dev.db). Create commit with message format: "Complete {phase-name} - {description}". Return commit SHA for inclusion in response JSON. Handle errors gracefully (not in git repo, nothing to commit, git command failed). Use file locking to prevent concurrent git operations.

### Phase Path Parser
Implement parser in internal/phase/parser.go to extract category and phase name from path. Handle formats: "phases/{category}/{phase}.md", "{category}/{phase}.md", "{phase}.md". Validate category exists in database. Return structured PhaseInfo with category, name, full path. Handle edge cases (missing .md extension, invalid category, malformed path).

### Database Operations Layer
Implement all phase operations in internal/db/phase.go using struct-based DB pattern from ARCHITECTURE.md. Use prepared statements for all queries (< 1ms performance). Implement CompletePhase with transaction and exclusive lock. Implement GetCurrentPhase, GetNextPhase, GetPhaseInfo, ListPhases using prepared statements. Return Go structs that map to compact JSON. Handle sql.ErrNoRows gracefully with appropriate errors. All operations must be concurrent-safe.

### Validation Logic
Implement validation functions in internal/db/validate.go. Query category statistics and compare to actual counts. Generate detailed error messages for mismatches. Suggest SQL commands to fix issues (update categories set ...). Check referential integrity (all phase.category in categories.name). Verify trigger existence by querying sqlite_master. Return structured validation report with pass/fail per check.

## Tests (TDD)

**Phase complete tests:**
1. Complete valid phase updates status
2. Triggers update category progress
3. Total progress updated
4. Audit log entry created
5. Next phase found correctly
6. Already completed phase rejected
7. Invalid phase path rejected
8. Dry run shows changes without commit
9. Git commit created if --commit flag
10. Concurrent completions serialized

**Phase current tests:**
1. Returns last completed phase
2. Returns null if no phases completed
3. Ordered by completed_date DESC
4. Compact JSON format
5. Query < 1ms

**Phase next tests:**
1. Finds next pending in category
2. Category flag filters correctly
3. Returns null if all complete
4. Falls back to other categories
5. Respects dependencies

**Phase info tests:**
1. Returns phase details by path
2. Not found returns error
3. Null fields omitted
4. All fields present if set

**Phase list tests:**
1. Lists all phases if no filters
2. Category filter works
3. Status filter works
4. Combined filters work
5. Ordered by category, ID
6. Pagination works

**Validate tests:**
1. Valid database passes all checks
2. Category count mismatch detected
3. Percentage mismatch detected
4. Orphaned phases detected
5. Invalid status detected
6. Missing triggers detected
7. Fix suggestions provided

**Git integration tests:**
1. Commit created with correct message
2. SHA returned in response
3. Not in git repo handled
4. Git not configured handled
5. Nothing to commit handled
6. File locking prevents concurrent commits

**Integration tests:**
1. Complete phase end-to-end
2. JSON output valid and compact
3. Exit codes correct
4. Errors structured
5. --debug logging works

**Minimum test count:** 50 tests
**Coverage target:** 80%+ on cmd/atlas-dev, internal/db/phase

## Integration Points
- Uses: Database from Phase 1
- Uses: Transaction handling from Phase 1
- Uses: JSON output from Phase 1
- Uses: Error handling from Phase 1
- Uses: Logging from Phase 1
- Creates: Phase CRUD operations
- Creates: Validation logic
- Creates: Git integration
- Creates: Phase path parser
- Output: Fully functional phase management for AI agents

## Acceptance
- atlas-dev phase complete works end-to-end
- Phase status updated atomically
- Category progress auto-updated by triggers
- Total progress updated in metadata
- Audit log records all changes
- Git commit created if --commit flag
- atlas-dev phase current returns last completed
- atlas-dev phase next returns next pending
- atlas-dev phase info returns phase details
- atlas-dev phase list filters by category/status
- atlas-dev validate detects all inconsistencies
- All commands return compact JSON
- Null/empty fields omitted
- Abbreviated field names used (cat, pct, etc)
- Exit codes correct (0-6)
- Structured errors with suggestions
- 50+ tests pass
- 80%+ coverage on phase operations
- go test -race passes
- golangci-lint passes
- Benchmark: phase complete < 100ms
- Benchmark: phase current < 1ms
- Benchmark: phase next < 1ms
- Concurrent phase completions safe (exclusive lock)
- All SQL queries use prepared statements
- JSON output matches TOKEN-EFFICIENCY.md spec

**Phase complete when all acceptance criteria met and make test lint passes.**

---

**Note:** Reference ARCHITECTURE.md for all DB patterns. Use struct-based DB (never globals). Use prepared statements (never direct queries). Use exclusive locks for writes (concurrent-safe).
