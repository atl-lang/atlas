# Test File Split Tracking

**Purpose:** Track progress on splitting large test files down to 12KB target size.
**Status:** PAUSED — resume next week or when convenient
**Last Updated:** 2026-02-27
**Next Reminder:** 2026-03-06 (1 week)

---

## ⚠️ WEEKLY REMINDER PROTOCOL

**AI agents MUST check this file at the start of EVERY session.**

If `Next Reminder` date has passed:
1. **Alert the user:** "Reminder: Test-split work is paused at 7/20 files. Resume this week?"
2. **Update this timestamp** to `Next Reminder: [current date + 7 days]`
3. **Wait for user decision** — do NOT auto-resume

This protocol ensures test-split work isn't forgotten but doesn't block critical feature work.

---

## Progress Summary

**Total large files identified:** ~20 (>12KB threshold)
**Files split so far:** 7 ✅
**Files remaining:** ~13 🔲
**Branch:** Was `block/test-split` — merged to main, will create new branch on resume

---

## ✅ Completed Splits (7 files)

| File | Original Size | Split Into | Status | Commit |
|------|---------------|------------|--------|--------|
| `frontend_syntax/*` | Various | 9 subdomain files | ✅ Merged | 4e1956d |
| `typesystem/inference.rs` | 60KB | 9 files in `inference/` | ✅ Merged | 58fc263 |
| `typesystem/integration.rs` | 52KB | 7 files in `integration/` | ✅ Merged | bf774f3 |
| `typesystem/generics.rs` | 48KB | 3 files in `generics/` | ✅ Merged | 2e74137 |
| `typesystem/flow.rs` | 36KB | 4 files in `flow/` | ✅ Merged | 89dec38 |
| `typesystem/constraints.rs` | 32KB | 4 files in `constraints/` | ✅ Merged | 607a430 |
| `typesystem/bindings.rs` | 28KB | 3 files in `bindings/` | ✅ Merged | 5157a84 |
| `api.rs` | 88KB | 11 files in `api/` | ✅ Merged | dde9d84 |
| `stdlib/real_world.rs` | 84KB | 9 files in `real_world/` | ✅ Merged | f16bb00 |
| `system/compression.rs` | 80KB | 9 files in `compression/` | ✅ Merged | faebff3 |
| `stdlib/vm_stdlib.rs` | 80KB | 7 files in `vm_stdlib/` | ✅ Merged | 036734f |
| `debugger.rs` | 80KB | 6 files in `debugger/` | ✅ Merged | 798cf04 |
| `interpreter/integration.rs` | 76KB | 15 files in `integration/` | ✅ Merged | 6d0d983 |

**Note:** Count is 13 not 7 because some phases split multiple files.

---

## 🔲 Remaining Large Files (Priority Order)

**Severity: CRITICAL (>60KB)**
1. `diagnostics.rs` — 68KB → split into `diagnostics/` (5 files: type_errors, parse_errors, runtime_errors, warnings, spans)
2. `async_runtime.rs` — 68KB → split into `async_runtime/` (4 files: futures, channels, tasks, integration)
3. `ffi.rs` — 64KB → split into `ffi/` (4 files: bindings, types, safety, integration)
4. `collections.rs` — 64KB → split into `collections/` (5 files: hash_map, set, queue, cow, integration)

**Severity: HIGH (40-60KB)**
5. `modules.rs` — 56KB → split into `modules/` (4 files: import, export, resolution, integration)
6. `datetime_regex.rs` — 52KB → split into `datetime_regex/` (2 files: datetime, regex)
7. `security.rs` — 48KB → split into `security/` (4 files: permissions, sandbox, fs_access, integration)
8. `closures.rs` — 48KB → split into `closures/` (5 files: capture, anon_fn, hof, ownership, integration)

**Severity: MEDIUM (30-40KB)**
9. `pattern_matching.rs` — 40KB → split into `pattern_matching/` (4 files: literals, destructure, guards, integration)
10. `regression.rs` — 40KB → split into `regression/` (4 files: parser, typechecker, runtime, stdlib)
11. `stdlib/integration.rs` — 52KB → split if not already done
12. `stdlib/functions.rs` — 40KB → split if needed
13. `vm/for_in.rs` — 36KB → split if needed

---

## Resume Instructions (For Next Week's Agent)

**When resuming test-split work:**

1. **Create fresh branch:** `git checkout main && git checkout -b block/test-split-phase2`
2. **Start with highest priority:** `diagnostics.rs` (68KB, most critical)
3. **Work sequentially:** One file at a time, verify with `cargo nextest run -p atlas-runtime --test <domain>`
4. **Track 1 process:** Each split gets committed and pushed to main with `[skip ci]`
5. **Update this file:** Move completed files from Remaining to Completed section
6. **Update STATUS.md:** Keep "Last Updated" current

**Phase files exist:** `phases/v0.3/block-ts-test-split/phase-04-top-level-monoliths.md` through `phase-08-warning-zone-sweep.md` — these have the original plan and split strategies.

---

## New Test Guidelines (Active NOW)

**For all AI agents adding tests while test-split is paused:**

- ✅ **Use the split subdirectories that exist** (frontend_syntax/, typesystem/*, api/*, etc.)
- ✅ **Check file size before adding:** `du -k <file>` — if it will exceed 12KB, split it first
- ✅ **Follow the domain routing table** in `.claude/rules/atlas-testing.md`
- ❌ **Do NOT add to monolith files** that are marked for splitting above
- ❌ **Do NOT create new top-level test files** without architectural approval

**The 12KB limit is now enforced** — `.claude/rules/atlas-testing.md` auto-loads and will guide you.

---

## Why This Matters

**AI token cost:** A 68KB test file burns ~17,000 tokens just to read. For an AI-maintained project, this is a real cost. Target is 10-12KB per file (~2,500-3,000 tokens).

**Agent context limits:** When working on test files, agents need room for the implementation code too. Huge test files crowd out the implementation context.

**Maintenance velocity:** Smaller, domain-focused files are faster to scan, understand, and modify.

---

## Metrics

| Metric | Before Split | After 13 Splits | Target (All 20) |
|--------|--------------|-----------------|-----------------|
| Largest test file | 88KB (api.rs) | 68KB (diagnostics.rs) | <12KB all files |
| Files >40KB | 20+ | ~13 | 0 |
| Avg tokens per read | ~15,000 | ~10,000 (reduced) | ~3,000 |

**Current best:** All split files are now <16KB. Next batch targets <12KB across the board.
