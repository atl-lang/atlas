# AI Agent Pre-Phase Checklist

**Run this checklist BEFORE starting any phase.**

---

## 📋 Step 1: Read Entry Point

```bash
# Always start here
cat STATUS.md
```

**Check:**
- [ ] What is the current phase?
- [ ] Are there any blocking items?
- [ ] What implementation guides do I need?

---

## 📏 Step 2: File Size Check (MANDATORY)

```bash
# Check for god files
find crates/*/src -name "*.rs" -not -path "*/tests/*" -exec wc -l {} + | sort -rn | head -10
```

**Verify:**
- [ ] No file exceeds 1000 lines (BLOCKING)
- [ ] No file in 800-1000 range without refactoring plan

**If any file exceeds 1000 lines:**
🚫 **STOP. Do not proceed. Refactor first.**
📖 See: `docs/CODE_ORGANIZATION.md`

---

## 📖 Step 3: Read Phase File

```bash
# Read the current phase file (path from STATUS.md)
cat phases/[section]/phase-XX-name.md
```

**Check:**
- [ ] Understand objective
- [ ] Review deliverables
- [ ] Note exit criteria

---

## 📚 Step 4: Read Implementation Guides

**Listed in STATUS.md under "Implementation Files Needed"**

Common guides by section:
- **Frontend:** `docs/implementation/03-lexer.md`, `04-parser.md`, `05-ast.md`
- **Typing:** `docs/implementation/06-symbol-table.md`, `07-typechecker.md`
- **Bytecode/VM:** `docs/implementation/11-bytecode.md`, `12-vm.md`

---

## ⚠️ Step 5: Syntax Verification (If Writing Atlas Code)

```bash
# If phase includes Atlas test code or examples
cat Atlas-SPEC.md | grep -A 10 "function signatures\|let/var\|semicolons"
```

**Verify:**
- [ ] Function syntax: `fn name(param: Type): ReturnType { }`
- [ ] Variable syntax: `let x = 5;` (immutable), `var y = 10;` (mutable)
- [ ] Semicolons required for statements

---

## ✅ Step 6: Pre-Implementation Check

**Before writing code:**
- [ ] File size check passed (no files >1000 lines)
- [ ] Phase file read and understood
- [ ] Implementation guides read
- [ ] Atlas syntax verified (if applicable)
- [ ] **Modern testing approach reviewed** (`docs/testing.md` and `tests/common/mod.rs`)
- [ ] Tests exist for similar features (reference them)

---

## 🚀 Step 7: Implementation

**Follow phase deliverables and exit criteria.**

**During implementation:**
- Run tests frequently: `cargo test`
- Check warnings: `cargo clippy`
- Monitor file sizes (if adding significant code)

**Writing tests (MODERN APPROACH):**
- Use `#[rstest]` for parameterized tests (1 line per case)
- Use `insta::assert_yaml_snapshot!()` for complex outputs
- Use helpers from `tests/common/mod.rs` (`assert_eval_number`, etc.)
- See `tests/lexer_tests_modern.rs` for examples
- Refer to `docs/testing.md` for full guide

---

## ✅ Step 8: Post-Phase Verification

**Before marking phase complete:**

```bash
# 1. Build check
cargo build

# 2. Test check
cargo test

# 3. Clippy check
cargo clippy

# 4. File size check (MANDATORY)
find crates/*/src -name "*.rs" -not -path "*/tests/*" -exec wc -l {} + | sort -rn | head -10
```

**Verify:**
- [ ] Exit criteria from phase file met
- [ ] All tests pass
- [ ] No compiler warnings
- [ ] No files exceed 1000 lines
- [ ] Files 800-1000 lines documented in handoff

---

## 📝 Step 9: Update STATUS.md

**Use handoff protocol:**

1. Mark phase complete (change ⬜ to ✅)
2. Update "Current Phase" section
3. Update "Implementation Files Needed"
4. Update "Last Updated" date
5. Add code organization notes if files 800-1000 lines

**Template:**
```markdown
**Last Completed:** phases/[section]/phase-XX-name.md
**Next Phase:** phases/[section]/phase-YY-name.md

**What to implement:** [Brief description]
```

---

## 🚫 Common Blocking Issues

### File Size Violation
**Symptom:** File exceeds 1000 lines
**Action:** Stop current work, refactor file, resume
**Reference:** `docs/CODE_ORGANIZATION.md`

### Missing Implementation Guide
**Symptom:** Phase references guide not in `docs/implementation/`
**Action:** Check `docs/implementation/README.md` for mapping
**Reference:** `STATUS.md` Phase-to-Implementation Mapping table

### Test Failures
**Symptom:** `cargo test` fails
**Action:** Fix failing tests before proceeding
**Do not:** Skip tests or mark phase complete

### Syntax Errors (Atlas Code)
**Symptom:** Atlas test code doesn't match spec
**Action:** Re-read `Atlas-SPEC.md` sections on syntax
**Common mistakes:** Missing semicolons, wrong function syntax, let/var confusion

---

## 📊 Quick Reference

| Command | Purpose |
|---------|---------|
| `cat STATUS.md` | Entry point - current state |
| `find crates/*/src -name "*.rs" -not -path "*/tests/*" -exec wc -l {} + \| sort -rn \| head -10` | File size check |
| `cargo build` | Compilation check |
| `cargo test` | Test suite |
| `cargo clippy` | Lint check |
| `cat Atlas-SPEC.md` | Language specification |
| `cat docs/CODE_ORGANIZATION.md` | File limits and refactoring |

---

**Remember:** This checklist is mandatory, not optional. Following it prevents:
- God files and technical debt
- Broken builds and test failures
- Syntax errors and spec violations
- Wasted work from incorrect assumptions

**Time investment:** 5-10 minutes per phase
**Time saved:** Hours of debugging and rework

---

**Last Updated:** 2026-02-12
