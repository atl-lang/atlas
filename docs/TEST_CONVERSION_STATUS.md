# Atlas Test Conversion - Current Status & Next Steps

**Date:** 2026-02-12
**Status:** Ready to begin conversion
**Priority:** HIGH - Critical for v0.1 production readiness

---

## 📊 Current Situation

### What We Have ✅
- **Modern dependencies installed:** rstest, insta, proptest, pretty_assertions
- **Test helpers created:** `tests/common/mod.rs` with assertion utilities
- **Proof of concept:** `lexer_tests_modern.rs` shows 61% line reduction
- **All tests passing:** 1,227 passing, 8 ignored (0 failures)

### What We Need to Fix ⚠️
- **12,000 lines of boilerplate-heavy tests** (70% redundancy)
- **God files:** `interpreter_tests.rs` is 2,150 lines!
- **Incomplete modern features:**
  - `diagnostic_golden.rs` - has unused functions (warnings)
  - `snapshot_tests.rs` - has unused functions (warnings)
- **No modularization:** Flat structure, hard to navigate
- **Manual snapshot testing:** No automation, inconsistent

---

## 🎯 Goals

### Technical Goals
1. **Reduce boilerplate:** 12,000 → 2,800 lines (77% reduction)
2. **Increase coverage:** 1,235 → 1,500+ tests
3. **Modularize tests:** Split into focused 150-300 line files
4. **Automate snapshots:** Use insta for all structural tests
5. **Zero test loss:** Every test preserved or improved

### Quality Goals
- **Max file size:** 300 lines (enforced, like our code)
- **One feature per file:** No god files
- **Fast tests:** All tests < 2 seconds total
- **Easy to review:** Table-driven tests, clear structure

---

## 📋 Conversion Plan Summary

### Phase 1: Quick Wins (Week 1) - MODULAR
**Files to convert:**
1. ✅ `lexer_tests.rs` → Already done (`lexer_tests_modern.rs`)
   - **Action:** Remove old file, rename modern version
   - **Split into:** `tests/unit/frontend/lexer/` directory
     - `keywords.rs` (~100 lines)
     - `string_literals.rs` (~80 lines)
     - `number_literals.rs` (~60 lines)
     - `operators.rs` (~70 lines)
     - `mod.rs` (re-exports)

2. `keyword_policy_tests.rs` (316 lines)
   - **Convert to:** `tests/unit/frontend/parser/keyword_policy.rs` (~80 lines)

3. `lexer_golden_tests.rs` (152 lines)
   - **Convert to:** `tests/snapshots/lexer.rs` (~40 lines with insta)

4. `operator_precedence_tests.rs` (448 lines)
   - **Convert to:** `tests/unit/frontend/parser/precedence.rs` (~100 lines)

**Expected result:** ~1,400 lines → ~400 lines across 8 focused files

### Phase 2-6: See `TEST_CONVERSION_PLAN.md`

---

## 🏗️ Target Test Structure

```
tests/
├── unit/                      # Fast unit tests
│   ├── frontend/
│   │   ├── lexer/             # Split lexer tests by feature
│   │   │   ├── keywords.rs
│   │   │   ├── string_literals.rs
│   │   │   ├── number_literals.rs
│   │   │   └── mod.rs
│   │   ├── parser/            # Split parser tests by feature
│   │   │   ├── expressions.rs
│   │   │   ├── statements.rs
│   │   │   ├── functions.rs
│   │   │   └── mod.rs
│   │   └── mod.rs
│   ├── typing/                # Type system tests
│   ├── diagnostics/           # Diagnostic tests
│   └── mod.rs
│
├── integration/               # End-to-end tests
│   ├── interpreter/           # Split by operation type
│   │   ├── arithmetic.rs      # ~100 lines (not 2,150!)
│   │   ├── strings.rs
│   │   ├── arrays.rs
│   │   ├── functions.rs
│   │   └── mod.rs
│   ├── vm/
│   └── mod.rs
│
├── snapshots/                 # Snapshot tests with insta
│   ├── ast/
│   ├── bytecode/
│   └── diagnostics/
│
└── common/                    # Shared utilities
    └── mod.rs
```

**Rules:**
- ✅ Max 300 lines per file
- ✅ One feature per file
- ✅ Organized by domain
- ✅ Easy to navigate

---

## 🚀 Getting Started - Immediate Next Steps

