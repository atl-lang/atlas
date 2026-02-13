# Atlas Test Conversion Plan - Zero Loss Strategy

**Status:** Ready to Execute
**Date:** 2026-02-12
**Goal:** Convert 12,000 lines of tests to modern format WITHOUT losing any coverage

---

## 🎯 Core Principle: Parallel Track Conversion

**Strategy:** Keep old tests running while building modern replacements, then swap atomically.

**Why this works:**
- Old tests keep passing → we never break the build
- New tests are validated before old ones are removed
- Easy to compare coverage between old and new
- Zero risk of losing critical tests

---

## 📊 Current State

### Test Files by Category

#### Frontend Tests (3,693 lines)
- `lexer_tests.rs` - 502 lines → **Already has modern version!** (`lexer_tests_modern.rs`)
- `parser_tests.rs` - 789 lines
- `operator_precedence_tests.rs` - 448 lines
- `assignment_target_tests.rs` - 457 lines
- `ast_instantiation.rs` - 450 lines
- `parser_error_tests.rs` - 354 lines
- `keyword_policy_tests.rs` - 316 lines
- `lexer_golden_tests.rs` - 152 lines (needs insta conversion)
- `lexer_tests_modern.rs` - 193 lines (MODERN - keep as-is)

#### Typing/Semantic Tests (4,177 lines)
- `type_rules_tests.rs` - 735 lines
- `scope_shadowing_tests.rs` - 782 lines
- `nullability_tests.rs` - 494 lines
- `function_return_analysis_tests.rs` - 681 lines
- `related_spans_tests.rs` - 337 lines
- `diagnostic_ordering_tests.rs` - 300 lines
- `warning_tests.rs` - 144 lines
- `typecheck_dump_stability_tests.rs` - 346 lines
- `diagnostic_golden.rs` - 152 lines (⚠️ has unused functions)
- `snapshot_tests.rs` - 213 lines (⚠️ has unused functions)

#### Runtime Tests (3,333 lines)
- `interpreter_tests.rs` - 2,150 lines (LARGEST - 40% reduction expected)
- `value_model_tests.rs` - 548 lines
- `numeric_edge_cases_tests.rs` - 484 lines
- `repl_state_tests.rs` - 373 lines
- `bytecode_compiler_integration.rs` - 261 lines
- `runtime_api.rs` - 160 lines

#### Infrastructure (188 lines)
- `common/mod.rs` - 188 lines (MODERN - keep as-is)

**Total:** ~12,000 lines (excluding common helpers)

---

## 🏗️ Modularization Philosophy

**CRITICAL:** No god files! Just like our code, tests should be:
- **Focused:** One file per concept/feature (max 300 lines)
- **Cohesive:** Related tests grouped together
- **Well-named:** Clear what the file tests
- **Independent:** Can run in isolation

**Example Structure:**
```
tests/
├── frontend/
│   ├── lexer/
│   │   ├── keywords.rs         # ~100 lines
│   │   ├── string_literals.rs  # ~80 lines
│   │   ├── number_literals.rs  # ~60 lines
│   │   ├── operators.rs        # ~70 lines
│   │   └── mod.rs             # Re-exports
│   ├── parser/
│   │   ├── expressions.rs
│   │   ├── statements.rs
│   │   ├── functions.rs
│   │   └── errors.rs
│   └── mod.rs
├── typing/
│   ├── type_rules.rs
│   ├── scopes.rs
│   ├── nullability.rs
│   └── mod.rs
├── runtime/
│   ├── interpreter/
│   │   ├── arithmetic.rs       # ~100 lines
│   │   ├── strings.rs          # ~80 lines
│   │   ├── arrays.rs           # ~120 lines
│   │   ├── functions.rs        # ~90 lines
│   │   └── control_flow.rs     # ~110 lines
│   ├── vm/
│   │   ├── bytecode.rs
│   │   └── execution.rs
│   └── mod.rs
└── common/
    └── mod.rs
```

