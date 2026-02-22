---
paths:
  - "phases/**/*.md"
---

# Atlas Phase File Rules

## Structure (enforced)

Every phase file must have:
```
# Phase XX: {Title}

**Block:** N ({Block Name})
**Depends on:** {Block N-1 complete | Phase XX}
**Complexity:** low/medium/high
**Files to modify:** [verified list]

## Summary
## Current State   ← verified against codebase, not assumed
## Requirements
## Acceptance Criteria
## Tests Required
## Notes (optional)
```

## Hard Rules

- **~100 lines max.** One thing done completely. If it's growing past 100, split it.
- **Current State must be verified.** Grep the actual codebase before writing this section.
  Never write "assumed" state. If you haven't checked, check first.
- **Every AC must be testable.** Vague ACs ("works correctly") are banned.
- **Block dependency must be explicit.** Every phase declares its block and what it depends on.
- **No cross-block dependencies within a phase.** If a phase needs something from a later
  block, it belongs in that later block.

## Block System

Phases live in `phases/v0.3/block-XX-{name}/`. Blocks are strictly sequential.
Block N cannot begin until Block N-1's ALL acceptance criteria are met.
Phases within a block are independent (different files/aspects, no ordering required).

## Final Phase of Every Block

The last phase of every block MUST include:
1. Spec update (`docs/specification/`)
2. `STATUS.md` block row: ⬜ → ✅
3. Auto-memory update (decisions/{domain}.md)
4. Crate CLAUDE.md audit — update `crates/*/src/CLAUDE.md` for structural changes
