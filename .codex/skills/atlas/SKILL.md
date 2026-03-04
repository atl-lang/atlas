---
name: atlas
description: Atlas AI-first programming language compiler. Use for any Atlas development - bug fixes, features, refactoring. Rust codebase with interpreter/VM parity requirement.
---

# Atlas - Codex Development Skill

You are working on Atlas, an AI-first programming language compiler written in Rust.

## Mission
Make AI code generation effortless and reliable. Every design decision optimized for LLMs.

## Codebase Structure
```
crates/
├── atlas-core/        # AST, types, parser
├── atlas-runtime/     # Interpreter + VM execution
│   ├── src/
│   │   ├── interpreter/  # Tree-walking interpreter
│   │   ├── vm/           # Bytecode VM
│   │   ├── typechecker/  # Type system
│   │   └── stdlib/       # Standard library
│   └── tests/         # ALL tests go here (not in src/)
├── atlas-lsp/         # Language server
├── atlas-jit/         # JIT compiler
└── atlas-package/     # Package management
docs/specification/    # Language spec (source of truth)
```

## Critical Rules

### 1. Interpreter/VM Parity (SACRED)
Atlas has TWO execution engines that MUST produce IDENTICAL output:
- **Interpreter:** `crates/atlas-runtime/src/interpreter/`
- **VM:** `crates/atlas-runtime/src/vm/`

**If you change behavior in one, you MUST change it in the other.**

Test both:
```bash
atlas run /tmp/test.atl        # interpreter
atlas run --vm /tmp/test.atl   # VM
```
Outputs must match exactly.

### 2. CoW Write-Back Pattern
Collections use Copy-on-Write. Mutations return NEW collections:
```atlas
let arr = [1, 2, 3];
arr = push(arr, 4);  // Must reassign - push returns new array
```
In Rust code:
- Interpreter: `apply_cow_writeback()`
- VM: `emit_cow_writeback_if_needed()`

### 3. No Stubs Ever
- ❌ NO `unimplemented!()` macros
- ❌ NO `todo!()` comments
- ❌ NO stub functions
- ❌ NO partial implementations
- ❌ NO "MVP for now"

Complete implementations only. Every edge case handled.

### 4. Test Location
ALL tests go in `crates/atlas-runtime/tests/` - no inline `#[cfg(test)]` in src/ files.

### 5. Error Codes
Errors use codes defined in `crates/atlas-runtime/src/diagnostic.rs`.
Format: AT#### (e.g., AT3037 for trait bound errors)

## Your Workflow

### For Any Task:

1. **UNDERSTAND**
   - Read the issue/task carefully
   - Grep codebase to find relevant code
   - Read the files you'll modify
   - Check docs/specification/ if syntax/semantics involved

2. **IMPLEMENT**
   - Write complete implementation
   - Handle all edge cases
   - Follow existing patterns in codebase
   - If touching interpreter, also update VM (and vice versa)

3. **TEST**
   ```bash
   cargo build --workspace
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   ```

4. **INVESTIGATE FAILURES**
   If tests fail:
   - Read the error message
   - Find the root cause
   - Fix it
   - Re-run tests
   - Repeat until ALL pass

   This is YOUR job. Do not report failures - fix them.

5. **VERIFY MANUALLY**
   ```bash
   echo '[test code]' > /tmp/test.atl
   atlas run /tmp/test.atl
   atlas run --vm /tmp/test.atl  # verify parity
   ```

6. **COMMIT**
   ```bash
   git add -A
   git commit -m "fix(component): description (ISSUE-ID)"
   ```

7. **CREATE VERIFICATION REPORT**
   Create `CODEX_VERIFY.md` with:
   - What you did
   - Test output (paste actual results)
   - Parity verification
   - Commit hash

## Atlas Syntax Quick Reference

```atlas
// Variables
let x: number = 42;
let name: string = "atlas";
let flag: bool = true;

// Functions
fn add(a: number, b: number) -> number {
    return a + b;
}

// Generics with trait bounds
fn identity<T: Copy>(x: T) -> T {
    return x;
}

// Collections
let arr: [number] = [1, 2, 3];
let map: HashMap<string, number> = hashMapNew();

// Control flow
if condition {
    // ...
} else {
    // ...
}

for item in collection {
    // ...
}

while condition {
    // ...
}

// Pattern matching
match value {
    1 => print("one"),
    2 => print("two"),
    _ => print("other"),
}

// Records (structs)
record Point {
    x: number,
    y: number,
}

// Traits
trait Printable {
    fn to_string(self: Self) -> string;
}

impl Printable for Point {
    fn to_string(self: Point) -> string {
        return "Point";
    }
}
```

## Key Commands

```bash
# Build
cargo build --workspace

# Test
cargo test --workspace
cargo test test_name           # specific test
cargo test --test corpus       # corpus tests

# Lint
cargo clippy --workspace -- -D warnings
cargo fmt --check

# Run Atlas code
atlas run file.atl             # interpreter
atlas run --vm file.atl        # VM
atlas check file.atl           # type check only

# Issue tracking
atlas-track issue H-001        # view issue
atlas-track claim H-001        # claim issue
# DO NOT close issues - Opus verifies and closes
```

## Quality Standards

1. **Every feature works in BOTH interpreter AND VM**
2. **Every edge case handled** (empty input, invalid input, boundaries)
3. **Clear error messages** with proper error codes
4. **Tests added** for new functionality
5. **No regressions** - all existing tests still pass

## When Stuck

1. Grep for similar patterns: `rg "pattern" crates/`
2. Check the spec: `docs/specification/`
3. Look at existing tests for examples
4. Read error messages carefully - they often tell you exactly what's wrong

## Handling Other Failures

If you encounter test failures or errors UNRELATED to your task:
1. **If quick to fix** (< 5 min) → Fix them too
2. **If blocking your task** → Fix them (you can't proceed otherwise)
3. **If complex and not blocking** → Check if issue exists: `atlas-track issues`
   - If no issue exists → Create one: `atlas-track add-issue "title" component P1 "problem" "needed"`
   - Then continue with your task

Fix what's blocking. Log the rest.

## When to Stop and Report

Stop working and create CODEX_VERIFY.md with status "BLOCKED" if:
1. **Architectural decision needed** - You need guidance on HOW to implement, not just fixing bugs
2. **Spec unclear** - The specification doesn't define the expected behavior
3. **Circular dependency** - Fixing A breaks B, fixing B breaks A, after 3+ attempts
4. **Missing infrastructure** - Needs external dependency, API, or tooling you can't add

Do NOT stop for:
- Test failures (fix them)
- Compile errors (fix them)
- Complex code (figure it out)
- Multiple files to change (do it)

**For large implementations** (new features, major refactors): Read `.codex/skills/atlas/references/big-implementations.md`

## DO NOT

- Close issues (Opus does this after verification)
- Leave partial implementations
- Skip parity (interpreter must match VM)
- Ignore test failures
- Use unimplemented!() or todo!()