**Why This Matters:**
- **Faster compilation:** Small files compile in parallel
- **Easier review:** Focused changes, clear scope
- **Better navigation:** Find tests by feature
- **Prevents merge conflicts:** Different features in different files

---

## 🔄 Conversion Process (Per File)

### Step 1: Analyze Coverage & Plan Splitting
```bash
# Count test cases in old file
grep -c "^#\[test\]" old_file.rs

# List all test functions to identify natural groupings
grep "^fn test_" old_file.rs | head -20

# Identify natural splits (e.g., arithmetic, strings, arrays)
grep "^fn test_" old_file.rs | sed 's/test_//' | cut -d_ -f1 | sort | uniq -c
```

**Decision:** If file > 500 lines or > 50 tests, split into logical modules

### Step 2: Create Modern Version (Modular!)
```rust
// File: {category}/{subcategory}.rs (NOT a god file!)
// Example: tests/runtime/interpreter/arithmetic.rs

mod common;
use common::*;
use rstest::rstest;

// ============================================================================
// Basic Arithmetic - Addition, Subtraction
// ============================================================================

#[rstest]
#[case("1 + 2", 3.0)]
#[case("10 - 3", 7.0)]
#[case("5.5 + 2.5", 8.0)]
fn test_basic_arithmetic(#[case] input: &str, #[case] expected: f64) {
    assert_eval_number(input, expected);
}

// File: tests/runtime/interpreter/strings.rs (separate file!)
// ... string tests here

// File: tests/runtime/interpreter/mod.rs
pub mod arithmetic;
pub mod strings;
pub mod arrays;
pub mod functions;
pub mod control_flow;
```

### Step 3: Verify Parity
```bash
# Count test cases in new file
grep -c "#\[case" new_file_modern.rs

# Run both old and new tests
cargo test old_file::
cargo test new_file_modern::

# Compare coverage
# OLD count should equal NEW count (or NEW should be higher)
```

### Step 4: Atomic Swap
```bash
# Only when new tests are proven complete:
git mv old_file.rs old_file.rs.archived
git mv new_file_modern.rs old_file.rs

# Or if keeping original name:
rm old_file.rs  # Only after verifying new tests!
mv new_file_modern.rs new_file.rs
```

### Step 5: Verify No Loss
```bash
# Total test count should be same or higher
cargo test --test old_file -- --list | wc -l
```

---

## 📋 Conversion Priority Order

### Phase 1: Quick Wins (Week 1)
**Goal:** Prove the process works with easiest conversions

1. ✅ **lexer_tests.rs** → Already done! (`lexer_tests_modern.rs` exists)
   - Action: Remove old `lexer_tests.rs`, rename modern version

2. **keyword_policy_tests.rs** (316 lines → ~80 lines expected)
   - Highly repetitive keyword checks
   - Perfect for rstest parameterization

3. **lexer_golden_tests.rs** (152 lines → ~40 lines with insta)
   - Convert manual golden file checks to insta snapshots

4. **operator_precedence_tests.rs** (448 lines → ~100 lines)
   - Repetitive precedence test patterns

**Expected:** ~1,400 lines → ~400 lines (71% reduction)

### Phase 2: Snapshot Testing (Week 2)
**Goal:** Convert all structural output tests to insta

5. **parser_tests.rs** (789 lines → ~200 lines)
   - AST output → insta snapshots
   - Reduce manual assertions

6. **typecheck_dump_stability_tests.rs** (346 lines → ~80 lines)
   - Perfect candidate for snapshot testing

7. **Fix incomplete snapshot files:**
   - `snapshot_tests.rs` - Complete the implementation
   - `diagnostic_golden.rs` - Integrate or remove unused functions

**Expected:** ~1,300 lines → ~350 lines (73% reduction)

### Phase 3: Semantic Tests (Week 3)
**Goal:** Convert repetitive semantic validation tests

