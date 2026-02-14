# Method Call Syntax Implementation Status

**Created:** 2026-02-14
**Status:** Partial - Foundation work in progress

## What's Complete

### Lexer (✓ DONE)
- Added `TokenKind::Dot` to token.rs
- Updated lexer to recognize `.` as standalone token (not just in numbers)
- Compiles successfully

### AST (✓ DONE)
- Added `Expr::Member(MemberExpr)` variant
- Defined `MemberExpr` struct with:
  - `target: Box<Expr>` - left side of dot
  - `member: Identifier` - method/property name
  - `args: Option<Vec<Expr>>` - arguments if method call
  - `span: Span` - source location
- Updated `Expr::span()` method
- Compiles (with non-exhaustive match warnings in other components)

### Decision Log (✓ DONE)
- Documented Rust-style method approach
- Rationale: AI-friendly, type-safe, zero-cost abstraction
- Alternatives considered and rejected

## What Remains

### Parser (Phase 16)
- Parse `expr.identifier` syntax
- Parse `expr.identifier(args)` syntax
- Handle method call chaining
- Integrate with existing postfix expression parsing

### Binder (Phase 16)
- Add Member case to expression binding
- Recurse into target and args (no new symbols)

### Type Checker (Phase 16)
- Create method table infrastructure
- Implement method resolution
- Type-check method calls
- Register JSON methods: as_string, as_number, as_bool, is_null
- Generate appropriate error messages

### Interpreter (Phase 17)
- Evaluate Member expressions
- Desugar to function calls
- Call stdlib extraction functions

### Compiler (Phase 17)
- Compile Member expressions to bytecode
- Generate function call bytecode
- Maintain parity with interpreter

### VM (Phase 17)
- No changes needed (methods compile to existing call opcode)

### Stdlib (Phase 17)
- Register extraction functions: jsonAsString, jsonAsNumber, jsonAsBool, jsonIsNull
- These functions already implemented in stdlib/json.rs
- Just need registration in stdlib/mod.rs

### Tests (Phases 16-17)
- Parser tests (~30 tests)
- Type checker tests (~30 tests)
- Interpreter tests (~25 tests)
- VM tests (~25 tests)
- Parity verification tests
- Integration tests

## Files Modified So Far

**Complete:**
- `crates/atlas-runtime/src/token.rs` - Added Dot token
- `crates/atlas-runtime/src/lexer/mod.rs` - Recognize dot
- `crates/atlas-runtime/src/ast.rs` - Added MemberExpr
- `docs/reference/decision-log.md` - Documented decision

**Needs Updates (causing compilation errors):**
- `crates/atlas-runtime/src/parser/expr.rs` - Parse Member
- `crates/atlas-runtime/src/binder.rs` - Bind Member
- `crates/atlas-runtime/src/typechecker/expr.rs` - Type check Member
- `crates/atlas-runtime/src/interpreter/expr.rs` - Evaluate Member
- `crates/atlas-runtime/src/compiler/expr.rs` - Compile Member

## Current Build Status

**Compiles:** ✗ No (non-exhaustive match errors)

**Errors:** 5 non-exhaustive pattern matches for `Expr::Member`
- binder.rs
- typechecker/expr.rs (multiple places)
- interpreter/expr.rs
- compiler/expr.rs

**To make it compile:** Add stub match arms returning errors for now

## Phase Files Created

1. `phases/foundation/phase-16-method-call-syntax-frontend.md`
   - Parser implementation
   - Type checker implementation
   - Method table infrastructure
   - JSON method registration

2. `phases/foundation/phase-17-method-call-syntax-backend.md`
   - Interpreter implementation
   - Compiler implementation
   - Stdlib function registration
   - Comprehensive testing

## Next Steps

1. **Immediate:** Make codebase compile by adding stub implementations
2. **Phase 16:** Complete frontend (parser + type checking)
3. **Phase 17:** Complete backend (interpreter + VM + tests)
4. **Then:** Return to Phase 06 (stdlib integration tests) with working JSON extraction

## Why This is a Blocker

**Problem discovered:**
- Phase 06 requires JSON integration tests
- JSON values can be created and indexed but not extracted
- Extraction requires method syntax: `json.as_string()`
- Global functions like `jsonAsString()` are janky and not world-class

**World-class requirement:**
- Rust, TypeScript, Python all use method syntax
- Methods are fundamental to modern language design
- Required for professional stdlib API

**Impact:**
- Blocks Phase 06 (stdlib integration tests)
- Blocks all future stdlib work requiring methods
- Foundation work that should have been in v0.1

**Priority:** CRITICAL - Must complete before continuing v0.2
