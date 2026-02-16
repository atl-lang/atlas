# Phase 08: Spec & API Management

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Phase 7 must be 100% complete

**Verification:**
```bash
atlas-dev feature list  # Must return valid JSON
atlas-dev feature validate --help  # Must show validate command
go test ./internal/feature/... -v  # All Phase 7 tests must pass
```

**What's needed:**
- Phase 7 feature management complete
- Feature validation working
- Feature sync functional
- Markdown parsing utilities operational

**If missing:** Complete Phase 7 first

---

## Objective
Implement specification and API documentation management - parsing spec documents and API references, validating grammar definitions, comparing API docs to code implementation, generating API docs from code, and tracking documentation coverage - ensuring specs and APIs remain synchronized with code.

## Files
**Create:** `cmd/atlas-dev/spec.go` (~150 lines - spec command group)
**Create:** `cmd/atlas-dev/spec_read.go` (~100 lines - read command)
**Create:** `cmd/atlas-dev/spec_search.go` (~100 lines - search command)
**Create:** `cmd/atlas-dev/spec_validate.go` (~150 lines - validate command)
**Create:** `cmd/atlas-dev/spec_grammar.go` (~150 lines - grammar command)
**Create:** `cmd/atlas-dev/api.go` (~150 lines - API command group)
**Create:** `cmd/atlas-dev/api_read.go` (~100 lines - read command)
**Create:** `cmd/atlas-dev/api_validate.go` (~150 lines - validate command)
**Create:** `cmd/atlas-dev/api_generate.go` (~150 lines - generate command)
**Create:** `cmd/atlas-dev/api_coverage.go` (~100 lines - coverage command)
**Create:** `internal/spec/parser.go` (~300 lines - parse spec markdown)
**Create:** `internal/spec/grammar.go` (~250 lines - validate EBNF grammar)
**Create:** `internal/api/parser.go` (~300 lines - parse API markdown)
**Create:** `internal/api/validator.go` (~250 lines - validate against code)
**Create:** `internal/api/generator.go` (~300 lines - generate API from code)
**Update:** `cmd/atlas-dev/main.go` (add spec and API command groups)

## Dependencies
- Phase 1 infrastructure (DB, JSON output)
- Phase 5 context (markdown parser utilities)
- Phase 7 feature management (code parsing utilities)
- EBNF grammar parsing (optional external library)
- All Phase 1-7 acceptance criteria met

## Implementation

### Spec Parser
Implement spec markdown parser in internal/spec/parser.go to extract structured data from docs/specification/ files. Parse spec documents to extract: title, sections (hierarchical by heading level), content per section, code blocks (with language tags), grammar rules (EBNF definitions in code blocks), cross-references to other specs. Build section tree preserving hierarchy. Extract EBNF grammar blocks specially for validation. Return Spec struct with sections array and grammar rules array. Use markdown parser utilities from Phase 5.

### Spec Read Command
Implement spec read subcommand taking spec path and optional --section argument. Parse spec markdown file using parser. If section specified, return only that section with content and code blocks. If no section, return document outline (title and section headings). Support --with-code flag to include code blocks in output. Return compact JSON with spec metadata and requested content. Query < 10ms by caching parsed specs.

### Spec Search Command
Implement spec search subcommand taking search query. Search across all spec files in docs/specification/ directory. Use LIKE operator or full-text search across section titles and content. Return matching sections with: spec file, section name, relevance score, snippet of matching text. Order by relevance. Support --spec filter to search within specific spec. Return compact JSON array. Index spec content for faster searching (optional).