8. **type_rules_tests.rs** (735 lines → ~180 lines)
   - Repetitive type checking patterns

9. **scope_shadowing_tests.rs** (782 lines → ~200 lines)
   - Similar scoping scenarios

10. **nullability_tests.rs** (494 lines → ~120 lines)
    - Null checking patterns

**Expected:** ~2,000 lines → ~500 lines (75% reduction)

### Phase 4: Interpreter Tests (Week 4) - WITH MODULARIZATION
**Goal:** Split biggest file into focused modules

11. **interpreter_tests.rs** (2,150 lines) → Split into 7 focused files:
    - **BIGGEST FILE - MUST SPLIT!**
    - `tests/integration/interpreter/arithmetic.rs` (~100 lines)
      - Basic operations: +, -, *, /, %
      - Operator precedence
      - Unary operators
    - `tests/integration/interpreter/logical.rs` (~80 lines)
      - Boolean operations: &&, ||, !
      - Truthiness
      - Short-circuit evaluation
    - `tests/integration/interpreter/strings.rs` (~90 lines)
      - String literals
      - String operations
      - String concatenation
    - `tests/integration/interpreter/arrays.rs` (~120 lines)
      - Array creation
      - Array indexing
      - Array methods
      - Mutation
    - `tests/integration/interpreter/functions.rs` (~110 lines)
      - Function declarations
      - Function calls
      - Closures
      - Recursion
    - `tests/integration/interpreter/control_flow.rs` (~100 lines)
      - If/else statements
      - While loops
      - For loops
      - Break/continue
    - `tests/integration/interpreter/mod.rs` (~20 lines)
      - Re-exports and shared test setup

**Expected:** ~2,150 lines → ~620 lines across 7 files (71% reduction + better organization)

### Phase 5: Remaining Runtime Tests (Week 5)
**Goal:** Complete the runtime test modernization

12. **value_model_tests.rs** (548 lines → ~150 lines)
13. **numeric_edge_cases_tests.rs** (484 lines → ~120 lines)
14. **repl_state_tests.rs** (373 lines → ~100 lines)
15. **bytecode_compiler_integration.rs** (261 lines → ~80 lines)

**Expected:** ~1,600 lines → ~450 lines (72% reduction)

### Phase 6: Advanced Semantic Tests (Week 6)
**Goal:** Complex analysis tests with snapshots

16. **function_return_analysis_tests.rs** (681 lines → ~180 lines)
17. **related_spans_tests.rs** (337 lines → ~100 lines)
18. **diagnostic_ordering_tests.rs** (300 lines → ~90 lines)
19. **warning_tests.rs** (144 lines → ~50 lines)

**Expected:** ~1,500 lines → ~420 lines (72% reduction)

---

## ✅ Quality Gates (Per File)

Before removing old tests, verify:

### Coverage Verification
```bash
# Count old tests
OLD_COUNT=$(grep -c "^#\[test\]" old_file.rs)

# Count new test cases (including parameterized)
NEW_COUNT=$(grep -c "#\[case" new_file.rs)

# NEW_COUNT must be >= OLD_COUNT
if [ $NEW_COUNT -lt $OLD_COUNT ]; then
    echo "⚠️ WARNING: Test count decreased!"
    echo "Old: $OLD_COUNT, New: $NEW_COUNT"
    exit 1
fi
```

### Functional Verification
```bash
# Both must pass
cargo test old_file::
cargo test new_file::

# Total test count should not decrease
BEFORE=$(cargo test -- --list | grep -c "test ")
# ... after swap ...
AFTER=$(cargo test -- --list | grep -c "test ")

if [ $AFTER -lt $BEFORE ]; then
    echo "⚠️ WARNING: Total test count decreased!"
    exit 1
fi
```

### Code Review Checklist
- [ ] Every old test has equivalent new test case
- [ ] Test names/descriptions are clear
- [ ] Error cases are preserved
- [ ] Edge cases are preserved
- [ ] Comments explaining tricky tests are preserved
- [ ] Line count reduced by 60%+ (or using snapshots)

