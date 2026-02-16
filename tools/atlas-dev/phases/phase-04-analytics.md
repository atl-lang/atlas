# Phase 04: Analytics & Validation

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Phase 3 must be 100% complete

**Verification:**
```bash
atlas-dev decision create --help  # Must show create command
atlas-dev decision list  # Must return valid JSON
go test ./internal/db/decision* -v  # All Phase 3 tests must pass
```

**What's needed:**
- Phase 3 decision log system complete
- All database queries optimized with indexes
- Prepared statements working
- JSON output compact and efficient

**If missing:** Complete Phase 3 first

---

## Objective
Implement comprehensive analytics, statistics, blocker detection, timeline analysis, and test coverage tracking - all using optimized SQLite queries with views and indexes - providing AI agents with instant access to project health metrics in < 5ms.

## Files
**Create:** `cmd/atlas-dev/summary.go` (~150 lines - summary command)
**Create:** `cmd/atlas-dev/stats.go` (~150 lines - stats command)
**Create:** `cmd/atlas-dev/blockers.go` (~100 lines - blockers command)
**Create:** `cmd/atlas-dev/timeline.go` (~120 lines - timeline command)
**Create:** `cmd/atlas-dev/coverage.go` (~100 lines - test coverage command)
**Create:** `internal/analytics/summary.go` (~200 lines - progress summary queries)
**Create:** `internal/analytics/stats.go` (~200 lines - velocity calculations)
**Create:** `internal/analytics/blockers.go` (~100 lines - blocked phase detection)
**Create:** `internal/analytics/timeline.go` (~100 lines - timeline analysis)
**Create:** `internal/analytics/coverage.go` (~100 lines - test coverage queries)
**Create:** `internal/validation/validate.go` (~250 lines - comprehensive validation)
**Create:** `internal/validation/consistency.go` (~150 lines - consistency checks)
**Update:** `cmd/atlas-dev/main.go` (add analytics commands)
**Update:** `cmd/atlas-dev/validate.go` (enhance validation)

## Dependencies
- Phase 1 infrastructure (DB, prepared statements)
- Phase 2 phase management (categories, progress)
- Phase 3 decision logs (for related decisions)
- All Phase 1-3 acceptance criteria met

## Implementation

### Summary Command
Implement summary command to show comprehensive project progress dashboard. Query v_progress view (created in Phase 1) for category summaries. Get total phase counts from metadata table. Query most recently completed phase ordered by completed_date DESC. Query next pending phase ordered by ID. Count blocked phases with status='blocked'. Return compact JSON with categories array (name, completed, total, percentage, status), total progress, current phase, next phase, and blocked count. Use single joined query where possible for < 5ms performance. **See ARCHITECTURE.md for view patterns.**

### Stats Command
Implement stats command to calculate velocity and completion estimates. Query phase counts and completion dates from phases table. Calculate phases per day by dividing completed count by days elapsed between first and last completion. Calculate phases per week by multiplying daily rate by 7. Estimate remaining days by dividing remaining phases by velocity. Project completion date by adding estimated days to last completion date. Return compact JSON with total/completed/remaining counts, velocity metrics (per day, per week), estimated days, and projected completion date. Handle edge cases: no completions yet, velocity too low.

### Blockers Command
Implement blockers command to list all blocked phases with their blocking dependencies. Query phases table WHERE status='blocked' ordered by category and ID. Parse blockers JSON field to extract blocking phase IDs or dependencies. Return array of blocked phases with: ID, path, name, category, blockers array. Use compact JSON format. Provide actionable information: what's blocked, what needs to be completed first. Query < 5ms using indexed status column.

### Timeline Command
Implement timeline command to show completion timeline grouped by date. Query phases table WHERE status='completed' with GROUP BY DATE(completed_date). Count phases completed per day. Order by date ascending for chronological view. Return array of timeline entries: date and count of phases completed that day. Useful for velocity tracking and burndown charts. Support optional --days flag to limit to recent N days. Use indexed completed_date column for performance.

### Test Coverage Command
Implement coverage command to aggregate test statistics from test_coverage table. Query SUM of test_count, passing_tests, failing_tests across all phases. Calculate AVG coverage_pct for overall coverage percentage. Count phases with test coverage data. Return compact JSON with: total_tests, passing_tests, failing_tests, coverage_pct, phases_with_tests. Support optional --category filter to get coverage per category. Query < 5ms using aggregation.

