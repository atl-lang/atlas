# Atlas Code Quality Standards

**Purpose:** Comprehensive code quality, style, and development standards for Atlas

**For AI Agents:** These are the quality gates and style rules. Follow these standards for all code contributions.

---

## Table of Contents
1. [Code Style](#code-style)
2. [Architecture Standards](#architecture-standards)
3. [Phase Gates](#phase-gates)
4. [Quality Checklist](#quality-checklist)

---

## Code Style

### Naming Conventions

**Modules:** `snake_case`
- Example: `type_checker`, `symbol_table`, `ast_builder`

**Types:** `PascalCase`
- Example: `Value`, `Diagnostic`, `AstNode`
- Enum variants: `PascalCase`

**Functions:** `snake_case`
- Example: `parse_expression`, `check_types`, `emit_bytecode`

**Variables:** `snake_case`
- Example: `symbol_table`, `current_scope`, `error_count`

**Constants:** `SCREAMING_SNAKE_CASE`
- Example: `MAX_ERRORS`, `BYTECODE_VERSION`, `DEFAULT_CAPACITY`

### File Organization

**One primary module per file**
- Keep related functionality together
- Split large modules into submodules

**File size limit:** < 400 lines preferred
- Larger files need justification
- Consider refactoring if approaching limit
- Known exceptions documented in CODE_ORGANIZATION.md

**Module structure:**
```
src/
  module_name/
    mod.rs        // Public interface
    internal.rs   // Internal implementation
    tests.rs      // Module tests
```

### Comment Policy

**Only add comments for non-obvious logic**
- Self-documenting code is preferred
- Explain WHY, not WHAT
- Complex algorithms need explanation

**Good comments:**
```rust
// Use binary search because array is sorted
let index = binary_search(&items, target);

// SAFETY: Pointer is valid because we just allocated it
unsafe { ptr.write(value) }
```

**Bad comments:**
```rust
// Increment counter
counter += 1;

// Return the result
return result;
```

### Formatting

**Use `rustfmt`:**
- Run `cargo fmt` before committing
- No manual formatting overrides
- Consistent across entire codebase

**Use `clippy`:**
- Run `cargo clippy -- -D warnings`
- Fix ALL warnings
- No `#[allow(...)]` without justification

---

## Architecture Standards

### Boundaries

**atlas-runtime is library-first:**
- No CLI logic in runtime
- No file I/O in core types
- Pure data structures and algorithms

**atlas-cli is thin wrapper:**
- Minimal logic in CLI
- Delegate to runtime APIs
- Handle user interaction only

**Frontend separate from runtime:**
- Lexer/parser/binder/typechecker are frontend
- Interpreter/VM are runtime
- Clear boundary between compilation and execution

### Error Handling

**Single diagnostic pipeline:**
- All errors flow through `Diagnostic` type
- Every diagnostic includes span, code, message
- Support both human and JSON formats

**Diagnostic requirements:**
- Precise error locations
- Helpful error messages
- Related spans for context
- Suggested fixes when possible

### Testing Strategy

**Unit tests for each component:**
- Lexer tests for tokenization
- Parser tests for AST construction
- Typechecker tests for type rules
- Interpreter/VM tests for execution

**Golden tests for end-to-end:**
- Input → Output verification
- Snapshot testing with `insta`
- Cross-platform consistency

**No flaky tests:**
- No time-based assertions
- Deterministic test data
- Reproducible failures

### REPL Architecture

**REPL core is UI-agnostic:**
- Shared by all frontends
- Pure evaluation logic
- No terminal-specific code

**REPL UI is thin layer:**
- Handles input/output only
- Uses rustyline for line editing
- Minimal presentation logic

---

## Phase Gates

### Purpose
Ensure features exist before tests and prevent premature or stubbed work.

### Gate Rules

**Test phases blocked by feature phases:**
- A test phase may NOT start until corresponding feature phase is complete
- Diagnostics must be implemented before diagnostics test phase
- Bytecode format must be defined before bytecode tests

**Feature completeness required:**
- Passing minimal end-to-end examples for the feature
- Diagnostics for invalid inputs emitted with correct codes
- No TODO/TBD markers in feature code

**Evidence required:**
- Demonstrable working examples
- Error cases handled properly
- Code is complete, not stubbed

### Enforcement

**Phase execution compliance:**
- Each phase has BLOCKERS section checking gate requirements
- If gate not satisfied, STOP and update plan
- Do not proceed without meeting gate requirements
- See `STATUS.md` for phase progress tracking

**Quality over velocity:**
- Gates prevent rushing
- Ensure solid foundations
- No compromises on completeness

### Examples

**GOOD:**
```
Phase 1: Implement type checker    ✓ Complete
Phase 2: Test type checker         ← Can proceed
```

**BAD:**
```
Phase 1: Implement type checker    ⚠️ Stubbed only
Phase 2: Test type checker         ← BLOCKED!
```

---

## Quality Checklist

### Before Committing Code

**Static Analysis:**
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] All compiler warnings resolved

**Testing:**
- [ ] All tests pass: `cargo test`
- [ ] New features have tests
- [ ] Tests are deterministic (not flaky)
- [ ] Snapshot tests updated if needed

**Code Quality:**
- [ ] No TODO/FIXME in production code
- [ ] Comments explain non-obvious logic
- [ ] Function/file sizes reasonable
- [ ] Code is self-documenting

**Architecture:**
- [ ] Follows architecture boundaries
- [ ] Uses diagnostic pipeline for errors
- [ ] Maintains REPL separation
- [ ] No CLI logic in runtime

### Before Marking Phase Complete

**Implementation:**
- [ ] All blockers checked and met
- [ ] All files created/updated as specified
- [ ] Code follows architecture notes
- [ ] All specific test cases implemented

**Testing:**
- [ ] Acceptance criteria met (100%)
- [ ] Tests pass: `cargo test`
- [ ] No clippy warnings
- [ ] Interpreter/VM parity maintained (if applicable)

**Documentation:**
- [ ] Relevant docs updated
- [ ] API changes documented
- [ ] Examples work correctly

**Quality:**
- [ ] No shortcuts taken
- [ ] No compromises on design
- [ ] Built right, not fast
- [ ] Ready for next phase to build on

---

## Security & Safety

### Unsafe Code

**Policy:**
- No `unsafe` without explicit approval
- Document safety invariants
- Justify need for unsafe

**Exceptions:**
- Performance-critical paths (with benchmarks)
- FFI boundaries (when needed)
- Low-level optimizations (proven necessary)

### Dependencies

**Minimal and vetted:**
- Keep external dependencies minimal
- Prefer standard library solutions
- Vet security of dependencies

**Auditing:**
- Run `cargo audit` regularly
- Check for known vulnerabilities
- Update dependencies promptly

**See:** `DEPENDENCIES.md` for dependency policy

---

## Rust Standards

**Stable Rust only:**
- No nightly features
- Rust edition 2021
- Standard library preferred

**Edition policy:**
- Upgrade edition when stable
- No unnecessary edition features
- Document edition requirements

---

## File Size Guidelines

**Preferred limits:**
- Modules: < 400 lines
- Functions: < 100 lines
- Test files: < 500 lines

**Known exceptions:**
- `vm/mod.rs` - 1972 lines (planned refactor v0.3)
- `bytecode/mod.rs` - 1421 lines (planned refactor v0.3)

**When exceeding limits:**
- Document reason in CODE_ORGANIZATION.md
- Plan refactoring for future phase
- Do not compromise readability

**See:** `docs/CODE_ORGANIZATION.md` for complete file size tracking

---

## Implementation References

**For implementation details, see:**
- `docs/implementation/` - Detailed implementation guides
- `docs/CODE_ORGANIZATION.md` - File size tracking and refactoring plans
- `docs/engineering.md` - Original engineering standards (archived)
- `docs/testing.md` - Comprehensive testing strategy

---

**Summary:** Atlas maintains high code quality through strict standards, phase gates, comprehensive testing, and architectural discipline. Quality over speed, always.
