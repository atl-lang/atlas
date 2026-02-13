# Phase File Documentation Reference Fix Map

**Date:** 2026-02-13
**Purpose:** Systematic mapping of broken doc references to correct consolidated docs
**Status:** AUDIT COMPLETE - Ready for implementation

---

## Executive Summary

**Total broken references:** 72 unique docs (155+ total references)
**Action required:** Create 10 new docs + Update 155+ references in phase files

---

## Critical Mappings (High-Volume Broken References)

### 1. docs/diagnostics.md → docs/DIAGNOSTIC_SYSTEM.md (27 refs)
**Status:** EXISTS - consolidated file
**Action:** Find/replace in phase files
**Used for:** Diagnostic system rules, error codes, warning codes

### 2. docs/stdlib.md → docs/STDLIB_API.md (23 refs)
**Status:** MUST CREATE - this is API reference that phases will update
**Action:** Create new file + update phase references
**Used for:** Standard library API documentation (functions, signatures, examples)
**Note:** docs/implementation/13-stdlib.md is for implementation, not API reference

### 3. docs/runtime.md → docs/RUNTIME.md (16 refs)
**Status:** EXISTS but wrong case - should use uppercase
**Action:** Find/replace lowercase → uppercase
**Used for:** Runtime specification (value model, bytecode, execution model)

### 4. docs/engineering.md → docs/CODE_QUALITY.md (13 refs)
**Status:** EXISTS - consolidated file
**Action:** Find/replace in phase files
**Used for:** Code quality standards, engineering practices

### 5. docs/style.md → docs/CODE_QUALITY.md (6 refs)
**Status:** EXISTS - consolidated file
**Action:** Find/replace in phase files
**Used for:** Code style guidelines

---

## Consolidated File Mappings (From Consolidation)

These were deleted during docs consolidation and merged into comprehensive files:

| Broken Reference | Correct Reference | Notes |
|------------------|-------------------|-------|
| docs/array-aliasing.md | docs/LANGUAGE_SEMANTICS.md | Section: Array Aliasing |
| docs/ast-dump.md | docs/JSON_DUMPS.md | Section: AST Dumps |
| docs/bytecode-format.md | docs/RUNTIME.md | Section: Bytecode Format |
| docs/debug-info.md | docs/JSON_DUMPS.md | Section: Debug Info |
| docs/diagnostic-normalization.md | docs/DIAGNOSTIC_SYSTEM.md | Section: Normalization |
| docs/diagnostic-ordering.md | docs/DIAGNOSTIC_SYSTEM.md | Section: Ordering |
| docs/e2e-parity.md | docs/testing.md | Section: Interpreter/VM Parity |
| docs/json-dump-stability.md | docs/JSON_DUMPS.md | Section: Stability |
| docs/numeric-edge-cases.md | docs/LANGUAGE_SEMANTICS.md | Section: Numeric Edge Cases |
| docs/operator-rules.md | docs/LANGUAGE_SEMANTICS.md | Section: Operator Rules |
| docs/phase-gates.md | docs/CODE_QUALITY.md | Section: Phase Gates |
| docs/prelude.md | docs/RUNTIME.md | Section: Prelude |
| docs/string-semantics.md | docs/LANGUAGE_SEMANTICS.md | Section: String Semantics |
| docs/top-level-execution.md | docs/LANGUAGE_SEMANTICS.md | Section: Execution Order |
| docs/typecheck-dump.md | docs/JSON_DUMPS.md | Section: Typecheck Dumps |
| docs/value-model.md | docs/RUNTIME.md | Section: Value Model |
| docs/warnings.md | docs/DIAGNOSTIC_SYSTEM.md | Section: Warnings |

---

## Feature-Specific Docs (Need Creation - Phase Will Populate)

These docs don't exist because the features haven't been implemented yet. Phases expect to CREATE or UPDATE these as they implement features:

### Foundation Category
| Reference | Action | Purpose |
|-----------|--------|---------|
| docs/modules.md | CREATE | Module system design and usage |
| docs/embedding-guide.md | CREATE | How to embed Atlas runtime |
| docs/configuration.md | CREATE | Configuration system |
| docs/build-system.md | CREATE | Build system design |
| docs/reflection.md | CREATE | Reflection API |
| docs/benchmarking.md | CREATE | Benchmarking framework |
| docs/security-model.md | CREATE | Security and permissions |

### Stdlib Category
| Reference | Action | Purpose |
|-----------|--------|---------|
| docs/stdlib-usage-guide.md | CREATE | How to use stdlib (examples, patterns) |
| docs/http-guide.md | CREATE | HTTP client usage guide |
| docs/async-io.md | CREATE | Async I/O design |
| docs/process-management.md | CREATE | Process spawning guide |
| docs/testing-framework.md | CREATE | Built-in testing framework |