---

## 🎯 Expected Final Results

### Before (Current)
- **Lines:** ~12,000
- **Tests:** ~1,235
- **Boilerplate:** ~70%
- **Files:** 24 test files (some > 2,000 lines!)
- **Organization:** Flat, hard to navigate
- **Snapshot testing:** Manual/incomplete
- **Property testing:** None
- **Benchmarks:** None

### After (Target)
- **Lines:** ~2,800 (77% reduction)
- **Tests:** ~1,500+ (20% increase in coverage)
- **Boilerplate:** ~10%
- **Files:** ~45-50 focused test files (150-300 lines each)
- **Organization:** ✅ Modular by feature (frontend/, typing/, runtime/)
- **Snapshot testing:** ✅ Fully automated with insta
- **Property testing:** ✅ Added for critical paths
- **Benchmarks:** ✅ Criterion benchmarks set up

### File Size Guidelines (Enforced!)
- **Max file size:** 300 lines (same as code!)
- **Target file size:** 150-250 lines
- **If > 300 lines:** Split into logical modules
- **Example:** `interpreter_tests.rs` (2,150 lines) → 7 files (~150 lines each)

### Productivity Gains
- **Adding new test:** 1-2 lines (was 10-15 lines)
- **Reviewing tests:** Much easier (table format)
- **Maintaining snapshots:** `cargo insta review` (was manual diffing)
- **Finding regressions:** Automatic (was manual)

---

## 🔧 Tools & Commands

### Daily Workflow
```bash
# Run all tests
cargo test

# Run specific modernized file
cargo test --test interpreter_modern

# Run with better output
cargo test -- --nocapture

# Review snapshot changes
cargo insta review

# Accept all snapshot updates
cargo insta accept

# Reject all snapshot updates
cargo insta reject
```

### Conversion Helper Script
```bash
#!/bin/bash
# convert_test_file.sh

OLD_FILE=$1
NEW_FILE="${OLD_FILE%.rs}_modern.rs"

echo "Converting $OLD_FILE to modern format..."
echo "1. Count old tests: $(grep -c "^#\[test\]" "$OLD_FILE")"
echo "2. Create $NEW_FILE with rstest/insta"
echo "3. Verify parity"
echo "4. Run: ./verify_test_parity.sh $OLD_FILE $NEW_FILE"
```

### Parity Verification Script
```bash
#!/bin/bash
# verify_test_parity.sh

OLD=$1
NEW=$2

OLD_COUNT=$(grep -c "^#\[test\]" "$OLD")
NEW_COUNT=$(grep -c "#\[case" "$NEW")

echo "Old tests: $OLD_COUNT"
echo "New cases: $NEW_COUNT"

if [ "$NEW_COUNT" -ge "$OLD_COUNT" ]; then
    echo "✅ Parity verified (or improved)"
else
    echo "⚠️ WARNING: New file has fewer tests!"
    exit 1
fi

echo "Running old tests..."
cargo test --test "$(basename "$OLD" .rs)" || exit 1

echo "Running new tests..."
cargo test --test "$(basename "$NEW" .rs)" || exit 1

echo "✅ Both test suites pass!"
```

---

## 🏛️ Target Test Organization Structure

