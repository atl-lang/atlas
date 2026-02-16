# Phase 09: Parity Validation

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Phase 8 must be 100% complete

**Verification:**
```bash
atlas-dev spec validate  # Must validate spec references
atlas-dev api coverage  # Must calculate coverage
go test ./internal/spec/... ./internal/api/... -v  # All Phase 8 tests must pass
```

**What's needed:**
- Phase 8 spec and API management complete
- API validation working
- Spec validation working
- Code parsing utilities operational

**If missing:** Complete Phase 8 first

---

## Objective
Implement comprehensive cross-system parity validation - comparing specs to code, API docs to implementations, features to tests, and detecting all inconsistencies - providing the critical quality assurance that keeps Atlas documentation and code perfectly synchronized.

## Files
**Create:** `cmd/atlas-dev/validate_parity.go` (~200 lines - parity validation command)
**Create:** `cmd/atlas-dev/validate_all.go` (~150 lines - comprehensive validation)
**Create:** `cmd/atlas-dev/validate_tests.go` (~100 lines - test validation)
**Create:** `cmd/atlas-dev/validate_consistency.go` (~150 lines - consistency checks)
**Create:** `internal/parity/code_analyzer.go` (~300 lines - parse and analyze Rust code)
**Create:** `internal/parity/spec_matcher.go` (~250 lines - match spec to code)
**Create:** `internal/parity/api_matcher.go` (~250 lines - match API to code)
**Create:** `internal/parity/test_analyzer.go` (~200 lines - analyze test coverage)
**Create:** `internal/parity/ref_validator.go` (~150 lines - cross-reference validation)
**Create:** `internal/parity/parity_checker.go` (~400 lines - comprehensive parity check)
**Update:** `cmd/atlas-dev/validate.go` (add parity validation commands)

## Dependencies
- Phase 1 infrastructure (DB, JSON output)
- Phase 7 feature management (code parsing)
- Phase 8 spec and API management (parsers, validators)
- Rust code analysis capability
- All Phase 1-8 acceptance criteria met

## Implementation

### Code Analyzer
Implement Rust code analyzer in internal/parity/code_analyzer.go to extract complete code structure. Parse Rust source files to extract: public functions (names, signatures, parameters, return types, file locations), structs (fields, methods), enums (variants, methods), traits (methods), impl blocks, tests (test functions, test modules). Use regex patterns or tree-sitter for parsing. Build CodeAnalysis struct with all extracted items. Include source location (file, line number) for each item. Handle generics, lifetimes, and complex types. Return structured representation of entire codebase.

### Spec Matcher
Implement spec-to-code matcher in internal/parity/spec_matcher.go. Parse spec documents to extract language feature definitions and requirements. For each spec requirement, search code analysis for corresponding implementation. Match spec definitions to code structures: spec type definition → Rust struct/enum, spec function → Rust function, spec behavior → implementation. Compare signatures, types, and semantics. Generate match report with: matches (spec requirement + code location), mismatches (spec requirement without implementation, implementation without spec, signature differences). Include fix suggestions for each mismatch.

### API Matcher
Implement API-to-code matcher in internal/parity/api_matcher.go. Parse API docs to extract documented functions and signatures. Compare against code analysis. For each API function: verify implementation exists, compare signature (parameters, types, return type), verify error types match, check if function is public and accessible. For each code function: check if documented in API. Calculate match percentage. Generate detailed mismatch report with: documented but not implemented, implemented but not documented, signature differences. Include file:line locations and specific differences.

### Test Analyzer
Implement test analyzer in internal/parity/test_analyzer.go to validate test coverage claims. Parse test files to count: total test functions, tests per module, tests per feature, test assertions. Parse phase files to extract test count requirements. Compare actual test counts to phase requirements. Parse feature docs to extract claimed test counts. Verify test counts match across: phase requirements, feature docs, actual test files. Generate test mismatch report with: phases with insufficient tests (required vs actual), features with incorrect test counts. Include file:line locations of test files.

