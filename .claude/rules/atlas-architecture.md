---
paths:
  - "**"
---

# Atlas Architecture Standards

Always-on. Applies to every file in the codebase.

## File Size Limits

### Source files
| Threshold | Action |
|-----------|--------|
| < 1,500 lines | Normal — no action needed |
| 1,500–2,000 lines | Warning zone — split before adding more logic |
| > 2,000 lines | **VIOLATION** — do not add to this file; split first |

**Exceptions** (genuinely monolithic by nature):
- VM execute/dispatch loop (`vm/mod.rs` execute_loop)
- Main parser switch
- Any file that physically cannot be split without breaking the hot path

An exception file MUST have this at the top:
```rust
// ARCH-EXCEPTION: <one-sentence reason> — target split: <milestone or "post-Block-N">
```

Without this comment, a file over 2,000 lines is a violation regardless of circumstance.

### Test files
| Threshold | Action |
|-----------|--------|
| < 3,000 lines | Normal |
| 3,000–4,000 lines | Warning — plan subdirectory migration |
| > 4,000 lines | **VIOLATION** — migrate to subdirectory before adding tests |

**Migration pattern:** `tests/stdlib.rs` → `tests/stdlib/mod.rs` + `tests/stdlib/{category}.rs`

The "NO new test files" rule (in `crates/atlas-runtime/src/CLAUDE.md`) means no new top-level
domain files without justification. It does NOT prevent subdirectory splits when a file hits the
threshold. Threshold-triggered subdirectory splits are **required**, not optional.

## No Inline Tests in Source Files

`#[cfg(test)]` blocks do not belong in `src/` files. All tests live in `crates/atlas-runtime/tests/`.

**Why:** Inline tests inflate source file line counts, mix execution logic with test logic, and
force AI agents to load test code every time they work on the implementation. On a project this
size, that's a real context and token cost.

**Enforcement:** If you are adding tests, they go in the appropriate `tests/*.rs` file.
If you find inline tests while working in a source file, note them in your handoff summary
as a cleanup item — do not refactor mid-phase, but do not add more.

## `mod.rs` Files

A `mod.rs` file is a module entry point, not a dumping ground.

- Re-exports, module declarations, and a small amount of glue code: fine
- Business logic in `mod.rs`: only if it genuinely belongs there and the file is < 800 lines
- If `mod.rs` contains the majority of a module's logic AND exceeds 800 lines: split by concern
  into named sibling files (`intrinsics.rs`, `dispatch.rs`, `call.rs`, etc.)

## Subagent Usage

The universal agent ban is lifted with guardrails:

**Allowed (haiku model):**
- `Explore` agent — codebase exploration, multi-location searches, answering structural questions
- `Plan` agent — architecture planning before complex changes

**Not allowed:**
- Task agents for code execution, test running, or file writing
- Spinning up agents for tasks that Glob + Grep + Read can handle in 2–3 calls
- Agents that would write or modify files (all writes stay in the main context)

**Rule of thumb:** If you'd need more than 3 rounds of Glob/Grep to find what you're looking for,
use an Explore agent. If you can find it in 1–2 searches, do it directly.

## Gate Hook — GATE 0

During GATE 0 (pre-work verification), check:
```bash
# Files approaching or over threshold
find crates/ -name "*.rs" -not -path "*/target/*" | xargs wc -l | sort -rn | awk '$1 > 1500 {print}' | head -20
```

If any non-exception source file is over 2,000 lines, or any test file is over 4,000 lines:
**flag it in your phase summary** even if it's pre-existing. Do not add to violating files.
If your phase requires adding to a violating file, split it first as a prerequisite step.