```
tests/
├── unit/                      # Fast, isolated unit tests (< 0.1s)
│   ├── frontend/
│   │   ├── lexer/
│   │   │   ├── keywords.rs             # ~100 lines
│   │   │   ├── string_literals.rs      # ~80 lines
│   │   │   ├── number_literals.rs      # ~60 lines
│   │   │   ├── operators.rs            # ~70 lines
│   │   │   ├── comments.rs             # ~50 lines
│   │   │   └── mod.rs                  # Re-exports
│   │   ├── parser/
│   │   │   ├── expressions.rs          # ~150 lines
│   │   │   ├── statements.rs           # ~180 lines
│   │   │   ├── functions.rs            # ~120 lines
│   │   │   ├── errors.rs               # ~100 lines
│   │   │   ├── precedence.rs           # ~90 lines
│   │   │   └── mod.rs
│   │   └── mod.rs
│   ├── typing/
│   │   ├── type_rules.rs               # ~180 lines
│   │   ├── scopes.rs                   # ~200 lines
│   │   ├── nullability.rs              # ~120 lines
│   │   ├── function_returns.rs         # ~150 lines
│   │   ├── warnings.rs                 # ~80 lines
│   │   └── mod.rs
│   ├── diagnostics/
│   │   ├── spans.rs                    # ~100 lines
│   │   ├── ordering.rs                 # ~90 lines
│   │   ├── formatting.rs               # ~80 lines
│   │   └── mod.rs
│   └── mod.rs
│
├── integration/               # End-to-end tests (< 1s)
│   ├── interpreter/
│   │   ├── arithmetic.rs               # ~100 lines
│   │   ├── logical.rs                  # ~80 lines
│   │   ├── strings.rs                  # ~90 lines
│   │   ├── arrays.rs                   # ~120 lines
│   │   ├── functions.rs                # ~110 lines
│   │   ├── control_flow.rs             # ~100 lines
│   │   └── mod.rs
│   ├── vm/
│   │   ├── bytecode.rs                 # ~80 lines
│   │   ├── execution.rs                # ~100 lines
│   │   ├── optimization.rs             # ~90 lines
│   │   └── mod.rs
│   ├── repl/
│   │   ├── state.rs                    # ~100 lines
│   │   ├── multiline.rs                # ~80 lines
│   │   └── mod.rs
│   └── mod.rs
│
├── snapshots/                 # Insta snapshot tests
│   ├── ast/
│   │   ├── expressions.rs              # ~60 lines
│   │   ├── statements.rs               # ~70 lines
│   │   └── mod.rs
│   ├── bytecode/
│   │   ├── basic.rs                    # ~50 lines
│   │   ├── functions.rs                # ~60 lines
│   │   └── mod.rs
│   ├── diagnostics/
│   │   ├── errors.rs                   # ~80 lines
│   │   ├── warnings.rs                 # ~50 lines
│   │   └── mod.rs
│   └── mod.rs
│
├── property/                  # Property-based tests (slow, later phase)
│   ├── arithmetic_properties.rs        # ~100 lines
│   ├── string_properties.rs            # ~80 lines
│   ├── parser_fuzzing.rs               # ~120 lines
│   └── mod.rs
│
├── common/
│   ├── mod.rs                          # Shared test utilities (~200 lines)
│   ├── assertions.rs                   # Custom assertions (~150 lines)
│   └── fixtures.rs                     # Test data (~100 lines)
│
└── .../                       # Old tests (delete after conversion)
```

**File Size Rules:**
- ✅ **Target:** 100-200 lines per test file
- ✅ **Max:** 300 lines per test file
- ❌ **Never:** > 300 lines (split it!)
- ✅ **Module files (mod.rs):** 20-50 lines (just re-exports)

**Organization Rules:**
- ✅ Group by **feature domain** (lexer, parser, interpreter)
- ✅ One feature per file (keywords, not "lexer tests")
- ✅ Related tests in same directory
- ✅ Use `mod.rs` to re-export for convenience

---

## 🚨 Critical Safety Rules

### NEVER Do This
❌ Delete old tests before new tests are proven
❌ Reduce test coverage to save lines
❌ Skip edge cases to simplify
❌ Combine unrelated test cases
❌ Remove tests that "seem redundant" without analysis
❌ **Create files > 300 lines** (split them!)
❌ **Mix unrelated features** in one file

