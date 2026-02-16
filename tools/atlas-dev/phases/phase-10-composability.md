# Phase 10: Composability & Piping

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Phase 9 must be 100% complete

**Verification:**
```bash
atlas-dev validate parity  # Must run comprehensive validation
atlas-dev validate all  # Must aggregate all validators
go test ./internal/parity/... -v  # All Phase 9 tests must pass
```

**What's needed:**
- Phase 9 parity validation complete
- All core commands operational
- JSON output standardized across commands
- Exit codes consistent

**If missing:** Complete Phase 9 first

---

## Objective
Enable command composition and piping - adding stdin support to all commands, enabling JSON streaming between commands, supporting batch operations, and implementing pipeline error handling - allowing AI agents to chain operations efficiently in Unix-style pipelines reducing tool calls from N to 1.

## Files
**Create:** `internal/compose/stdin.go` (~150 lines - stdin input handling)
**Create:** `internal/compose/batch.go` (~200 lines - batch operations)
**Create:** `internal/compose/pipeline.go` (~150 lines - pipeline execution)
**Update:** ALL cmd/atlas-dev/*_*.go files (~20-30 lines each - add --stdin flag)
**Update:** `internal/output/json.go` (~50 lines - streaming output support)

## Dependencies
- Phase 1 infrastructure (JSON output foundation)
- All Phases 2-9 commands (to add stdin support)
- All commands return valid, parseable JSON
- All commands use consistent field names
- All Phase 1-9 acceptance criteria met

## Implementation

### Stdin Support
Implement stdin reader in internal/compose/stdin.go. Create ReadStdin function to read JSON from stdin until EOF. Create ParseJSONFromStdin to parse stdin content as JSON (object or array). Create ExtractIDs function to extract ID fields from JSON (handles: single object with id, array of objects with id, array of strings). Create ExtractPaths function to extract path fields (phase paths, file paths). Add error handling for invalid JSON, empty stdin, malformed data. Support both single items and arrays for flexibility.

### Add Stdin Flag to All Commands
Update every command file (decision, phase, feature, spec, api, etc) to add --stdin flag. Implement logic: if --stdin flag set, read input from stdin instead of command arguments, parse JSON to extract relevant fields (IDs, paths, names), execute command for each item from stdin. For single-item commands (read, validate), extract first item from stdin array. For batch commands (list results), process all items. Maintain same output format regardless of input source. Commands should work identically whether arguments passed on command line or via stdin.

### JSON Streaming
Ensure all commands output valid JSON that can be piped to next command. Standardize output formats: list commands return arrays in consistent field names (items, results, phases, decisions, features), single-item commands return object with data field, error responses include ok=false. Add streaming mode (optional) where commands output one JSON object per line for large result sets. Implement compact output by default (no pretty-printing) for efficient piping. **Reference ARCHITECTURE.md and TOKEN-EFFICIENCY.md for output standards.**

### Batch Operations
Implement batch processor in internal/compose/batch.go. Create BatchProcess function that takes array of items and operation function, executes operation for each item, collects results and errors, returns aggregated result. Add progress reporting to stderr (doesn't interfere with stdout JSON). Implement BatchValidate for validating multiple items in parallel. Add BatchUpdate for updating multiple items atomically. Support --parallel flag to process items concurrently (use goroutines with worker pool). Include --continue-on-error flag to process all items even if some fail.

### Pipeline Error Handling
Implement pipeline utilities in internal/compose/pipeline.go. Create Pipeline struct to represent multi-step pipeline. Add error propagation: if any step returns non-zero exit code, stop pipeline. Add error aggregation: collect errors from all steps for final report. Implement retry logic (optional): retry failed steps with exponential backoff. Add --dry-run support: simulate pipeline without executing. Include transaction-like behavior for database operations: rollback all changes if pipeline fails partway.

### Progress Reporting
Add --progress flag to batch operations. Output progress to stderr in format: [N/M] Processing item... OK/FAIL. Show percentage complete. Estimate time remaining based on average operation time. Don't interfere with stdout JSON output. Use ANSI escape codes for pretty output if terminal detected (optional). Support --quiet flag to suppress progress output.

### Command Integration
Update each command category to support composition patterns. Decision commands: pipe search results to read, pipe list to validate. Phase commands: pipe list to context, pipe next to complete. Feature commands: pipe list to validate, pipe list to sync. Support xargs integration: output newline-separated values with --format=lines for xargs compatibility. Support jq integration: ensure JSON structure is jq-friendly (arrays, consistent field names).

### Dry-Run Support
Add --dry-run flag to all mutating commands (create, update, delete, complete). In dry-run mode: parse and validate inputs, simulate operation, show what would change, return JSON with changes object showing before/after, don't actually modify database or files. Support --dry-run in pipelines to preview entire pipeline. Useful for AI agents to verify operations before executing.

### Pipeline Composition Examples
Document common pipeline patterns for AI agents. Pattern 1: Search and read (search decisions → read each). Pattern 2: List and validate (list features → validate each). Pattern 3: Find and process (find pending phases → show context for each). Pattern 4: Complete workflow (complete phase → update feature → validate parity → commit if valid). Provide command templates AI can use as-is.

## Tests (TDD)

**Stdin reader tests:**
1. Read JSON from stdin
2. Parse object from stdin
3. Parse array from stdin
4. Extract IDs correctly
5. Extract paths correctly
6. Handle empty stdin
7. Handle invalid JSON
8. Handle EOF correctly

**Stdin integration tests:**
1. Decision read with stdin
2. Phase context with stdin
3. Feature validate with stdin
4. Batch processing via stdin
5. Array input works
6. Single object input works

**Batch processor tests:**
1. Process multiple items
2. Collect results correctly
3. Handle errors per item
4. Continue on error works
5. Stop on error works
6. Parallel processing works
7. Progress reporting to stderr

**Pipeline tests:**
1. Chain commands with pipes
2. Error propagation works
3. Exit codes correct
4. JSON flows between commands
5. Dry-run shows changes
6. Transaction rollback works

**Integration tests:**
1. Search → read pipeline
2. List → validate pipeline
3. Find → context pipeline
4. Complete → validate → commit pipeline
5. xargs integration works
6. jq parsing works

**Minimum test count:** 35 tests
**Coverage target:** 80%+ on internal/compose

## Integration Points
- Updates: ALL commands from Phases 2-9
- Uses: JSON output from Phase 1
- Uses: Transaction handling from Phase 1
- Creates: Stdin support layer
- Creates: Batch processing
- Creates: Pipeline utilities
- Creates: Composability infrastructure
- Output: Complete Unix-style composability - commands can be chained efficiently

## Acceptance
- ALL commands support --stdin flag
- Stdin reads JSON (object or array)
- IDs extracted from stdin correctly
- Paths extracted from stdin correctly
- Commands work identically with stdin or args
- JSON output pipes to next command
- Consistent field names across commands
- List commands return arrays
- Single-item commands return objects
- Batch processing works
- Multiple items processed efficiently
- Progress reporting to stderr works
- Doesn't interfere with stdout JSON
- --parallel flag speeds up batch ops
- --continue-on-error processes all items
- Pipeline error propagation works
- Non-zero exit stops pipeline
- --dry-run previews changes
- Shows before/after in JSON
- Doesn't modify data in dry-run
- xargs integration works
- Commands output newline-separated with --format=lines
- jq integration works
- JSON structure jq-friendly
- Exit codes propagate correctly
- 35+ tests pass
- 80%+ coverage on composability
- go test -race passes
- golangci-lint passes
- Pipeline examples documented
- AI agents can use patterns as-is
- Decision search | read works
- Feature list | validate works
- Phase list | context works
- Complex workflows possible in single pipeline
- Token savings: N commands → 1 pipeline

**Phase complete when all acceptance criteria met and make test lint passes.**

---

**Note:** Reference ARCHITECTURE.md for patterns. This phase makes atlas-dev truly composable - AI agents can accomplish complex workflows with single pipelines instead of multiple tool calls. Critical for token efficiency and agent simplicity.
