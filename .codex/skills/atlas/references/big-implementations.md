# Big Implementations Guide

For large features, major refactors, or multi-component changes.

## Before You Start

1. **Understand the full scope**
   - Read all relevant spec sections in `docs/specification/`
   - Grep for existing related code
   - Identify ALL files that need changes

2. **Check for dependencies**
   - Does this need changes in multiple crates?
   - Are there type definitions that need updating first?
   - Is there a natural order (parser → typechecker → interpreter → VM)?

## Implementation Order

For features touching multiple components, follow this order:

1. **Types/AST** (`atlas-core/src/ast/`, `types.rs`)
   - Add new AST nodes, type variants
   - Update visitors if needed

2. **Parser** (`atlas-core/src/parser/`)
   - Parse the new syntax
   - Add tests for parsing

3. **Typechecker** (`atlas-runtime/src/typechecker/`)
   - Add type checking logic
   - Handle inference if needed

4. **Compiler** (`atlas-runtime/src/compiler/`)
   - Generate bytecode for VM
   - Add new opcodes if needed

5. **Interpreter** (`atlas-runtime/src/interpreter/`)
   - Execute the feature directly

6. **VM** (`atlas-runtime/src/vm/`)
   - Execute bytecode for the feature

7. **Tests** (`atlas-runtime/tests/`)
   - Add comprehensive tests
   - Test both interpreter AND VM

## Breaking Down Work

If the task is too large for one session:

1. **Identify milestones** - What's a working checkpoint?
2. **Implement to first milestone** - Get something working
3. **Commit that milestone** - Don't lose progress
4. **Document remaining work** in CODEX_VERIFY.md
5. **Create issues** for remaining milestones: `atlas-track add-issue`

## Parity Strategy

For features needing both interpreter and VM:

**Option A: Interpreter First**
1. Implement fully in interpreter
2. Add tests that pass with interpreter
3. Implement in VM
4. Verify same tests pass with `--vm`

**Option B: Parallel (for simple features)**
1. Implement in both simultaneously
2. Test both as you go

Choose based on complexity. Interpreter is usually easier to debug.

## When Feature is Too Big

If you realize mid-implementation that scope is larger than expected:

1. **Don't abandon work** - Commit what you have
2. **Document in CODEX_VERIFY.md**:
   - What's done
   - What's remaining
   - Any blockers discovered
3. **Create follow-up issues** for remaining work
4. **Mark status as PARTIAL** - Opus will review and decide next steps