### Validation Enhancements
Enhance validate command from Phase 2 with additional consistency checks. Implement validation functions to check: category totals match actual phase counts, category completed counts match actual completed phases, no orphaned phases (category not in categories table), no invalid statuses, all triggers exist and are enabled, referential integrity maintained, metadata totals match actual counts. For each validation check, query relevant tables and compare values. Generate detailed error messages with fix suggestions: specific SQL UPDATE commands to correct issues. Return structured validation report with: valid boolean, errors array, warnings array, checks_run count, details per check.

### Analytics Database Operations
Implement all analytics queries in internal/analytics/ packages using struct-based DB pattern. Create reusable query functions that return Go structs serializing to compact JSON. Use prepared statements where queries are repeated. Use database views (v_progress, v_active_phases) for complex queries. Implement caching for expensive queries with short TTL. All analytics queries must be read-only, use shared read locks for concurrent access. Target < 5ms for all analytics queries using indexed columns.

### Validation Logic
Implement validation logic in internal/validation/ packages. Create ValidateDatabase function that runs all consistency checks in single transaction for atomicity. Check category statistics against actual phase counts using JOIN queries. Detect orphaned records with NOT IN subqueries. Verify trigger existence by querying sqlite_master table. Compare metadata values to aggregated counts. Return detailed validation report with specific errors and SQL fix commands. Enable --fix flag to automatically apply corrections using UPDATE statements.

## Tests (TDD)

**Summary tests:**
1. Summary returns all categories
2. Total progress calculated correctly
3. Current phase is most recent completed
4. Next phase is first pending
5. Blocked count accurate
6. Query < 5ms
7. Compact JSON output
8. Empty database handled

**Stats tests:**
1. Velocity calculated correctly
2. Estimated days accurate
3. Completion date projected
4. Handles no completions
5. Handles single day completions
6. Per-day and per-week rates correct
7. Query < 5ms

**Blockers tests:**
1. Lists all blocked phases
2. Blockers array parsed from JSON
3. Ordered by category
4. Empty blockers handled
5. Query < 5ms

**Timeline tests:**
1. Groups by date correctly
2. Counts phases per day
3. Ordered chronologically
4. Handles multiple phases per day
5. Recent N days filter works
6. Query < 5ms

**Coverage tests:**
1. Aggregates test counts
2. Calculates avg coverage
3. Counts phases with tests
4. Category filter works
5. Zero tests handled
6. Query < 5ms

**Validation tests:**
1. Valid database passes all checks
2. Category count mismatch detected
3. Orphaned phases detected
4. Invalid status detected
5. Missing triggers detected
6. Metadata mismatch detected
7. Fix suggestions generated
8. Auto-fix applies corrections

**Minimum test count:** 40 tests
**Coverage target:** 80%+ on internal/analytics, internal/validation

## Integration Points
- Uses: Database from Phase 1 (views, indexes)
- Uses: Category system from Phase 1
- Uses: Phase tracking from Phase 2
- Uses: Decision logs from Phase 3
- Creates: Analytics query layer
- Creates: Validation system
- Creates: Health metrics
- Creates: Velocity tracking
- Output: Comprehensive project health dashboard for AI agents

## Acceptance
- atlas-dev summary returns comprehensive dashboard
- Category progress accurate (matches database)
- Total progress calculated correctly
- Current/next phase identification works
- atlas-dev stats calculates velocity
- Velocity metrics accurate (per day, per week)
- Completion estimates reasonable
- atlas-dev blockers lists blocked phases
- Blocker dependencies parsed correctly
- atlas-dev timeline shows completion by date
- Timeline grouping by date works
- atlas-dev test-coverage aggregates test stats
- Coverage percentages accurate
- atlas-dev validate enhanced with consistency checks
- Category count validation works
- Orphaned record detection works
- Invalid status detection works
- Metadata validation works
- Trigger existence verification works
- Fix suggestions actionable
- Optional --fix flag applies corrections
- All commands return compact JSON
- Null/empty fields omitted
- Abbreviated field names used
- Exit codes correct (0-6)
- 40+ tests pass
- 80%+ coverage on analytics/validation
- go test -race passes
- golangci-lint passes
- Benchmark: summary < 5ms
- Benchmark: stats < 5ms
- Benchmark: all analytics queries < 5ms
- JSON output ~60-120 tokens (token-efficient)
- All queries use indexes for performance
- Validation detects all inconsistency types

**Phase complete when all acceptance criteria met and make test lint passes.**

---

**Note:** Reference ARCHITECTURE.md for view and index patterns. Use prepared statements for repeated queries. Target < 5ms for all analytics - essential for AI agent responsiveness.