### Cross-Reference Validator
Implement cross-reference validator in internal/parity/ref_validator.go. Scan all documentation files for references: markdown links [text](path), spec references (docs/specification/file.md#section), API references (docs/api/file.md#function), phase references. For each reference, verify: target file exists, target section/anchor exists if specified, no broken links. Build reference graph showing document interconnections. Detect orphaned documents (not referenced anywhere). Generate broken reference report with: source location, target reference, error type (file missing, section missing), fix suggestion.

### Parity Checker
Implement comprehensive parity checker in internal/parity/parity_checker.go. Run all validation subsystems: spec-to-code parity, API-to-code parity, feature-to-implementation parity, test count validation, cross-reference validation. Aggregate all results into unified ParityReport. Calculate overall health score based on: percentage of specs implemented, percentage of API documented, percentage of references valid, test coverage percentage. Return detailed report with: ok boolean (true if all checks pass), total checks run, passed/failed counts, errors array (all mismatches), warnings array (non-critical issues), details per subsystem. Generate actionable fix suggestions for each error.

### Validation Report Generation
Create validation report generator that formats parity results for AI consumption. Generate compact JSON with: summary (ok, health score, checks/passed/failed counts), errors array (type, severity, source, issue, fix), warnings array, details map (per-subsystem results). For each error, include: error type (spec_code_mismatch, api_code_mismatch, test_count_mismatch, broken_reference), severity (error or warning), precise location (file:line), clear issue description, actionable fix suggestion (specific command or change needed). Format for < 500 token output with all essential information.

### Parity Command
Implement validate parity command as the main parity checker. Run comprehensive CheckParity function. Display results with exit code 0 if all checks pass, exit code 3 if validation failures found. Return detailed JSON report. Support --detailed flag for extended report with all subsystem details. Support --fix-suggestions flag to include SQL/shell commands to fix issues. Query time < 30s for complete validation (may be slower than other commands due to code parsing).

### Validate All Command
Implement validate all command to run every validation system. Execute: validate parity (spec/code/API/docs), validate database consistency (from Phase 4), validate references, validate links, validate tests, validate grammar. Aggregate results into single health report. Calculate overall health score (0-100). Return comprehensive JSON with: ok (true if everything passes), health score, results per validator (ok, score, issues count), summary message. Exit code 0 only if all validators pass.

### Test Validation Command
Implement validate tests command to check test coverage requirements. Parse all phase files to extract test count requirements (e.g., "Minimum test count: 35 tests"). Parse all test files to count actual tests per phase/module. Compare required vs actual. Generate report with: phases meeting requirements (green), phases below requirements (red with deficit), total test count, coverage by category. Return compact JSON. Include specific file locations for adding tests.

### Consistency Command
Implement validate consistency command to detect internal documentation conflicts. Check for inconsistencies: feature doc says X functions but API doc says Y, spec says type A but code implements type B, phase requires N tests but feature shows M. Build consistency graph across all documentation. Detect contradictions. Return conflict report with: conflicting sources, conflicting values, recommended resolution (which source is likely correct based on code ground truth).

## Tests (TDD)

**Code analyzer tests:**
1. Parse Rust functions
2. Extract signatures correctly
3. Find structs and enums
4. Locate tests
5. Handle generics
6. File:line locations accurate

**Spec matcher tests:**
1. Match spec to code
2. Detect missing implementations
3. Detect spec-code mismatches
4. Fix suggestions generated

**API matcher tests:**
1. Match API to code
2. Detect undocumented functions
3. Signature mismatches found
4. Coverage calculated

**Test analyzer tests:**
1. Count tests accurately
2. Compare to phase requirements
3. Detect insufficient tests
4. Report deficits

**Reference validator tests:**
1. Detect broken links
2. Validate spec references
3. Validate section anchors
4. Find orphaned documents

**Parity checker tests:**
1. Run all subsystems
2. Aggregate results correctly
3. Health score calculated
4. Report comprehensive

**Integration tests:**
1. Validate parity end-to-end
2. Validate all command works
3. JSON output valid
4. Exit codes correct
5. Fix suggestions actionable

**Minimum test count:** 40 tests
**Coverage target:** 80%+ on internal/parity

## Integration Points
- Uses: Spec parser from Phase 8
- Uses: API parser from Phase 8
- Uses: Feature parser from Phase 7
- Uses: Code parsing utilities from Phase 7
- Creates: Code analyzer
- Creates: Parity validation system
- Creates: Cross-system consistency checks
- Creates: Comprehensive quality assurance
- Output: Complete parity validation ensuring Atlas documentation perfectly matches code

## Acceptance
- atlas-dev validate parity runs comprehensive checks
- Spec-to-code parity validated
- API-to-code parity validated
- Feature-to-implementation parity validated
- Test count requirements validated
- Cross-references validated
- Broken links detected
- atlas-dev validate all runs all validators
- Overall health score calculated
- All subsystem results included
- atlas-dev validate tests checks test coverage
- Phase requirements compared to actual
- Deficits reported with locations
- atlas-dev validate consistency detects conflicts
- Contradictions found across docs
- Recommended resolutions provided
- All commands return detailed JSON reports
- Error messages include file:line locations
- Fix suggestions actionable and specific
- Exit codes correct (0=pass, 3=fail)
- 40+ tests pass
- 80%+ coverage on parity validation
- go test -race passes
- golangci-lint passes
- Parity validation completes in < 30s
- Code analyzer handles complex Rust syntax
- Spec matcher accurately compares definitions
- API matcher catches all mismatches
- Test analyzer counts accurately
- Reference validator finds all broken links
- Health score reflects actual project state
- Fix suggestions resolve actual issues

**Phase complete when all acceptance criteria met and make test lint passes.**

---

**Note:** Reference ARCHITECTURE.md for patterns. This is THE critical feature for keeping Atlas consistent. May require external libraries for robust Rust parsing (tree-sitter recommended). Graceful degradation if full parsing unavailable.
