# GATE 6: Update Status (Structured Development Only)

**Condition:** Structured development workflow, all gates passed

---

## Action

Use `atlas-dev phase complete` to update tracking database.

**Command:**
```bash
atlas-dev phase complete <path> \
  -d "Brief description" \
  --tests N \
  --commit
```

**Required:**
- `<path>` - Full phase path (e.g., `phases/stdlib/phase-07b-hashset.md`)
- `-d` - Description of what was completed
- `--tests N` - Number of tests added in this phase
- `--commit` - Create git commit automatically

**Output:**
- JSON with updated progress
- Git commit created (if --commit flag used)
- Database updated with completion date

**Example:**
```bash
atlas-dev phase complete phases/stdlib/phase-07b-hashset.md \
  -d "HashSet with 25 tests, 100% parity" \
  --tests 25 \
  --commit
```

**NEVER:**
- Edit STATUS.md manually (deprecated, database is truth)
- Edit tracker files manually (deprecated, database is truth)
- Skip this step (required for progress tracking)

---

**BLOCKING:** Required for structured development only.

**Next:** Done