### Typing Category
| Reference | Action | Purpose |
|-----------|--------|---------|
| docs/type-aliases.md | CREATE | Type alias design |
| docs/union-types.md | CREATE | Union/intersection types |
| docs/generic-constraints.md | CREATE | Generic constraint design |
| docs/type-guards.md | CREATE | Type guard design |
| docs/type-inference.md | CREATE | Advanced inference design |

### VM/Bytecode Category
| Reference | Action | Purpose |
|-----------|--------|---------|
| docs/vm-architecture.md | CREATE | VM architecture overview |
| docs/vm-optimizer-guide.md | CREATE | Bytecode optimizer guide |
| docs/vm-profiler-guide.md | CREATE | VM profiler guide |
| docs/vm-debugger-guide.md | CREATE | VM debugger guide |
| docs/jit.md | CREATE | JIT compilation design |

### Frontend Category
| Reference | Action | Purpose |
|-----------|--------|---------|
| docs/formatter-guide.md | CREATE | Code formatter design |
| docs/source-maps.md | CREATE | Source map format |
| docs/incremental-compilation.md | CREATE | Incremental compilation design |

### CLI Category
| Reference | Action | Purpose |
|-----------|--------|---------|
| docs/cli-guide.md | CREATE | CLI usage guide |
| docs/cli-reference.md | CREATE | CLI command reference |
| docs/package-manager-cli.md | CREATE | Package manager CLI |
| docs/package-manifest.md | CREATE | Package manifest format |
| docs/dependency-resolution.md | CREATE | Dependency resolution algorithm |
| docs/project-templates.md | CREATE | Project scaffolding templates |

### LSP Category
| Reference | Action | Purpose |
|-----------|--------|---------|
| docs/lsp-features.md | CREATE | LSP features overview |
| docs/lsp-navigation.md | CREATE | LSP navigation features |
| docs/lsp-refactoring.md | CREATE | LSP refactoring actions |

---

## Archived/Obsolete References

These were archived to archive/docs/v0.1/ as historical context:

| Reference | Action | Notes |
|-----------|--------|-------|
| docs/ir.md | REMOVE or point to archive | v0.1 historical decision |
| docs/keyword-policy.md | REMOVE or point to archive | v0.1 historical |
| docs/language-comparison.md | REMOVE or point to archive | v0.1 draft |
| docs/runtime-api-evolution.md | REMOVE or point to archive | v0.1 historical |

---

## Status/Tracking Docs (Low Priority)

These are status tracking docs that phases mention. Decision needed:

| Reference | Decision | Notes |
|-----------|----------|-------|
| docs/foundation-status.md | OPTIONAL | Could track foundation progress |
| docs/frontend-status.md | OPTIONAL | Could track frontend progress |
| docs/cli-status.md | OPTIONAL | Could track CLI progress |
| docs/lsp-status.md | OPTIONAL | Could track LSP progress |
| docs/interpreter-status.md | OPTIONAL | Could track interpreter progress |
| docs/typing-status.md | OPTIONAL | Could track typing progress |
| docs/warnings-test-plan.md | OPTIONAL | Test plan doc |

**Recommendation:** Don't create these - STATUS.md serves this purpose globally

---

## Implementation Strategy

### Phase 1: Create Critical Docs (Do First)
1. **CREATE docs/STDLIB_API.md** (CRITICAL - 23 refs depend on this)
2. Create all feature-specific docs as stubs with clear structure
3. Each stub should have sections ready for phases to populate

### Phase 2: Bulk Find/Replace Operations
Use sed to update all phase files:
```bash
# Consolidated file mappings
find phases -name "*.md" -exec sed -i '' 's/docs\/diagnostics\.md/docs\/DIAGNOSTIC_SYSTEM.md/g' {} \;
find phases -name "*.md" -exec sed -i '' 's/docs\/stdlib\.md/docs\/STDLIB_API.md/g' {} \;
find phases -name "*.md" -exec sed -i '' 's/docs\/runtime\.md/docs\/RUNTIME.md/g' {} \;
find phases -name "*.md" -exec sed -i '' 's/docs\/engineering\.md/docs\/CODE_QUALITY.md/g' {} \;
find phases -name "*.md" -exec sed -i '' 's/docs\/style\.md/docs\/CODE_QUALITY.md/g' {} \;
# ... continue for all consolidated mappings
```

### Phase 3: Remove Obsolete References
For archived docs, remove references or add note that they're historical

### Phase 4: Verification
1. Run audit script again - should show 0 broken critical references
2. Test flow: STATUS.md → phase → docs (all exist)
3. Verify stub docs have proper structure

---

## Success Criteria

- ✅ All 72 broken references resolved
- ✅ docs/STDLIB_API.md created and structured
- ✅ All feature-specific doc stubs created
- ✅ Phase files updated with correct references
- ✅ Audit script shows 0 critical broken references
- ✅ AI agent can follow: STATUS.md → phase → docs without errors

---

**Next Step:** Create docs/STDLIB_API.md and feature doc stubs
