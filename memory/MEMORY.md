# Atlas Memory System

**Purpose:** Consolidated knowledge base for AI agents working on Atlas compiler.

**Auto-loaded:** Files in this directory provide patterns, decisions, and workflows.

---

## File Index

### Core Documentation

**patterns.md** - Codebase implementation patterns
- Collection types (`Rc<RefCell<T>>`)
- Intrinsic pattern (callback-based, interpreter + VM)
- Stdlib function pattern (non-intrinsic)
- Error handling
- Helper functions
- Test harness

**testing-patterns.md** - Testing strategies and guidelines
- Integration test pattern
- Testing intrinsics with callbacks
- Testing collections
- Parity verification (interpreter vs VM)
- Parameterized tests (rstest)
- Snapshot tests (insta)
- Property-based tests (proptest)
- Atlas language semantics

**decisions.md** - Architectural decision log
- DR-001: Interpreter + VM dual execution
- DR-002: Reference semantics for collections
- DR-003: Hash function design
- DR-004: HashMap key equality
- DR-005: Collection API design
- DR-006: Collection benchmarking (deferred)
- DR-007: Phase file accuracy
- DR-008: Scope sizing for phases

**gates.md** - Quality gate definitions
- GATE -1: Sanity check (always first)
- GATE 0: Declaration
- GATE 1: Implementation
- GATE 2: Testing
- GATE 3: Parity
- GATE 4: Quality (clippy, fmt)
- GATE 5: Documentation
- GATE 6: Handoff
- Testing protocol (-- --exact during dev)

---

## Quick Reference

### For AI Agents Starting Work

1. **Check STATUS.md** - Current phase and progress
2. **Read phase file** - Requirements and acceptance criteria
3. **Run GATE -1** - Sanity check before starting
4. **Reference patterns.md** - Implementation patterns
5. **Reference testing-patterns.md** - Testing guidance
6. **Check decisions.md** - Architectural context
7. **Follow gates.md** - Quality checkpoints
8. **Update STATUS.md** - On completion

### Project Structure

```
atlas/
├── crates/atlas-runtime/       # Core runtime
│   ├── src/
│   │   ├── value.rs            # Value enum (all types)
│   │   ├── interpreter/        # Interpreter engine
│   │   │   └── expr.rs         # Intrinsics here
│   │   ├── vm/                 # VM engine
│   │   │   └── mod.rs          # VM intrinsics here
│   │   └── stdlib/             # Standard library
│   │       ├── mod.rs          # Function registration
│   │       ├── collections/    # HashMap, HashSet, etc.
│   │       └── {module}.rs     # Other stdlib modules
│   └── tests/                  # Integration tests
├── phases/                     # Work queue (~100 lines each)
├── docs/specification/         # Language spec
├── memory/                     # This directory
└── STATUS.md                   # Single source of truth
```

### Key File Locations

**Runtime core:**
- `crates/atlas-runtime/src/value.rs` - All Atlas types
- `crates/atlas-runtime/src/stdlib/mod.rs` - Function registration (is_builtin, is_array_intrinsic)
- `crates/atlas-runtime/src/interpreter/expr.rs` - Interpreter intrinsics
- `crates/atlas-runtime/src/vm/mod.rs` - VM intrinsics

**Collections:**
- `crates/atlas-runtime/src/stdlib/collections/hashmap.rs` - HashMap impl
- `crates/atlas-runtime/src/stdlib/collections/hashset.rs` - HashSet impl
- `crates/atlas-runtime/src/stdlib/collections/hash.rs` - Hash infrastructure
- `crates/atlas-runtime/src/stdlib/collections/queue.rs` - Queue impl
- `crates/atlas-runtime/src/stdlib/collections/stack.rs` - Stack impl

**Tests:**
- `crates/atlas-runtime/tests/collection_iteration_tests.rs` - Collection iteration
- `crates/atlas-runtime/tests/` - All integration tests

**Specifications:**
- `docs/specification/syntax.md` - Grammar and syntax
- `docs/specification/types.md` - Type system
- `docs/specification/runtime.md` - Runtime behavior

---

## Decision Quick Lookup

**Need hash function details?** → DR-003
**Need collection API design?** → DR-005
**Why Rc<RefCell<>>?** → DR-002
**Why interpreter + VM?** → DR-001
**Phase file issues?** → DR-007

---

## Pattern Quick Lookup

**Implementing intrinsic?** → patterns.md "Intrinsic Pattern"
**Implementing stdlib function?** → patterns.md "Stdlib Function Pattern"
**Error handling?** → patterns.md "Error Pattern"
**Type checking?** → patterns.md "Helper Pattern"

**Testing intrinsics?** → testing-patterns.md "Testing Intrinsics"
**Testing collections?** → testing-patterns.md "Testing Collections"
**Parity testing?** → testing-patterns.md "Testing Parity"

---

## Gate Quick Lookup

**Starting phase?** → GATE -1 (sanity check)
**Implementation done?** → GATE 1-2 (tests)
**Tests pass?** → GATE 3 (parity)
**Before handoff?** → GATE 4-6 (quality, docs, status)

**Testing protocol:**
- During dev: `cargo test -p atlas-runtime test_name -- --exact`
- Before handoff: `cargo test -p atlas-runtime` (full suite once)

---

## Common Commands

### Development

```bash
# Sanity check (GATE -1)
cargo clean && cargo check -p atlas-runtime

# Run specific test (during dev)
cargo test -p atlas-runtime test_name -- --exact

# Format code
cargo fmt -p atlas-runtime

# Lint (zero warnings required)
cargo clippy -p atlas-runtime -- -D warnings
```

### Before Handoff

```bash
# Full test suite (GATE 2/3)
cargo test -p atlas-runtime

# Verify all gates
cargo fmt -p atlas-runtime
cargo clippy -p atlas-runtime -- -D warnings
cargo build -p atlas-runtime --release
```

---

## AI Workflow Summary

**Execution Mode (Default):**
1. User says "Next: Phase-XX" or STATUS.md shows next phase
2. Run GATE -1 immediately
3. Declare workflow type (GATE 0)
4. Execute gates 0-6 without asking for permission
5. Deliver handoff summary (user may engage here)

**Key Principles:**
- Autonomous execution (no "should I proceed?" questions)
- 100% spec compliance
- All acceptance criteria met
- Zero shortcuts (no TODOs, no stubs)
- World-class quality

---

## Notes

**Token Efficiency:**
- Memory files are designed to be referenced, not re-read every time
- Use Grep to find specific patterns: `grep -r "pattern" memory/`
- Each file is standalone (can read individually)

**Maintenance:**
- Update patterns.md when new patterns emerge
- Add decisions to decisions.md (use DR-XXX format)
- Update testing-patterns.md for new testing approaches
- Keep gates.md in sync with actual workflow

**History:**
- Created: 2025-02-16 (workflow audit identified gaps)
- Purpose: Prevent phase file assumptions, document actual patterns
- Scope: Project-specific (not ~/.claude/memory)

---

## For Humans

This memory system is optimized for AI agents. If you're human:

- Ask AI to summarize: "Explain memory/patterns.md"
- Ask AI to find info: "Where's the intrinsic pattern?"
- Ask AI for status: "What's documented in memory/decisions.md?"

The memory system consolidates:
- Old docs/decision-logs/ → memory/decisions.md
- Old docs/gates/ → memory/gates.md
- Scattered examples → memory/patterns.md (from actual code)
- Test knowledge → memory/testing-patterns.md

**Everything is derived from actual codebase, not assumptions.**
