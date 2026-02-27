# Phase 08: Warning Zone Sweep + Final Verification

**Block:** test-split (Maintenance)
**Track:** 1 — direct push to main with [skip ci] after local verification
**Depends on:** Phase 07 complete

## Goal

Address all remaining 20–40KB warning-zone files not handled in earlier phases. Produce a
clean final state where the entire test suite satisfies the 40KB hard cap and ideally
approaches the 20KB soft target across the board. Final AC check for the entire block.

## Remaining Warning-Zone Files (post Phase 07)

Re-run the check at the start of this phase — phases 01–07 will have resolved some:

```bash
find crates/atlas-runtime/tests -name "*.rs" -not -path "*/target/*" -size +20k | xargs du -sh | sort -rh
```

Expected remaining at phase start (adjust based on actual state):

| File | Size |
|------|------|
| `stdlib_hardening.rs` | 28KB |
| `http.rs` | 28KB |
| `bytecode/optimizer.rs` | 28KB |
| `stdlib/parity.rs` | 24KB |
| `stdlib/io.rs` | 26KB |
| `stdlib/collections.rs` | 28KB |
| `stdlib/types.rs` | 32KB |
| `repl.rs` | 24KB |
| `system/filesystem.rs` | 24KB |

## Approach

For each file:
1. Check actual current size (earlier phases may have reduced it)
2. If still > 20KB: read the file, identify natural split points, split
3. If between 15–20KB: note it, leave as-is (soft target, not a violation)
4. If < 15KB: already fine, skip

**Priority:** Files > 25KB get split. Files 20–25KB are judgment calls — split only if there
are clear domain boundaries. Do not create artificial splits just to hit a number.

## Suggested Splits

**`stdlib/types.rs` (32KB):**
`stdlib/types/conversion.rs`, `stdlib/types/predicates.rs`, `stdlib/types/coercion.rs`

**`stdlib/collections.rs` (28KB):**
`stdlib/collections/array.rs`, `stdlib/collections/map.rs`, `stdlib/collections/set.rs`

**`stdlib/io.rs` (26KB):**
`stdlib/io/file.rs`, `stdlib/io/console.rs`, `stdlib/io/path.rs`

**`stdlib_hardening.rs` (28KB):**
`stdlib_hardening/` → `security.rs`, `edge_cases.rs`, `overflow.rs`

**`http.rs` (28KB):**
`http/` → `requests.rs`, `responses.rs`, `integration.rs`

**`bytecode/optimizer.rs` (28KB):**
`bytecode/optimizer/` → `constant_folding.rs`, `dead_code.rs`, `peephole.rs`

**`repl.rs`, `system/filesystem.rs`, `stdlib/parity.rs`:** Read first — may already be
under 25KB after upstream splits. Split only if over 20KB with clear boundaries.

## Final Block AC Check

After all splits:

```bash
# Zero hard violations
find crates/atlas-runtime/tests -name "*.rs" -not -path "*/target/*" -size +40k | xargs du -sh 2>/dev/null
# Expected: no output

# Warning zone inventory (document, don't necessarily fix)
find crates/atlas-runtime/tests -name "*.rs" -not -path "*/target/*" -size +20k | xargs du -sh 2>/dev/null | sort -rh

# Full suite green
cargo nextest run -p atlas-runtime
```

## CLAUDE.md Final Audit

Run `atlas-doc-auditor` after this phase. Every test table entry in CLAUDE.md and
atlas-testing.md must reflect the final split structure.

## Acceptance Criteria

- [ ] Zero files > 40KB in `crates/atlas-runtime/tests/`
- [ ] All warning-zone files (20–40KB) either split or documented as intentionally deferred
- [ ] `cargo nextest run -p atlas-runtime` — full suite green, test count matches pre-block baseline
- [ ] `cargo nextest run -p atlas-lsp` — no regressions (no LSP files changed, sanity check)
- [ ] CLAUDE.md test tables accurate to actual file layout
- [ ] atlas-testing.md domain table updated
- [ ] `atlas-doc-auditor` passes
