# Phase 05: Context System

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Phase 4 must be 100% complete

**Verification:**
```bash
atlas-dev summary  # Must return valid dashboard JSON
atlas-dev stats  # Must calculate velocity
go test ./internal/analytics/... -v  # All Phase 4 tests must pass
```

**What's needed:**
- Phase 4 analytics system complete
- All database queries optimized
- Summary and stats commands working
- Validation system operational

**If missing:** Complete Phase 4 first

---

## Objective
Aggregate comprehensive phase context from database and phase markdown files - combining structured DB data with phase instructions, dependencies, related decisions, and navigation - providing AI agents with everything needed to start work in a single < 10ms query returning ~200 tokens.

## Files
**Create:** `cmd/atlas-dev/context.go` (~150 lines - context command group)
**Create:** `cmd/atlas-dev/context_current.go` (~100 lines - current command)
**Create:** `cmd/atlas-dev/context_phase.go` (~100 lines - phase command)
**Create:** `internal/context/aggregator.go` (~300 lines - context aggregation)
**Create:** `internal/context/phase_parser.go` (~250 lines - parse phase markdown files)
**Create:** `internal/context/enricher.go` (~200 lines - enrich with DB data)
**Create:** `internal/parser/markdown.go` (~150 lines - markdown parser utilities)
**Update:** `cmd/atlas-dev/main.go` (add context command group)

## Dependencies
- Phase 1 infrastructure (DB, JSON output)
- Phase 2 phase management (phase tracking)
- Phase 3 decision logs (related decisions)
- Phase 4 analytics (category progress)
- All Phase 1-4 acceptance criteria met

## Implementation

### Phase File Parser
Implement markdown parser in internal/context/phase_parser.go to extract structured data from phase markdown files. Parse phase files to extract: objective (from "Objective:" line), priority (from "Priority:" metadata), dependencies (from "Depends On:" metadata), deliverables (from numbered list in Deliverables section), acceptance criteria (from checklist in Acceptance section), estimated time (from metadata). Use bufio.Scanner to read line-by-line. Track current section with state machine. Extract deliverables from lines starting with numbers or checkboxes. Extract acceptance criteria from "- [ ]" checklist items. Handle missing sections gracefully (phase file format may vary). Return PhaseFile struct with all extracted fields.

### Context Aggregator
Implement context aggregator in internal/context/aggregator.go to combine DB data with parsed phase files. Create GetPhaseContext function that queries database for: phase metadata (ID, path, name, category, status, description, blockers, completed_date), category progress (completed/total/percentage from categories table), related decisions (query decisions table WHERE component=category, order by date DESC, limit 5), test coverage data (query test_coverage table for phase_id), navigation (previous phase: last completed in same category, next phase: first pending in same category). Parse phase markdown file using phase_parser. Combine all data into single PhaseContext struct. Use single JOIN query where possible to minimize database round-trips. **See ARCHITECTURE.md for efficient query patterns.**

### Context Current Command
Implement context current subcommand to show context for the next phase to work on. Query database for last completed phase (ORDER BY completed_date DESC LIMIT 1). Get category of last completed phase. Find next pending phase in same category (WHERE category=X AND status='pending' ORDER BY id LIMIT 1). If no pending in same category, find next pending overall. Call GetPhaseContext with found phase path. Return complete context with all phase data, instructions, dependencies, and navigation. Target < 10ms query time by using prepared statements and efficient joins.

### Context Phase Command
Implement context phase subcommand taking phase path argument. Validate phase path exists in database. Call GetPhaseContext with provided path. Return complete phase context including: phase metadata from DB, parsed phase file instructions (objective, deliverables, acceptance criteria), category progress, related decisions (up to 5 most recent for category), test coverage data if available, navigation hints (previous/next phase in category). Handle missing phase file gracefully - return DB data even if markdown file not found. Output compact JSON omitting null fields.

