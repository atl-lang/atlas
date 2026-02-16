# Phase 01: SQLite Infrastructure

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Go 1.22+ installed, SQLite3 available

**Verification:**
```bash
go version  # Must be 1.22 or higher
sqlite3 --version
```

**What's needed:**
- Go 1.22+ with module support
- Git configured
- Understanding of SQLite and Go database/sql patterns

**If missing:** Install Go 1.22+ and SQLite3

---

## Objective
Build the foundational SQLite database infrastructure with schema, transactions, prepared statements, and CLI framework - establishing the single source of truth for all atlas-dev tracking with < 1ms query performance and ACID guarantees.

## Files
**Create:** `internal/db/db.go` (~200 lines - struct-based DB with prepared statements)
**Create:** `internal/db/schema.go` (~500 lines - tables, indexes, triggers, views)
**Create:** `internal/db/transaction.go` (~100 lines - correct transaction handling)
**Create:** `internal/db/errors.go` (~50 lines - sentinel errors)
**Create:** `internal/db/testing.go` (~100 lines - test helpers)
**Create:** `internal/output/json.go` (~150 lines - compact JSON output)
**Create:** `internal/lock/filelock.go` (~100 lines - file locking for git)
**Create:** `internal/audit/log.go` (~150 lines - audit logging)
**Create:** `cmd/atlas-dev/main.go` (~200 lines - CLI framework with Cobra)
**Create:** `cmd/atlas-dev/migrate.go` (~100 lines - migration commands)
**Create:** `Makefile` (~100 lines - build, test, lint targets)
**Create:** `.golangci.yml` (~50 lines - linter config)
**Update:** `go.mod` (add dependencies: mattn/go-sqlite3, cobra)

## Dependencies
- mattn/go-sqlite3 v1.14.18+ (CGO-based, fastest)
- github.com/spf13/cobra v1.8.0+ (CLI framework)
- Go stdlib: database/sql, log/slog

## Implementation

### Database Connection Pattern
Implement struct-based DB (not globals) for concurrent safety and testability. Create New function that opens SQLite with WAL mode, foreign keys enabled, and busy timeout of 5000ms. Set MaxOpenConns to 1 for SQLite single-writer constraint. Prepare common queries as prepared statements and cache in map for < 1ms performance. Implement Close to cleanup prepared statements and connection. **See ARCHITECTURE.md for canonical pattern.**

### Schema Creation
Create 8 tables: phases, categories, decisions, features, metadata, parity_checks, test_coverage, audit_log. Define all columns with appropriate types and constraints. Create 14 indexes for query performance: phase category/status/date, decision component/date/status, feature version/status, audit timestamp/entity, parity timestamp/status, coverage timestamp/category. Implement 4 triggers for auto-updates: update_category_progress, update_phases_timestamp, update_decisions_timestamp, update_features_timestamp. Create 3 views: v_progress, v_active_phases, v_recent_decisions. Seed categories table with 9 categories: foundation, stdlib, bytecode-vm, frontend, typing, interpreter, cli, lsp, polish. Seed metadata with schema_version=1, atlas_version=v0.2, total_phases=78, completed_phases=0.

### Transaction Handling
Implement WithTransaction method on DB struct that creates sql.Tx, wraps in Transaction type, handles panic recovery with rollback, executes function, and commits on success or rollbacks on error. Add Exec, QueryRow, Query methods to Transaction type. **Critical: Do NOT defer rollback unconditionally - see ARCHITECTURE.md for correct pattern.**

### Prepared Statement Caching
Prepare common queries on DB initialization: getPhase, getPhaseByPath, updatePhaseStatus, listPhases, getCategory, getCategoryProgress, getTotalProgress, getNextPhase, insertAuditLog. Store in map[string]*sql.Stmt on DB struct. Use prepared statements in all query methods for performance (< 1ms guarantee).

### Concurrent Access Support
Implement WithExclusiveLock method using sync.RWMutex for write operations. WAL mode enables concurrent reads (multiple AI agents can query simultaneously). Exclusive lock ensures single writer (prevents race conditions on phase complete). All write operations must use WithExclusiveLock + WithTransaction for ACID guarantees.

### Structured Error Handling
Define sentinel errors in internal/db/errors.go: ErrPhaseNotFound, ErrPhaseAlreadyDone, ErrInvalidStatus, ErrCategoryNotFound. Implement Error and ErrorWithDetails in internal/output/error.go that map errors to exit codes 0-6 (defined in AI-OPTIMIZATION.md). Output structured JSON with ok=false, err message, optional details, and optional suggestion. **See ARCHITECTURE.md for canonical error pattern.**

### Structured Logging
Use log/slog for all logging. Configure JSON handler on stderr with DEBUG level if --debug flag set, INFO level otherwise. Log all database operations with structured fields: query type, duration_ms, row count, errors. Log transaction begin/commit/rollback events. Separate logs (stderr) from JSON output (stdout) for AI parsing. **See ARCHITECTURE.md for logging pattern.**