### ALWAYS Do This
✅ Keep old tests running until new tests pass
✅ Count tests before and after conversion
✅ Verify same or better coverage
✅ Preserve edge cases and error cases
✅ Document any intentional coverage changes
✅ **Split files > 300 lines** into focused modules
✅ **One feature per file** (e.g., `arithmetic.rs`, not `interpreter.rs`)

### When in Doubt
1. Keep both old and new tests temporarily
2. Ask for review before deleting old tests
3. Check git history if unsure about test purpose
4. Better to have redundant coverage than gaps

---

## 📝 Conversion Template

```rust
//! Modern {component} tests using rstest and insta
//!
//! Converted from {old_file}.rs on {date}
//! Original: {old_line_count} lines, {old_test_count} tests
//! Modern: {new_line_count} lines, {new_test_count} cases
//! Reduction: {percentage}%

mod common;
use common::*;
use rstest::rstest;

// ============================================================================
// {Category} Tests - Converted from old {function_range}
// ============================================================================

#[rstest]
#[case("input1", expected1)]  // From: test_{old_name_1}
#[case("input2", expected2)]  // From: test_{old_name_2}
fn test_{category}(#[case] input: &str, #[case] expected: Type) {
    assert_{category}(input, expected);
}

// ============================================================================
// Snapshot Tests - Replacing manual assertions
// ============================================================================

#[test]
fn snapshot_{category}() {
    let output = generate_output("input");
    insta::assert_yaml_snapshot!(output);
}
```

---

## 🎯 Success Criteria

### Per-File Success
- ✅ New test count ≥ Old test count
- ✅ All new tests passing
- ✅ Line count reduced by 60%+
- ✅ No functionality lost
- ✅ Better readability

### Project-Wide Success
- ✅ Total tests: 1,235+ (current) → 1,500+ (target)
- ✅ Test code: 12,000 lines → 2,800 lines
- ✅ All tests passing
- ✅ CI/CD unchanged (or faster)
- ✅ Documentation updated

### Development Experience
- ✅ Adding test: < 2 lines
- ✅ Reviewing changes: Visual snapshot diffs
- ✅ Running tests: < 2 seconds for all tests
- ✅ Confidence: Higher (more coverage, less boilerplate)

---

## 📅 Timeline

| Week | Phase | Files | Lines Reduced | Cumulative % |
|------|-------|-------|---------------|--------------|
| 1 | Quick Wins | 4 | 1,000 → 300 | 8% |
| 2 | Snapshots | 4 | 1,300 → 350 | 19% |
| 3 | Semantics | 3 | 2,000 → 500 | 31% |
| 4 | Interpreter | 1 | 2,150 → 500 | 45% |
| 5 | Runtime | 4 | 1,600 → 450 | 59% |
| 6 | Advanced | 4 | 1,500 → 420 | 68% |

**Total:** 6 weeks to complete conversion (can be done in parallel with other work)

---

## 🚀 Next Steps

### Immediate (Today)
1. Review and approve this plan
2. Archive this plan in git
3. Pick first file to convert (recommended: `keyword_policy_tests.rs`)

### Week 1
1. Convert `keyword_policy_tests.rs` (proof of concept)
2. Create helper scripts (`convert_test_file.sh`, `verify_test_parity.sh`)
3. Document learnings and update this plan
4. Continue with remaining Phase 1 files

### Ongoing
- Keep old tests until new tests are proven
- Update `TEST_MODERNIZATION_PLAN.md` with progress
- Track metrics (lines saved, tests added, time saved)
- Celebrate wins! 🎉

---

## 📖 Reference

### Key Documents
- `docs/TEST_MODERNIZATION_PLAN.md` - Original vision
- `tests/common/mod.rs` - Test helpers
- `tests/lexer_tests_modern.rs` - Modern example
- This document - Conversion roadmap

### Tools
- **rstest** - Parameterized tests
- **insta** - Snapshot testing
- **proptest** - Property-based testing (later)
- **criterion** - Benchmarking (later)

---

**Ready to start? Pick a file from Phase 1 and let's modernize! 🚀**
