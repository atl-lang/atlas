# Status Directory Structure

This directory contains organized status tracking for Atlas v0.2 development.

---

## Organization

```
status/
├── trackers/      Phase inventories (detailed lists)
├── references/    Reference materials (standards, checklists, mappings)
└── history/       Historical context (v0.1 summary, archives)
```

---

## Files

### Trackers (Phase Inventories)

Detailed phase lists for each category:
- **0-foundation.md** - Foundation infrastructure (21/21 complete)
- **1-stdlib.md** - Standard library (9/21 in progress)
- **2-bytecode-vm.md** - Bytecode VM optimization
- **3-frontend.md** - Frontend enhancements
- **4-typing.md** - Type system improvements
- **5-interpreter.md** - Interpreter enhancements
- **6-cli.md** - CLI tooling
- **7-lsp.md** - LSP integration
- **8-polish.md** - Quality verification

### References

Reference materials loaded on-demand:
- **quality-standards.md** - Phase file structure requirements
- **verification-checklist.md** - Pre-completion checklist
- **phase-mapping.md** - Category to implementation file mapping
- **documentation-map.md** - Spec routing guide

### History

Historical context and archives:
- **v0.1-summary.md** - v0.1 completion details and technical debt

---

## Usage

**Primary entry point:** `../STATUS.md` (the dashboard)

**AI Agent workflow:**
1. Read `STATUS.md` for current state and progress summary
2. Click category link to view detailed phase list in `trackers/`
3. Reference `references/` as needed for standards/checklists

**Human workflow:**
- Point AI agents to `STATUS.md`: "Read STATUS.md and continue"
- View category progress in the dashboard table
- Click through to detailed trackers for full phase lists

---

## Handoff Protocol

When completing a phase:
1. Update relevant tracker file (mark ✅)
2. Update `../STATUS.md` (current phase + percentages)
3. Verify sync (counts match)
4. Commit both files together