### Compact JSON Output
Implement Success and Error functions in internal/output/json.go. Use compact encoding (no spaces or newlines). Remove null and empty fields from output. Use abbreviated field names from TOKEN-EFFICIENCY.md: ok, err, msg, cat, pct, cnt, tot, cmp, mod, dep, blk, desc, ts. Encode directly to stdout with no escaping. **Follow TOKEN-EFFICIENCY.md exactly - already designed for 76% token reduction.**

### File Locking for Git Operations
Implement Acquire and Release in internal/lock/filelock.go. Create exclusive lock file (atlas-dev.db.lock) with PID. Wait up to 5 seconds with 100ms retry interval. Implement WithLock helper for automatic cleanup. Used by git commit operations to prevent concurrent commits.

### Audit Logging
Implement Log and LogWithCommit in internal/audit/log.go. Insert to audit_log table with timestamp, action, entity_type, entity_id, changes JSON, optional commit_sha. Implement GetRecent to query last N audit entries. Used to track all database changes for undo capability and debugging.

### CLI Framework
Implement main.go with Cobra root command. Add persistent --db flag (default: atlas-dev.db), --debug flag for logging. Implement PersistentPreRunE to open database, PersistentPostRunE to close. Implement version command that outputs JSON with version and schema_version. Implement migrate command group with bootstrap and schema subcommands. Configure slog based on --debug flag. Handle errors with structured JSON output and proper exit codes.

### Build and Quality Infrastructure
Create Makefile with targets: build (compile binary), test (run all tests with -race -cover), bench (run benchmarks), lint (golangci-lint), fmt (gofmt + goimports), clean. Create .golangci.yml with linters: govet, errcheck, staticcheck, gosec, ineffassign, unused. Set coverage threshold to 80% on critical paths. **All commands must pass before phase is complete.**

## Tests (TDD)

**Database tests:**
1. Database opens successfully (file and :memory:)
2. Schema creates all 8 tables
3. Indexes created (14 total)
4. Triggers created and fire correctly
5. Views created and query correctly
6. Categories seeded (9 rows)
7. Metadata seeded (4 rows)
8. Prepared statements cached
9. Connection pool configured (MaxOpenConns=1)
10. Close cleans up statements and connection

**Transaction tests:**
1. Transaction commits on success
2. Transaction rollbacks on error
3. Transaction rollbacks on panic
4. Nested operations atomic
5. Concurrent transactions serialized

**Error handling tests:**
1. Error maps to correct exit code
2. Structured JSON error output
3. Details included in error response
4. Suggestions generated when applicable

**Logging tests:**
1. Debug logs only with --debug flag
2. Structured JSON format
3. Logs to stderr not stdout
4. Query duration logged

**JSON output tests:**
1. Compact encoding (no spaces)
2. Null fields omitted
3. Empty arrays omitted
4. Abbreviated field names used
5. Valid JSON always

**CLI tests:**
1. version command outputs valid JSON
2. migrate schema creates all tables
3. --db flag overrides default path
4. --debug enables debug logging
5. Exit codes correct (0 success, 1+ errors)

**Benchmark tests:**
1. GetPhase < 1ms (prepared statement)
2. Category progress query < 1ms
3. Total progress query < 1ms
4. Transaction commit < 10ms

**Minimum test count:** 35 tests
**Coverage target:** 80%+ on internal/db, internal/output

## Integration Points
- Creates: Database schema (8 tables, 14 indexes, 4 triggers)
- Creates: Struct-based DB with prepared statements
- Creates: Transaction handling (correct pattern)
- Creates: Structured error handling (exit codes 0-6)
- Creates: Structured logging (slog)
- Creates: Compact JSON output (token-efficient)
- Creates: CLI framework (Cobra)
- Creates: Build infrastructure (Makefile, linters)
- Creates: Test helpers (newTestDB, seedTestPhase)
- Output: Ready for Phase 2 (phase management)

## Acceptance
- All 8 tables created with correct schema
- All 14 indexes created
- All 4 triggers fire correctly
- All 3 views query correctly
- 9 categories seeded
- Metadata seeded (schema_version=1)
- Prepared statements cached (9 queries)
- Struct-based DB (no globals)
- Transaction pattern correct (no defer bug)
- Exclusive lock for writes (concurrent-safe)
- Exit codes 0-6 implemented
- Structured errors with codes
- Logging with slog (--debug flag)
- JSON output compact (token-efficient)
- Null/empty fields omitted
- Abbreviated field names used
- atlas-dev version returns valid JSON
- atlas-dev migrate schema creates all tables
- 35+ tests pass
- 80%+ coverage on critical paths
- go test -race passes (no race conditions)
- golangci-lint passes (no warnings)
- Benchmark: GetPhase < 1ms
- Benchmark: Transaction < 10ms
- Documentation: ARCHITECTURE.md exists (canonical patterns)

**Phase complete when all acceptance criteria met and make test lint passes.**

---

**Note:** Reference ARCHITECTURE.md for all implementation patterns. Do not deviate from canonical patterns without updating ARCHITECTURE.md first.