### Step 1: Clean Up Modern Lexer Tests (5 minutes)
The modern lexer tests already exist but need to be integrated:

```bash
# Remove old lexer tests
rm crates/atlas-runtime/tests/lexer_tests.rs

# Keep modern version
# (already exists as lexer_tests_modern.rs)

# Verify tests still pass
cargo test lexer_tests_modern
```

### Step 2: Fix Incomplete Snapshot Files (15 minutes)
Address the warnings about unused functions:

**Option A:** Complete the implementation
```bash
# Review and fix:
# - crates/atlas-runtime/tests/diagnostic_golden.rs
# - crates/atlas-runtime/tests/snapshot_tests.rs
```

**Option B:** Remove if not needed
```bash
# If they're not being used and were experimental:
rm crates/atlas-runtime/tests/diagnostic_golden.rs
rm crates/atlas-runtime/tests/snapshot_tests.rs
```

### Step 3: Convert First File - Proof of Process (1-2 hours)
Pick the easiest file to validate the process:

**Recommended:** `keyword_policy_tests.rs` (316 lines → ~80 lines)

```bash
# Create modern version
cat > crates/atlas-runtime/tests/keyword_policy_modern.rs << 'EOF'
//! Modern keyword policy tests using rstest
//! Converted from keyword_policy_tests.rs

mod common;
use common::*;
use rstest::rstest;

#[rstest]
#[case("let")]
#[case("var")]
#[case("fn")]
// ... all keywords
fn test_keywords_reserved(#[case] keyword: &str) {
    let source = format!("let {} = 1;", keyword);
    assert_error_code(&source, "AT3002"); // or whatever the code is
}

// More parameterized tests...
EOF

# Verify parity
cargo test keyword_policy        # Old tests
cargo test keyword_policy_modern # New tests

# If new tests pass and coverage is same/better:
rm crates/atlas-runtime/tests/keyword_policy_tests.rs
mv crates/atlas-runtime/tests/keyword_policy_modern.rs \
   crates/atlas-runtime/tests/keyword_policy_tests.rs
```

### Step 4: Create Modular Structure (30 minutes)
Set up the directory structure for modular tests:

```bash
cd crates/atlas-runtime/tests

# Create directory structure
mkdir -p unit/frontend/lexer
mkdir -p unit/frontend/parser
mkdir -p unit/typing
mkdir -p unit/diagnostics
mkdir -p integration/interpreter
mkdir -p integration/vm
mkdir -p snapshots/ast
mkdir -p snapshots/bytecode

# Create module files
cat > unit/mod.rs << 'EOF'
pub mod frontend;
pub mod typing;
pub mod diagnostics;
EOF

cat > unit/frontend/mod.rs << 'EOF'
pub mod lexer;
pub mod parser;
EOF

# Similar for other directories...
```

### Step 5: Convert One Large File with Splitting (2-3 hours)
Take `interpreter_tests.rs` (2,150 lines) and split it:

```bash
# Analyze the file to find natural groupings
grep "^fn test_" interpreter_tests.rs | sed 's/test_//' | cut -d_ -f1 | sort | uniq -c

# Example output:
#  45 arithmetic
#  32 string
#  28 array
#  35 function
#  25 control
# ... etc

# Create focused files:
# integration/interpreter/arithmetic.rs (~100 lines)
# integration/interpreter/strings.rs (~90 lines)
# integration/interpreter/arrays.rs (~120 lines)
# integration/interpreter/functions.rs (~110 lines)
# integration/interpreter/control_flow.rs (~100 lines)
```

---

## ✅ Quality Checklist

Before removing old tests, verify:

### Coverage Verification
```bash
# Count old tests
OLD_COUNT=$(grep -c "^#\[test\]" old_file.rs)

# Count new test cases
NEW_COUNT=$(grep -c "#\[case" new_file.rs)

# Verify: NEW_COUNT >= OLD_COUNT
```

### Functional Verification
```bash
# Both must pass
cargo test old_file::
cargo test new_file::

# Total count should not decrease
cargo test -- --list | wc -l
```

### Code Quality
- [ ] No file > 300 lines
- [ ] Each file tests one feature
- [ ] Tests use rstest for parameterization
- [ ] Snapshots use insta (not manual)
- [ ] Comments preserved for tricky tests
- [ ] Line count reduced by 60%+