### Spec Validate Command
Implement spec validate subcommand to check spec consistency. Validate: all code blocks have valid syntax if language specified, all cross-references point to existing files/sections, all internal links valid, grammar rules complete (no undefined references), no broken spec references. Parse markdown to find all reference patterns: [text](path), [text](path#section), spec-name#section. Verify each reference target exists. Return validation report with errors array listing broken references with line numbers and fix suggestions.

### Spec Grammar Command
Implement spec grammar subcommand to validate EBNF grammar definitions. Parse all EBNF code blocks from grammar spec file. Extract grammar rules (production name and definition). Validate: EBNF syntax correct, all referenced non-terminals are defined, no circular dependencies (or mark them), no ambiguous productions (warn if detected). Return grammar validation report with: total rules count, valid boolean, errors array, warnings array. Optionally compare grammar to parser implementation if --compare-to-parser flag provided. **Reference external EBNF validator if available.**

### API Parser
Implement API markdown parser in internal/api/parser.go to extract structured data from docs/api/ files. Parse API documents to extract: function definitions (name, signature, parameters, return type, errors), type definitions, module structure. Parse function sections to extract: signature line, parameter descriptions, return value description, example code blocks, notes/warnings. Build APIDoc struct with functions array and types array. Handle various API doc formats gracefully.

### API Read Command
Implement API read subcommand taking API doc path and optional --function filter. Parse API markdown file using parser. If function specified, return only that function's documentation. If no function, return summary: total functions count, functions grouped by category/module. Support --detailed flag for full documentation. Return compact JSON. Cache parsed API docs for performance.

### API Validate Command
Implement API validate subcommand to compare API docs against code implementation. Parse API docs to extract all documented functions and their signatures. Parse Rust code to extract all public functions and their actual signatures. Compare: every documented function exists in code, signatures match (parameters and return types), every public function is documented. Return validation report with: match count, mismatch array (documented but missing in code, in code but not documented, signature mismatch), coverage percentage. Include specific file:line references for mismatches.

### API Generate Command
Implement API generate subcommand to auto-generate API docs from Rust code. Parse Rust source files in specified directory (e.g., crates/atlas-runtime/src/stdlib/). Extract public functions with doc comments. Extract function signatures (parameters and return types). Build API documentation markdown from extracted data. Format as standard API doc with sections per module. Write to output file or stdout. Return JSON with functions count and output path. Support --update flag to merge with existing docs rather than replace.

### API Coverage Command
Implement API coverage subcommand to track documentation coverage. Parse Rust code to count all public functions. Parse API docs to count documented functions. Calculate coverage percentage. Group by module/category for detailed breakdown. Return compact JSON with: total functions, documented count, coverage percentage, missing functions array (with file:line locations). Flag critical gaps (stdlib functions without docs). Query < 10ms using cached data.

### Grammar Validation Logic
Implement EBNF grammar validation in internal/spec/grammar.go. Parse EBNF syntax to extract rules and productions. Build dependency graph of non-terminals. Validate all referenced non-terminals are defined. Detect circular dependencies using graph traversal. Check for common EBNF syntax errors. Return detailed validation report with specific errors and line numbers. Support multiple EBNF syntax variants (W3C, ISO, custom).

### API Validation Logic
Implement API validation in internal/api/validator.go. Parse API signature strings to extract types and parameters. Parse Rust function signatures from code. Normalize both for comparison (handle type aliases, generics). Build matcher that compares normalized signatures. Generate detailed mismatch reports with: expected signature (from docs), actual signature (from code), difference explanation. Include fix suggestions (update doc or update code).

### API Generation Logic
Implement API generation in internal/api/generator.go. Parse Rust syntax tree to extract public items (functions, structs, enums, traits). Extract doc comments (/// and //!). Convert Rust doc comments to markdown format. Build structured API doc sections per module. Generate function signature lines in readable format. Include examples from doc comment examples. Format output as clean markdown. Support templates for customization.

## Tests (TDD)

**Spec parser tests:**
1. Parse complete spec file
2. Extract section hierarchy
3. Parse code blocks with language tags
4. Extract EBNF grammar blocks
5. Handle missing sections
6. Cross-references extracted

**Spec read tests:**
1. Read full spec document
2. Read specific section
3. Include code blocks if requested
4. Section not found handled
5. Output compact

**Spec search tests:**
1. Search across all specs
2. Search within specific spec
3. Relevance scoring works
4. Results include snippets
5. Query < 10ms

**Spec validate tests:**
1. Broken references detected
2. Valid references pass
3. Line numbers included
4. Fix suggestions provided

**Spec grammar tests:**
1. EBNF syntax validated
2. Undefined non-terminals detected
3. Circular dependencies detected
4. Validation report detailed
5. Multiple syntax variants supported

**API parser tests:**
1. Parse API functions
2. Extract signatures
3. Parse parameter descriptions
4. Code examples extracted
5. Multiple formats handled

**API validate tests:**
1. Documented functions found in code
2. Missing implementations detected
3. Signature mismatches detected
4. Coverage calculated correctly
5. File:line references accurate

**API generate tests:**
1. Generate from Rust code
2. Doc comments extracted
3. Signatures formatted correctly
4. Markdown output valid
5. Module structure preserved

**API coverage tests:**
1. Coverage percentage accurate
2. Missing functions listed
3. Grouped by module
4. Query < 10ms

**Minimum test count:** 45 tests
**Coverage target:** 80%+ on internal/spec, internal/api

## Integration Points
- Uses: Markdown parser from Phase 5
- Uses: Code parsing utilities from Phase 7
- Creates: Spec management system
- Creates: API documentation system
- Creates: Grammar validation
- Creates: API generation from code
- Creates: Documentation coverage tracking
- Output: Complete spec and API management ensuring code/doc synchronization

## Acceptance
- atlas-dev spec read returns spec content
- Section filtering works
- atlas-dev spec search finds matches across specs
- Relevance scoring reasonable
- atlas-dev spec validate detects broken references
- Cross-reference validation works
- atlas-dev spec grammar validates EBNF
- EBNF syntax errors detected
- Undefined non-terminals caught
- atlas-dev API read returns API documentation
- Function filtering works
- atlas-dev API validate compares docs to code
- Missing functions detected
- Signature mismatches detected
- Coverage percentage accurate
- atlas-dev API generate creates docs from code
- Doc comments extracted correctly
- Generated markdown valid
- atlas-dev API coverage tracks documentation gaps
- Missing functions listed with locations
- All commands return compact JSON
- Exit codes correct (0-6)
- Structured errors with suggestions
- 45+ tests pass
- 80%+ coverage on spec/API management
- go test -race passes
- golangci-lint passes
- Spec parser handles format variations
- Grammar validator supports multiple EBNF syntaxes
- API validator accurately compares signatures
- API generator produces clean markdown
- Coverage tracking identifies critical gaps

**Phase complete when all acceptance criteria met and make test lint passes.**

---

**Note:** Reference ARCHITECTURE.md for patterns. EBNF validation may use external library or regex-based parser. Rust parsing may use tree-sitter or regex patterns with graceful degradation.
