# Phase 07: Feature Management

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Phase 6 must be 100% complete

**Verification:**
```bash
atlas-dev export markdown  # Must generate STATUS.md
atlas-dev undo --help  # Must show undo command
go test ./internal/export/... -v  # All Phase 6 tests must pass
```

**What's needed:**
- Phase 6 polish features complete
- Export system working
- Undo system functional
- All core systems operational

**If missing:** Complete Phase 6 first

---

## Objective
Implement feature documentation management system - parsing feature docs from docs/features/, validating against code implementation, syncing with codebase changes, and tracking implementation status - enabling AI agents to maintain accurate feature documentation automatically.

## Files
**Create:** `cmd/atlas-dev/feature.go` (~150 lines - feature command group)
**Create:** `cmd/atlas-dev/feature_create.go` (~100 lines - create command)
**Create:** `cmd/atlas-dev/feature_list.go` (~100 lines - list command)
**Create:** `cmd/atlas-dev/feature_read.go` (~100 lines - read command)
**Create:** `cmd/atlas-dev/feature_update.go` (~150 lines - update command)
**Create:** `cmd/atlas-dev/feature_validate.go` (~150 lines - validate command)
**Create:** `cmd/atlas-dev/feature_sync.go` (~150 lines - sync command)
**Create:** `cmd/atlas-dev/feature_delete.go` (~80 lines - delete command)
**Create:** `cmd/atlas-dev/feature_search.go` (~100 lines - search command)
**Create:** `internal/feature/parser.go` (~300 lines - parse feature markdown files)
**Create:** `internal/feature/validator.go` (~250 lines - validate against code)
**Create:** `internal/feature/sync.go` (~200 lines - auto-sync from code)
**Create:** `internal/db/feature.go` (~250 lines - feature DB operations)
**Update:** `cmd/atlas-dev/main.go` (add feature command group)

## Dependencies
- Phase 1 infrastructure (DB, JSON output)
- Phase 5 context (markdown parser utilities)
- Phase 6 export (for feature export)
- Rust code parsing capability (for validation/sync)
- All Phase 1-6 acceptance criteria met

## Implementation

### Feature Parser
Implement feature markdown parser in internal/feature/parser.go to extract structured data from docs/features/ files. Parse feature files to extract: name (from heading), category (from Category metadata), status (Implemented/InProgress/Planned), since version, spec reference, API reference, overview text, function list, implementation details (file paths, test paths, test count, parity percentage), related items (decisions, other features). Use markdown parser utilities from Phase 5. Handle various markdown formats gracefully. Return Feature struct with all extracted fields. Support both strict and lenient parsing modes.

### Feature Create Command
Implement feature create subcommand with flags: --name (required), --category (required), --status (default: Planned), --spec, --api. Generate feature markdown file in docs/features/ directory with template format. Insert feature record into features table in database with: name, category, status, version, spec_ref, api_ref, created timestamp. Create placeholder sections: Overview, Functions, Implementation, Related. Return compact JSON with feature name and file path. Use WithTransaction for atomic creation.

### Feature List Command
Implement feature list subcommand with optional --category and --status filters. Query features table with WHERE clauses for filters. Parse feature markdown files to get additional metadata if --detailed flag set. Return array of features with: name, category, status, function count, test count, parity percentage. Support --format flag for different output styles: compact (default), detailed, summary. Order by category then name. Use prepared statements for performance.

### Feature Read Command
Implement feature read subcommand taking feature name argument. Query features table for feature record. Parse feature markdown file using parser. Combine database and file data. Return complete feature details: name, category, status, version, spec/API refs, overview, functions list, implementation details, test coverage, parity status, related items. Output compact JSON omitting null fields. Handle feature not found with structured error.

### Feature Update Command
Implement feature update subcommand with flags: --status, --version, --parity, --test-count, --add-function, --remove-function. Query feature by name, load current data. Update specified fields in database using WithTransaction. Optionally update markdown file if --sync-file flag set. Insert audit log entry with changes. Return compact JSON with updated fields. Validate status transitions (Planned → InProgress → Implemented). **See ARCHITECTURE.md for transaction patterns.**

### Feature Validate Command
Implement feature validate subcommand taking feature name argument. Parse feature markdown to extract claimed implementation details (function count, test count, parity percentage). Parse Rust code in implementation file to count actual public functions. Parse test files to count actual tests. Compare claimed vs actual values. Check spec reference file exists if specified. Check API reference section exists if specified. Return validation report with: spec_ref_valid, api_ref_valid, impl_file_exists, test_file_exists, function_count_match, test_count_match, parity_accurate. Include expected vs actual values for mismatches.