### Context Enricher
Implement enricher in internal/context/enricher.go to add supplementary data to context. Enrich phase context with: related features (query features table WHERE category matches or feature mentioned in phase), blocking/blocked phases (parse blockers JSON and look up phase names), recent audit log entries (last 3 changes to this phase), git information if available (last commit affecting phase file). Keep enrichment lightweight - don't slow down < 10ms target. Make enrichment fields optional - omit if data not available or takes too long to fetch.

### Markdown Parser Utilities
Implement reusable markdown parsing utilities in internal/parser/markdown.go. Create functions to: extract metadata from key:value lines, parse numbered lists, parse checkbox lists, extract code blocks, parse heading sections, split document by headings. Use regex patterns for robust parsing. Handle edge cases: missing sections, malformed markdown, UTF-8 encoding. Return parsed structures that are easy to serialize to JSON. Support both strict and lenient parsing modes.

### Database Integration
Use prepared statements for all context queries. Create composite query that JOINs phases, categories, and related tables in single database call. Cache parsed phase files with short TTL to avoid re-parsing on repeated requests. Use read-only transactions for consistency. Ensure all queries complete in < 10ms by using indexed columns and efficient JOINs. Monitor query performance with benchmark tests.

## Tests (TDD)

**Phase parser tests:**
1. Parse complete phase file
2. Extract objective correctly
3. Extract priority and dependencies
4. Parse numbered deliverables list
5. Parse acceptance checklist
6. Handle missing sections gracefully
7. Handle malformed markdown
8. UTF-8 encoding handled

**Context aggregator tests:**
1. Aggregate DB data and phase file
2. Category progress included
3. Related decisions fetched (max 5)
4. Test coverage included if available
5. Navigation (prev/next) correct
6. Handles missing phase file
7. Single query for performance
8. Query < 10ms

**Context current tests:**
1. Finds next phase to work on
2. Prefers same category as last completed
3. Falls back to other categories
4. Returns complete context
5. Handles no completed phases
6. Handles all phases complete
7. Query < 10ms

**Context phase tests:**
1. Returns context for specific phase
2. Validates phase exists
3. Includes all context fields
4. Handles phase not found
5. Null fields omitted
6. Query < 10ms

**Markdown parser tests:**
1. Extract metadata correctly
2. Parse lists and checkboxes
3. Extract code blocks
4. Split by headings
5. Handle edge cases
6. Regex patterns work

**Integration tests:**
1. Context current end-to-end
2. Context phase end-to-end
3. JSON output valid
4. Compact format (~200 tokens)
5. All fields present when available

**Minimum test count:** 35 tests
**Coverage target:** 80%+ on internal/context, internal/parser

## Integration Points
- Uses: Database from Phase 1
- Uses: Phase tracking from Phase 2
- Uses: Decision logs from Phase 3
- Uses: Analytics from Phase 4 (category progress)
- Creates: Context aggregation system
- Creates: Phase file parser
- Creates: Markdown utilities
- Creates: AI-ready context bundles
- Output: Complete phase context in single query for AI agents

## Acceptance
- atlas-dev context current returns next phase context
- Context includes all necessary fields
- Phase file parsing extracts objectives/deliverables/criteria
- Context aggregation combines DB + file data efficiently
- atlas-dev context phase <path> returns specific phase context
- Category progress included in context
- Related decisions included (up to 5 recent)
- Test coverage data included if available
- Navigation hints included (prev/next phase)
- Missing phase file handled gracefully
- All commands return compact JSON
- Null/empty fields omitted
- Context output ~200 tokens (everything AI needs)
- Exit codes correct (0-6)
- Structured errors with suggestions
- 35+ tests pass
- 80%+ coverage on context/parser
- go test -race passes
- golangci-lint passes
- Benchmark: context current < 10ms
- Benchmark: context phase < 10ms
- Phase file parsing resilient to format variations
- Markdown parser handles edge cases
- Single database query for performance
- JSON output includes everything AI needs to start work
- No redundant or unnecessary data in output

**Phase complete when all acceptance criteria met and make test lint passes.**

---

**Note:** Reference ARCHITECTURE.md for efficient query patterns. Target < 10ms total time including file parsing. Context should contain everything an AI needs to start working on a phase - no additional queries required.
