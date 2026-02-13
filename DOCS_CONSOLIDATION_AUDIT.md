# Atlas Documentation Consolidation Audit

**Date:** 2026-02-13
**Auditor:** AI Agent (Claude Sonnet 4.5)
**Purpose:** Identify outdated docs, broken references, and consolidation opportunities

---

## Executive Summary

**Total docs/**: 65 markdown files
**Critical Issues Found:** 4
**Files < 1KB (tiny):** 21 files
**Files with version references:** 29 files
**Files needing consolidation:** ~15 files
**Files to archive:** ~5 files (v0.1-specific context)

---

## 🚨 CRITICAL ISSUES (Must Fix Immediately)

### 1. BROKEN REFERENCE in docs/stdlib.md
**File:** `docs/stdlib.md` (line 3)
**Issue:** References non-existent `docs/stdlib-expansion-plan.md`
**Reality:** File is in `archive/stdlib-plans/stdlib-expansion-plan.md`
**Fix:** Remove the reference or update to correct path
**Priority:** CRITICAL

### 2. OUTDATED VERSION LANGUAGE
**Files affected:** 29 files contain "v0.1", "v1.0", "v1.1", "release", "ship"
**Issue:** Violates new DOCUMENTATION_PHILOSOPHY.md standards
**Examples:**
- `ir.md`: "A typed IR may be introduced in v1.1+"
- `modules.md`: "No package registry in v1.0"
- `value-model.md`: "Optional Option<T> or Result<T, E> types" (mentions future)

**Fix:** Reframe to "when ready" not "in vX.X"
**Priority:** HIGH

### 3. TINY FILE SPRAWL
**Issue:** 21 files < 20 lines, hard to navigate, increases cognitive load
**Impact:** AI agents have to read 21 separate files for semantic rules
**Fix:** Consolidate related files into respectable-sized reference docs
**Priority:** HIGH

### 4. v0.1 CONTEXT FILES WITHOUT ARCHIVE NOTICE
**Files:** Several files say "v0.1" but aren't in archive
**Issue:** Unclear if they're current or historical
**Fix:** Either update to current or archive with clear notice
**Priority:** MEDIUM

---

## 📊 File Size Distribution

### Tiny Files (< 1KB, < 20 lines) - 21 files
```
  301B (12 lines) diagnostic-ordering.md
  256B (12 lines) repl-state.md
  288B (12 lines) ir.md
  315B (13 lines) string-semantics.md
  324B (13 lines) numeric-edge-cases.md
  332B (13 lines) diagnostic-normalization.md
  345B (13 lines) e2e-parity.md
  386B (13 lines) json-dump-stability.md
  398B (19 lines) style.md
  407B (14 lines) warnings.md
  429B (14 lines) prelude.md
  448B (13 lines) array-aliasing.md
  458B (14 lines) top-level-execution.md
  473B (19 lines) ast-dump.md
  519B (18 lines) keyword-policy.md
  546B (19 lines) typecheck-dump.md
  577B (18 lines) debug-info.md
  649B (16 lines) language-comparison.md
  662B (18 lines) phase-gates.md
  677B (28 lines) modules.md
  681B (19 lines) operator-rules.md
```

### Small Files (1-3KB, 20-100 lines) - 14 files
```
  743B (23 lines) runtime-api-evolution.md
  820B (26 lines) stdlib.md
 1.0K (29 lines) runtime.md
 1.0K (35 lines) repl.md
 1.0K (37 lines) value-model.md
 1.0K (41 lines) bytecode-format.md
 1.3K (55 lines) runtime-api.md
 1.7K (22 lines) coverage-matrix.md
 1.8K (50 lines) engineering.md
 1.9K (72 lines) diagnostics.md
 2.4K (76 lines) versioning.md
 2.6K (88 lines) ast.md
 3.6K (65 lines) decision-log.md
```

### Respectable Size (4-8KB) - 13 files
```
 4.1K (160 lines) repl-modes.md
 4.9K (229 lines) cli-config.md
 5.0K (203 lines) AI_AGENT_CHECKLIST.md
 5.3K implementation/01-project-structure.md
 5.4K implementation/14-repl.md
 5.6K (204 lines) parser-recovery-policy.md
 6.0K implementation/11-bytecode.md
 6.2K (281 lines) ai-principles.md
 6.9K (270 lines) CODE_ORGANIZATION.md
 7.7K implementation/06-symbol-table.md
 8.1K (386 lines) testing.md
 8.4K (397 lines) why-strict.md
```

### Good Size (8KB+) - 17 files
```
All implementation/ guides (4-16KB) - EXCELLENT
 9.3K implementation/12-vm.md
 9.5K implementation/03-lexer.md
 9.9K (391 lines) AI-MANIFESTO.md
10K implementation/04-parser.md
12K (370 lines) GRAMMAR_CONFORMANCE.md
12K implementation/07-typechecker.md
13K (505 lines) DOCUMENTATION_PHILOSOPHY.md
13K implementation/10-interpreter.md
13K implementation/16-lsp.md
14K (490 lines) ai-workflow.md
22K (736 lines) io-security-model.md
```

---

## 🗂️ CONSOLIDATION PROPOSAL

### Group 1: Language Semantics Reference
**New file:** `docs/LANGUAGE_SEMANTICS.md` (~4-5KB)
**Consolidate:**
- `string-semantics.md` (315B) - UTF-8 rules, len() behavior
- `array-aliasing.md` (448B) - Reference counting, mutation visibility
- `numeric-edge-cases.md` (324B) - NaN/Infinity handling
- `operator-rules.md` (681B) - Type rules for operators
- `top-level-execution.md` (458B) - Execution order

**Rationale:** All define core language semantics. Single reference doc is easier to navigate.
**AI Agent Impact:** ONE file to read instead of FIVE for semantic rules.

### Group 2: Diagnostic System Reference
**New file:** `docs/DIAGNOSTIC_SYSTEM.md` (~3-4KB)
**Consolidate:**
- `diagnostics.md` (1.9K) - Already has diagnostic overview
- `diagnostic-normalization.md` (332B) - Path normalization rules
- `diagnostic-ordering.md` (301B) - Ordering rules
- `warnings.md` (407B) - Warning codes and policy

**Rationale:** All about diagnostic system. Merge into single comprehensive guide.
**AI Agent Impact:** ONE file instead of FOUR for diagnostic rules.

### Group 3: JSON Dump Specifications
**New file:** `docs/JSON_DUMPS.md` (~2-3KB)
**Consolidate:**
- `ast-dump.md` (473B) - AST JSON format
- `typecheck-dump.md` (546B) - Typecheck JSON format
- `json-dump-stability.md` (386B) - Stability rules
- `debug-info.md` (577B) - Debug info format

**Rationale:** All about JSON dump formats. Grouped logically.
**AI Agent Impact:** ONE file instead of FOUR for JSON formats.

### Group 4: Runtime Specification
**New file:** `docs/RUNTIME.md` (~4-5KB)
**Consolidate:**
- `runtime.md` (1.0K) - Runtime model overview
- `value-model.md` (1.0K) - Value enum details
- `bytecode-format.md` (1.0K) - Bytecode format
- `prelude.md` (429B) - Built-in functions
- `stdlib.md` (820B) - Standard library (fix broken reference!)

**Rationale:** All describe runtime behavior. Single comprehensive runtime guide.
**AI Agent Impact:** ONE file instead of FIVE for runtime details.

### Group 5: Code Quality Standards
**New file:** `docs/CODE_QUALITY.md` (~2-3KB)
**Consolidate:**
- `style.md` (398B) - Code style rules
- `phase-gates.md` (662B) - Phase gate requirements
- `coverage-matrix.md` (1.7K) - Spec-to-phase mapping
- `engineering.md` (1.8K) - Engineering standards (or merge parts)

**Rationale:** All about code quality and development standards.
**AI Agent Impact:** Clearer quality standards in one place.

### Group 6: Testing Standards
**Merge into existing:** `docs/testing.md` (8.1K - already good size!)
**Add sections from:**
- `e2e-parity.md` (345B) - Interpreter/VM parity testing

**Rationale:** testing.md already exists and is good size. Add parity section.
**AI Agent Impact:** Complete testing guide in one file.

---

## 📁 FILES TO ARCHIVE (v0.1 Historical Context)

### Archive to: `archive/docs/v0.1/`

**Files with strong v0.1 context that may be historical:**

1. **ir.md** (288B)
   - Says "v0.1 Decision" and "No separate typed IR in v0.1"
   - Says "A typed IR may be introduced in v1.1+"
   - This is historical context, not current guidance

2. **modules.md** (677B)
   - Says "Not implemented in v0.1"
   - Says "No package registry in v1.0"
   - Should be rewritten as research doc or archived

3. **language-comparison.md** (649B)
   - Says "Draft Entries"
   - Barely started, not useful to AI agents
   - Archive or expand significantly

4. **runtime-api-evolution.md** (743B)
   - Very brief, says "Plan for v0.1"
   - May be historical

5. **keyword-policy.md** (519B)
   - May be v0.1-specific
   - Check if still relevant

**Recommendation:** Archive these with clear "v0.1 historical context" notice.

---

## 🔍 FILES TO UPDATE (Fix Version Language)

**Remove version references per DOCUMENTATION_PHILOSOPHY.md:**

### High Priority:
- `modules.md` - Remove "v1.0" language
- `ir.md` - Remove "v1.1+" language
- `value-model.md` - Remove "Future Extensions" with vague promises
- `runtime-api-evolution.md` - Reframe or archive

### Medium Priority:
- All implementation/ guides mentioning "v0.1" as current context
  → Change to "current implementation" or remove version reference

### Low Priority:
- Files with "v0.1" in historical context (archive material) - OK to keep

---

## ✅ FILES TO KEEP AS-IS

### Excellent Size & Quality (No Changes Needed):

**Top-level guides (8KB+):**
- ✅ `DOCUMENTATION_PHILOSOPHY.md` (13K) - Perfect!
- ✅ `AI-MANIFESTO.md` (9.9K) - Just fixed
- ✅ `ai-workflow.md` (14K) - Good size
- ✅ `why-strict.md` (8.4K) - Good size
- ✅ `ai-principles.md` (6.2K) - Good size
- ✅ `AI_AGENT_CHECKLIST.md` (5.0K) - Good size
- ✅ `CODE_ORGANIZATION.md` (6.9K) - Good size
- ✅ `GRAMMAR_CONFORMANCE.md` (12K) - Good size
- ✅ `io-security-model.md` (22K) - Comprehensive
- ✅ `testing.md` (8.1K) - Good (just add parity section)
- ✅ `parser-recovery-policy.md` (5.6K) - Good size
- ✅ `repl-modes.md` (4.1K) - Good size
- ✅ `cli-config.md` (4.9K) - Good size

**Implementation guides (ALL EXCELLENT 4-16KB):**
- ✅ `implementation/01-project-structure.md` (5.3K)
- ✅ `implementation/02-core-types.md` (3.3K)
- ✅ `implementation/03-lexer.md` (9.5K)
- ✅ `implementation/04-parser.md` (10K)
- ✅ `implementation/05-ast.md` (4.1K)
- ✅ `implementation/06-symbol-table.md` (7.7K)
- ✅ `implementation/07-typechecker.md` (12K)
- ✅ `implementation/08-diagnostics.md` (4.0K)
- ✅ `implementation/09-value-model.md` (3.2K)
- ✅ `implementation/10-interpreter.md` (13K)
- ✅ `implementation/11-bytecode.md` (6.0K)
- ✅ `implementation/12-vm.md` (9.3K)
- ✅ `implementation/13-stdlib.md` (4.2K)
- ✅ `implementation/14-repl.md` (5.4K)
- ✅ `implementation/15-testing.md` (5.0K)
- ✅ `implementation/16-lsp.md` (13K)
- ✅ `implementation/README.md` (2.9K)

**These are PERFECT - do not touch!**

---

## 📋 RECOMMENDED ACTION PLAN

### Phase 1: Critical Fixes (Do First)
1. ✅ Fix broken reference in `stdlib.md`
2. ✅ Archive v0.1 historical context files
3. ✅ Update version language in remaining docs

### Phase 2: Consolidation (Do After User Approval)
1. ✅ Create `LANGUAGE_SEMANTICS.md` (consolidate 5 files)
2. ✅ Create `DIAGNOSTIC_SYSTEM.md` (consolidate 4 files)
3. ✅ Create `JSON_DUMPS.md` (consolidate 4 files)
4. ✅ Create `RUNTIME.md` (consolidate 5 files)
5. ✅ Create `CODE_QUALITY.md` (consolidate 4 files)
6. ✅ Update `testing.md` (add parity section)

### Phase 3: Cleanup
1. ✅ Delete consolidated source files
2. ✅ Update references in other docs
3. ✅ Update README.md doc index if needed

---

## 📊 BEFORE/AFTER METRICS

### Before Consolidation:
- **Total files:** 65
- **Files < 1KB:** 21 (32%)
- **Files to read for semantics:** 5+
- **Files to read for diagnostics:** 4+
- **Files to read for runtime:** 5+
- **Cognitive load:** HIGH

### After Consolidation:
- **Total files:** ~50 (-15 files)
- **Files < 1KB:** ~5 (10%)
- **Files to read for semantics:** 1
- **Files to read for diagnostics:** 1
- **Files to read for runtime:** 1
- **Cognitive load:** MEDIUM

### Benefit to AI Agents:
- **Fewer files to discover:** Less glob/grep operations
- **Faster reference lookup:** Single file per topic
- **Better context:** Related info together
- **Clearer structure:** Logical grouping

---

## ⚠️ RISKS & MITIGATIONS

### Risk 1: Losing Granular Version History
**Mitigation:** Git history preserves everything. Consolidated files will have clear sections showing what was merged.

### Risk 2: Breaking References in Phase Files
**Mitigation:** Grep for all references before deleting. Update references systematically.

### Risk 3: Making Files Too Large
**Mitigation:** Target 3-8KB per consolidated file. This is respectable, not "god file" territory.

### Risk 4: Consolidating Files That Should Stay Separate
**Mitigation:** Get user approval before consolidating. User knows their project best.

---

## 🎯 DECISION POINTS FOR USER

**I NEED YOUR APPROVAL ON:**

1. **Consolidation Groups:** Do the proposed groupings make sense?
2. **Archive List:** Do you agree these are v0.1 historical context?
3. **Files to Keep:** Do you agree implementation/ guides are good as-is?
4. **Target Sizes:** Is 3-8KB per file the right target?
5. **Critical Fix:** Should I fix `stdlib.md` broken reference immediately?

**DO NOT PROCEED without your explicit approval on each group.**

---

## 💬 RECOMMENDATIONS

**My honest assessment:**

1. **implementation/ guides are EXCELLENT** - 4-16KB each, comprehensive, well-structured. DO NOT TOUCH.

2. **Top-level guides are mostly GOOD** - Recent fixes (AI-MANIFESTO, DOCUMENTATION_PHILOSOPHY) are great. Keep as-is.

3. **Tiny semantic files SHOULD be consolidated** - They're reference cards scattered everywhere. Group them logically.

4. **v0.1 context files SHOULD be archived** - They're historical, not current guidance.

5. **Broken reference MUST be fixed** - stdlib.md references non-existent file.

6. **Version language MUST be updated** - Violates new documentation philosophy.

**This consolidation will make Atlas documentation:**
- ✅ Easier for AI agents to navigate
- ✅ Faster to reference
- ✅ More logical structure
- ✅ Less cognitive overhead
- ✅ Still comprehensive and detailed

**It will NOT:**
- ❌ Create god files (target is 3-8KB)
- ❌ Lose information (everything preserved, just organized)
- ❌ Break working references (we'll update them)
- ❌ Rush or compromise quality

---

**Ready for your decision on how to proceed.** 🎯