### Feature Sync Command
Implement feature sync subcommand to auto-update feature docs from codebase. Parse Rust implementation file to extract: public function count, function signatures, doc comments, last modified date. Parse test files to count tests. Calculate actual parity if both implementations exist. Update feature markdown file with current values. Update features table in database. Insert audit log entry. Return JSON with fields that were updated and new values. Support --dry-run to preview changes without applying.

### Feature Search Command
Implement feature search subcommand taking search query. Use LIKE operator to search feature name, category, overview text. Support filtering by status and category simultaneously. Return matching features with relevance score (based on where match found: name=high, category=medium, overview=low). Order by relevance then name. Return compact JSON array. Query < 10ms using indexes.

### Feature Validation Logic
Implement validation logic in internal/feature/validator.go. Create functions to: check file existence, parse Rust code for function count, parse test files for test count, verify spec/API references valid, compare claimed vs actual values. Use regex patterns or tree-sitter for Rust parsing. Generate detailed validation reports with specific errors and fix suggestions. Support batch validation of all features.

### Feature Sync Logic
Implement sync logic in internal/feature/sync.go. Create functions to: find implementation files, parse Rust code structure, extract function signatures and counts, find test files, count test cases, calculate code coverage if available, update markdown files with current data, update database records. Handle missing files gracefully. Support --all flag to sync all features. Return summary of changes made.

### Database Operations
Implement feature CRUD in internal/db/feature.go using struct-based DB pattern. Use prepared statements for: getFeature, listFeatures, searchFeatures, createFeature, updateFeature, deleteFeature. Store feature metadata in features table, reference markdown files on disk for full content. Use transactions for consistency. Return Go structs that serialize to compact JSON.

## Tests (TDD)

**Feature parser tests:**
1. Parse complete feature file
2. Extract all metadata fields
3. Parse function lists
4. Parse implementation section
5. Handle missing sections
6. Multiple format variants work
7. UTF-8 encoding handled

**Feature create tests:**
1. Create feature with markdown file
2. Database record created
3. Template format correct
4. Transaction atomic
5. Duplicate name rejected
6. Directory created if needed

**Feature list tests:**
1. List all features
2. Filter by category
3. Filter by status
4. Combined filters work
5. Detailed mode includes extra data
6. Query < 5ms

**Feature read tests:**
1. Read feature by name
2. Combine DB + file data
3. Not found handled
4. All fields included
5. Null fields omitted

**Feature update tests:**
1. Update status works
2. Update version works
3. Update parity works
4. Audit log created
5. Transaction atomic
6. Invalid transitions rejected

**Feature validate tests:**
1. Function count validated
2. Test count validated
3. Spec reference checked
4. API reference checked
5. Implementation file checked
6. Mismatches detected
7. Validation report detailed

**Feature sync tests:**
1. Sync from code works
2. Function count updated
3. Test count updated
4. Markdown file updated
5. Database updated
6. Dry-run previews changes
7. Handles missing files

**Feature search tests:**
1. Search by name
2. Search by category
3. Search by overview
4. Relevance scoring works
5. Multiple results ordered
6. Query < 10ms

**Minimum test count:** 45 tests
**Coverage target:** 80%+ on internal/feature, cmd/atlas-dev/feature*

## Integration Points
- Uses: Database from Phase 1
- Uses: Markdown parser from Phase 5
- Uses: Transaction handling from Phase 1
- Uses: Audit logging from Phase 1
- Creates: Feature documentation system
- Creates: Feature validation against code
- Creates: Auto-sync from codebase
- Creates: Feature CRUD operations
- Output: Complete feature management for tracking implementation status

## Acceptance
- atlas-dev feature create generates markdown file
- Feature markdown follows template format
- Database record created atomically
- atlas-dev feature list filters by category/status
- List output compact and efficient
- atlas-dev feature read returns complete details
- Combines database and file data
- atlas-dev feature update modifies status/version/parity
- Updates atomic with audit logging
- atlas-dev feature validate checks against code
- Function count validation works
- Test count validation works
- Spec/API reference validation works
- Mismatch detection accurate
- atlas-dev feature sync updates from codebase
- Sync extracts function/test counts from code
- Markdown and database both updated
- Dry-run previews without applying
- atlas-dev feature search finds matches
- Relevance scoring reasonable
- All commands return compact JSON
- Exit codes correct (0-6)
- Structured errors with suggestions
- 45+ tests pass
- 80%+ coverage on feature management
- go test -race passes
- golangci-lint passes
- Feature parser handles format variations
- Rust code parsing works (or graceful degradation)
- Validation catches all mismatch types
- Sync keeps docs current with code

**Phase complete when all acceptance criteria met and make test lint passes.**

---

**Note:** Reference ARCHITECTURE.md for DB patterns. Rust code parsing may use regex or tree-sitter. Graceful degradation if parsing unavailable - rely on manual updates.