---

## 📊 Expected Impact

### Before
```
interpreter_tests.rs - 2,150 lines
parser_tests.rs - 789 lines
type_rules_tests.rs - 735 lines
... 21 more files
Total: ~12,000 lines, 24 files
```

### After
```
integration/interpreter/
  arithmetic.rs - 100 lines
  strings.rs - 90 lines
  arrays.rs - 120 lines
  ... 7 focused files
unit/frontend/parser/
  expressions.rs - 150 lines
  statements.rs - 180 lines
  ... 4 focused files
unit/typing/
  type_rules.rs - 180 lines
  ... 5 focused files
Total: ~2,800 lines, 45-50 focused files
```

### Metrics
- **Lines:** 12,000 → 2,800 (77% reduction)
- **Files:** 24 → 50 (better organization)
- **Avg file size:** 500 lines → 150 lines
- **Max file size:** 2,150 lines → 300 lines
- **Tests:** 1,235 → 1,500+ (better coverage)
- **Time to add test:** 10 lines → 1 line

---

## 🔧 Helper Scripts

### verify_test_parity.sh
```bash
#!/bin/bash
# Usage: ./verify_test_parity.sh old_file.rs new_file.rs

OLD=$1
NEW=$2

echo "Verifying test parity..."
OLD_COUNT=$(grep -c "^#\[test\]" "$OLD")
NEW_COUNT=$(grep -c "#\[case" "$NEW")

echo "Old tests: $OLD_COUNT"
echo "New cases: $NEW_COUNT"

if [ "$NEW_COUNT" -ge "$OLD_COUNT" ]; then
    echo "✅ Parity verified"
else
    echo "⚠️ WARNING: Test count decreased!"
    exit 1
fi

cargo test --test "$(basename "$OLD" .rs)" || exit 1
cargo test --test "$(basename "$NEW" .rs)" || exit 1

echo "✅ All tests pass!"
```

### analyze_test_file.sh
```bash
#!/bin/bash
# Usage: ./analyze_test_file.sh test_file.rs

FILE=$1

echo "Analyzing $FILE..."
echo "Lines: $(wc -l < "$FILE")"
echo "Tests: $(grep -c "^#\[test\]" "$FILE")"
echo ""
echo "Test groupings:"
grep "^fn test_" "$FILE" | sed 's/test_//' | cut -d_ -f1 | sort | uniq -c
```

---

## 📚 Key Documents

1. **This Document** - Current status and next steps
2. **TEST_CONVERSION_PLAN.md** - Detailed 6-week conversion roadmap
3. **TEST_MODERNIZATION_PLAN.md** - Original vision and motivation
4. **tests/common/mod.rs** - Test helper utilities
5. **tests/lexer_tests_modern.rs** - Example of modern testing

---

## 🎯 Success Criteria

**Weekly Goals:**
- Week 1: 4 files converted, directory structure created
- Week 2: Snapshot testing fully working
- Week 3: Semantic tests modernized
- Week 4: Interpreter tests split and modernized
- Week 5: Runtime tests complete
- Week 6: All tests modern, documentation updated

**Final Success:**
- ✅ All 1,235+ tests passing
- ✅ No test coverage lost
- ✅ All files < 300 lines
- ✅ Modular organization by feature
- ✅ Automated snapshot testing
- ✅ 77% less boilerplate

---

## 🚨 Warning Signs

Stop and reassess if you see:
- ⚠️ Total test count decreasing
- ⚠️ Tests failing after conversion
- ⚠️ Any file > 300 lines
- ⚠️ Unclear what a test file tests
- ⚠️ Related tests in different files
- ⚠️ Unrelated tests in same file

---

## 🤝 Need Help?

**Stuck on:**
- Splitting a large file? Use `analyze_test_file.sh` to find groupings
- Verifying parity? Use `verify_test_parity.sh`
- Writing rstest tests? See `lexer_tests_modern.rs`
- Snapshot testing? See `TEST_MODERNIZATION_PLAN.md`

**Questions:**
- How do I split file X? → Look for natural groupings (arithmetic, strings, etc.)
- Should I use snapshots? → Yes, for AST/bytecode/diagnostics output
- What's the max file size? → 300 lines (enforced)
- Can I combine tests? → Only if testing the same feature

---

**Ready to start? Follow Step 1 above! 🚀**
